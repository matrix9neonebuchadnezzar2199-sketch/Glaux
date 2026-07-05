//! llama-server 子プロセス

use anyhow::{Context, Result};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use sysinfo::{MemoryRefreshKind, Pid, ProcessesToUpdate, RefreshKind, System};

use super::append_log;
use super::extractor::RuntimePaths;
use crate::config::{
    LLAMA_BATCH_SIZE, LLAMA_CTX_CHECKPOINTS, LLAMA_FLASH_ATTN, LLAMA_KV_CACHE_TYPE,
    LLAMA_THREADS, LLAMA_UBATCH_SIZE,
};
use crate::startup_progress::{ProgressCallback, StartupPhase, StartupProgress};

pub struct MemoryStatus {
    pub available_mb: u64,
    pub required_mb: u64,
    pub model_mb: u64,
    pub sufficient: bool,
}

/// モデル GGUF サイズとコンテキスト長から起動に必要な空きメモリ（MB）を見積もる。
pub fn estimate_required_memory_mb(model_path: &std::path::Path, ctx_len: u32, floor_mb: u64) -> u64 {
    let model_mb = std::fs::metadata(model_path)
        .map(|m| m.len() / 1024 / 1024)
        .unwrap_or(0)
        .max(1);
    // モデル本体 + KV/実行時（ctx 比例・q8 KV 想定）+ 小型 ubatch バッファ
    // --no-warmup / -ub 64 により起動ピーク・常駐バッファを抑えた見積り
    let runtime_mb = 256 + (ctx_len as u64 * 128 / 1000);
    (model_mb + runtime_mb).max(floor_mb)
}

pub fn check_memory_mb(min_mb: u64) -> MemoryStatus {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    );
    sys.refresh_memory();
    let avail = sys.available_memory() / 1024 / 1024;
    MemoryStatus {
        available_mb: avail,
        required_mb: min_mb,
        model_mb: 0,
        sufficient: avail >= min_mb,
    }
}

/// モデルファイルを考慮したメモリチェック。
pub fn check_memory_for_model(
    model_path: &std::path::Path,
    ctx_len: u32,
    floor_mb: u64,
) -> MemoryStatus {
    let required_mb = estimate_required_memory_mb(model_path, ctx_len, floor_mb);
    let mut status = check_memory_mb(required_mb);
    status.required_mb = required_mb;
    status.model_mb = std::fs::metadata(model_path)
        .map(|m| m.len() / 1024 / 1024)
        .unwrap_or(0);
    status
}

pub struct LlamaServerHandle {
    child: Option<Child>,
    pub port: u16,
    pub base_url: String,
}

impl LlamaServerHandle {
    pub fn start(
        paths: &RuntimePaths,
        ctx_len: u32,
        min_memory_mb: u64,
    ) -> Result<Self> {
        Self::start_with_progress(paths, ctx_len, min_memory_mb, &|_| {})
    }

    pub fn start_with_progress(
        paths: &RuntimePaths,
        ctx_len: u32,
        min_memory_mb: u64,
        on_progress: ProgressCallback<'_>,
    ) -> Result<Self> {
        let started = Instant::now();

        if !paths.server_exe.exists() {
            anyhow::bail!(
                "llama-server.exe が見つかりません: {}",
                paths.server_exe.display()
            );
        }
        if !paths.model_gguf.exists() {
            anyhow::bail!("GGUF が見つかりません: {}", paths.model_gguf.display());
        }

        on_progress(StartupProgress::at(
            StartupPhase::CheckingMemory,
            0.28,
            "空きメモリを確認しています…",
            started,
        ));
        let mem = check_memory_for_model(&paths.model_gguf, ctx_len, min_memory_mb);
        // #region agent log
        crate::debug_agent_log::agent_log(
            "H9",
            "process.rs:check_memory",
            "memory check",
            serde_json::json!({
                "available_mb": mem.available_mb,
                "required_mb": mem.required_mb,
                "model_mb": mem.model_mb,
                "floor_mb": min_memory_mb,
                "ctx_len": ctx_len,
                "sufficient": mem.sufficient,
            }),
        );
        // #endregion
        if !mem.sufficient {
            anyhow::bail!(
                "利用可能メモリが不足しています: 約 {} MB（このモデルには約 {} MB 以上必要。モデル約 {} MB + 実行時バッファ。他のアプリを終了するか、より軽いモデルをご利用ください）",
                mem.available_mb,
                mem.required_mb,
                mem.model_mb,
            );
        }
        on_progress(StartupProgress::at(
            StartupPhase::CheckingMemory,
            0.32,
            format!(
                "空きメモリ約 {} MB（必要約 {} MB）",
                mem.available_mb, mem.required_mb
            ),
            started,
        ));

        let port = pick_free_port()?;
        let host = format!("127.0.0.1:{port}");
        let base_url = format!("http://{host}");

        on_progress(StartupProgress::at(
            StartupPhase::SpawningServer,
            0.35,
            "llama-server プロセスを起動しています…",
            started,
        ));

        llama_runtime_preflight(&paths.server_exe)?;

        let model_est_mb = std::fs::metadata(&paths.model_gguf)
            .map(|m| m.len() / 1024 / 1024)
            .unwrap_or(0)
            .max(1);

        let stderr_log = crate::runtime::data_dir().join("llama-server-last.stderr.log");
        let stderr_file = std::fs::File::create(&stderr_log)
            .with_context(|| format!("create {}", stderr_log.display()))?;

        let mut cmd = Command::new(&paths.server_exe);
        if let Some(runtime_dir) = paths.server_exe.parent() {
            cmd.current_dir(runtime_dir);
        }
        cmd.arg("-m")
            .arg(&paths.model_gguf)
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .arg("-c")
            .arg(ctx_len.to_string())
            .arg("-np")
            .arg("1")
            .arg("-t")
            .arg(LLAMA_THREADS.to_string())
            .arg("-b")
            .arg(LLAMA_BATCH_SIZE.to_string())
            .arg("-ub")
            .arg(LLAMA_UBATCH_SIZE.to_string())
            .arg("-ctk")
            .arg(LLAMA_KV_CACHE_TYPE)
            .arg("-ctv")
            .arg(LLAMA_KV_CACHE_TYPE)
            .arg("-ngl")
            .arg("0")
            .arg("--no-op-offload")
            .arg("--device")
            .arg("none")
            // b9760 既定 cache-ram=8192 MiB は低メモリ端末で AV (0xc0000005) の原因になりうる
            .arg("--cache-ram")
            .arg("0")
            .arg("--no-warmup")
            .arg("--no-webui")
            .arg("-fa")
            .arg(LLAMA_FLASH_ATTN)
            .arg("--ctx-checkpoints")
            .arg(LLAMA_CTX_CHECKPOINTS.to_string())
            .env("LLAMA_ARG_N_GPU_LAYERS", "0")
            .env("LLAMA_ARG_DEVICE", "none")
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr_file));

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        append_log(&format!(
            "starting llama-server (CPU-only) on {host} model={}",
            paths.model_gguf.display()
        ));
        let mut child = cmd.spawn().with_context(|| "spawn llama-server")?;
        let child_pid = child.id();
        // #region agent log
        crate::debug_agent_log::agent_log(
            "H4",
            "process.rs:spawn",
            "llama-server spawned",
            serde_json::json!({
                "pid": child_pid,
                "port": port,
                "cwd": paths.server_exe.parent().map(|p| p.display().to_string()),
                "model_est_mb": model_est_mb,
                "model_bytes": std::fs::metadata(&paths.model_gguf).map(|m| m.len()).unwrap_or(0),
                "ctx_len": ctx_len,
                "cache_ram": 0,
                "threads": LLAMA_THREADS,
                "batch": LLAMA_BATCH_SIZE,
                "ubatch": LLAMA_UBATCH_SIZE,
                "kv_cache": LLAMA_KV_CACHE_TYPE,
                "flash_attn": LLAMA_FLASH_ATTN,
                "ctx_checkpoints": LLAMA_CTX_CHECKPOINTS,
                "cmd_flags": "-c -np -t -b -ub -ctk -ctv -ngl --no-op-offload --device --cache-ram 0 --no-warmup --no-webui -fa on --ctx-checkpoints 0",
            }),
        );
        // #endregion

        wait_until_healthy_with_progress(
            &base_url,
            &mut child,
            child_pid,
            model_est_mb,
            &stderr_log,
            Duration::from_secs(600),
            on_progress,
            started,
        )?;
        append_log("llama-server ready");

        on_progress(StartupProgress::at(
            StartupPhase::Done,
            1.0,
            "モデルの読み込みが完了しました",
            started,
        ));

        Ok(Self {
            child: Some(child),
            port,
            base_url,
        })
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            append_log("stopping llama-server");
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

impl Drop for LlamaServerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

fn pick_free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// ランタイムフォルダの DLL 一覧（展開欠落の検出用）
fn runtime_inventory(runtime_dir: &std::path::Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(runtime_dir) else {
        return vec![];
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

/// モデル読込前に llama-server 単体が動くか確認（MSVC 欠落はここで即死し stderr が空になりやすい）
fn probe_llama_server(server_exe: &std::path::Path) -> Result<(i32, String)> {
    let runtime_dir = server_exe
        .parent()
        .context("llama-server parent dir")?;
    let mut cmd = Command::new(server_exe);
    cmd.arg("--version")
        .current_dir(runtime_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output = cmd.output().with_context(|| "probe llama-server --version")?;
    let code = output.status.code().unwrap_or(-1);
    let mut text = String::from_utf8_lossy(&output.stderr).into_owned();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&output.stdout).into_owned();
    }
    Ok((code, text.trim().to_string()))
}

fn format_windows_exit(code: i32) -> String {
    if code == -1073741819 {
        "0xc0000005 (ACCESS_VIOLATION)".to_string()
    } else if code == -1073741515 {
        "0xc0000135 (DLL_NOT_FOUND)".to_string()
    } else {
        format!("{code}")
    }
}

fn llama_runtime_preflight(server_exe: &std::path::Path) -> Result<()> {
    let runtime_dir = server_exe.parent().context("runtime dir")?;
    let inventory = runtime_inventory(runtime_dir);
    let has_vc = inventory.iter().any(|n| n.eq_ignore_ascii_case("vcruntime140.dll"));
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H31",
        "process.rs:preflight",
        "runtime inventory",
        serde_json::json!({
            "runtime_dir": runtime_dir.display().to_string(),
            "file_count": inventory.len(),
            "has_vcruntime140": has_vc,
            "dll_sample": inventory.iter().filter(|n| n.ends_with(".dll")).take(12).collect::<Vec<_>>(),
        }),
    );
    // #endregion

    let (code, probe_out) = probe_llama_server(server_exe)?;
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H31",
        "process.rs:preflight",
        "llama-server --version probe",
        serde_json::json!({
            "exit_code": code,
            "exit_label": format_windows_exit(code),
            "output_head": probe_out.chars().take(400).collect::<String>(),
            "has_vcruntime140": has_vc,
        }),
    );
    // #endregion

    if code == 0 {
        return Ok(());
    }

    let hint = if code == -1073741819 || code == -1073741515 {
        "Visual C++ 再頒布可能パッケージ (x64) のインストールまたは更新が必要な可能性があります。\n\
         https://aka.ms/v14/vcredist/x64\n\
         インストール後に Glaux を再起動してください。"
    } else {
        "llama-server が起動できません。ランタイム DLL の欠落や CPU 非対応の可能性があります。"
    };
    anyhow::bail!(
        "llama-server の事前チェックに失敗しました ({}).\n{}\n{}",
        format_windows_exit(code),
        if probe_out.is_empty() {
            "（出力なし — 多くの場合 MSVC ランタイム不足です）"
        } else {
            &probe_out
        },
        hint
    );
}

fn child_memory_mb(pid: u32, sys: &mut System) -> u64 {
    let pid = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    sys.process(pid)
        .map(|p| p.memory() / 1024 / 1024)
        .unwrap_or(0)
}

fn read_stderr_tail(path: &std::path::Path, max_bytes: usize) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    if bytes.len() <= max_bytes {
        return String::from_utf8_lossy(&bytes).to_string();
    }
    String::from_utf8_lossy(&bytes[bytes.len() - max_bytes..]).to_string()
}

fn wait_until_healthy_with_progress(
    base_url: &str,
    child: &mut Child,
    child_pid: u32,
    model_est_mb: u64,
    stderr_log: &std::path::Path,
    timeout: Duration,
    on_progress: ProgressCallback<'_>,
    started: Instant,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    let health_urls = [
        format!("{base_url}/health"),
        format!("{base_url}/v1/models"),
    ];
    let deadline = Instant::now() + timeout;
    let mut sys = System::new();

    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            let stderr_tail = read_stderr_tail(stderr_log, 16384);
            let proc_mb = child_memory_mb(child_pid, &mut sys);
            // #region agent log
            crate::debug_agent_log::agent_log(
                "H4",
                "process.rs:child_exit",
                "llama-server exited early",
                serde_json::json!({
                    "status": format!("{status}"),
                    "exit_code": status.code().unwrap_or(-1),
                    "exit_label": format_windows_exit(status.code().unwrap_or(-1)),
                    "elapsed_secs": started.elapsed().as_secs(),
                    "stderr_tail": stderr_tail,
                    "proc_mb": proc_mb,
                    "available_mb": check_memory_mb(0).available_mb,
                    "model_est_mb": model_est_mb,
                }),
            );
            // #endregion
            if stderr_tail.is_empty() {
                anyhow::bail!("llama-server が異常終了しました: {status}");
            }
            anyhow::bail!(
                "llama-server が異常終了しました: {status}\n--- stderr ---\n{stderr_tail}\n---\n（ログ: {}）",
                stderr_log.display()
            );
        }

        let proc_mb = child_memory_mb(child_pid, &mut sys);
        let load_ratio = (proc_mb as f32 / model_est_mb as f32).clamp(0.0, 0.98);
        let fraction = 0.38 + load_ratio * 0.60;

        let detail = if proc_mb > 0 {
            format!(
                "メモリ読み込み 約 {proc_mb} / {model_est_mb} MB（CPU・初回は数分かかることがあります）"
            )
        } else {
            "モデルファイルを読み込み開始しています…".to_string()
        };

        on_progress(StartupProgress::at(
            StartupPhase::LoadingModel,
            fraction,
            detail,
            started,
        ));

        for url in &health_urls {
            if client.get(url).send().is_ok() {
                // #region agent log
                crate::debug_agent_log::agent_log(
                    "H5",
                    "process.rs:health_ok",
                    "health check passed",
                    serde_json::json!({
                        "url": url,
                        "proc_mb": proc_mb,
                        "elapsed_secs": started.elapsed().as_secs(),
                    }),
                );
                // #endregion
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H5",
        "process.rs:health_timeout",
        "health check timeout",
        serde_json::json!({
            "timeout_secs": timeout.as_secs(),
            "last_proc_mb": child_memory_mb(child_pid, &mut sys),
        }),
    );
    // #endregion
    anyhow::bail!(
        "llama-server の起動がタイムアウトしました（{} 秒）。メモリ不足やディスク速度を確認してください。",
        timeout.as_secs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_model(size_bytes: u64) -> (std::path::PathBuf, std::path::PathBuf) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("glaux_mem_test_{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let model = dir.join("tiny.gguf");
        let f = std::fs::File::create(&model).unwrap();
        f.set_len(size_bytes).unwrap();
        (dir, model)
    }

    #[test]
    fn estimate_required_memory_uses_reduced_runtime_buffer() {
        let (dir, model) = temp_model(11 * 1024 * 1024);

        let required = estimate_required_memory_mb(&model, 1_000, 0);
        // 11 MB model + 256 runtime + 128 ctx-term = 395 MB（旧式 523 MB より小さい）
        assert_eq!(required, 395);
        assert!(required < 523, "旧 runtime 384 見積りより小さいこと");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn estimate_required_memory_respects_floor() {
        let (dir, model) = temp_model(1);

        let required = estimate_required_memory_mb(&model, 1_000, 2_048);
        assert_eq!(required, 2_048);

        let _ = std::fs::remove_dir_all(dir);
    }
}
