//! Runtime path resolution (embedded extract / dev artifacts).

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{copy, Cursor};
use std::path::PathBuf;
use std::time::Instant;
use zip::ZipArchive;

use crate::paths::{self, resolve_model_path};
use crate::runtime::{append_log, data_dir, embedded};
use crate::startup_progress::{ProgressCallback, StartupPhase, StartupProgress};

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub server_exe: PathBuf,
    pub model_gguf: PathBuf,
}

pub fn resolve_runtime(model_file: &str) -> Result<RuntimePaths> {
    resolve_runtime_with_progress(model_file, &|_| {})
}

pub fn resolve_runtime_with_progress(
    model_file: &str,
    on_progress: ProgressCallback<'_>,
) -> Result<RuntimePaths> {
    let started = Instant::now();
    on_progress(StartupProgress::at(
        StartupPhase::CheckingFiles,
        0.05,
        format!("GGUF を確認: {model_file}"),
        started,
    ));

    let server_exe = resolve_server_exe_with_progress(on_progress, started)?;
    let model_gguf = resolve_model_path(model_file);

    if !model_gguf.is_file() {
        anyhow::bail!(
            "{model_file} が見つかりません: {}\n{}",
            model_gguf.display(),
            paths::bundle_readme_hint()
        );
    }

    let model_mb = fs::metadata(&model_gguf)
        .map(|m| m.len() / 1024 / 1024)
        .unwrap_or(0);
    on_progress(StartupProgress::at(
        StartupPhase::CheckingFiles,
        0.12,
        format!("モデルファイル確認済み（約 {model_mb} MB）"),
        started,
    ));

    Ok(RuntimePaths {
        server_exe,
        model_gguf,
    })
}

fn resolve_server_exe_with_progress(
    on_progress: ProgressCallback<'_>,
    started: Instant,
) -> Result<PathBuf> {
    if let Ok(dev) = std::env::var("GLAUX_DEV_SERVER") {
        let p = PathBuf::from(&dev);
        if p.is_file() {
            return Ok(p);
        }
    }

    let portable = paths::app_root().join("runtime").join("llama-server.exe");
    if portable.is_file() {
        on_progress(StartupProgress::at(
            StartupPhase::CheckingFiles,
            0.15,
            "同梱 runtime/ を使用します",
            started,
        ));
        return Ok(portable);
    }

    if embedded::RUNTIME_BUNDLE_EMBEDDED {
        let dir = ensure_embedded_runtime_with_progress(on_progress, started)?;
        let server = dir.join("llama-server.exe");
        if server.is_file() {
            return Ok(server);
        }
        anyhow::bail!("埋め込みランタイムの展開後も llama-server.exe が見つかりません");
    }

    let dev = paths::dev_artifacts_root().join("llama-server.exe");
    if dev.is_file() {
        return Ok(dev);
    }

    anyhow::bail!(
        "llama-server.exe が利用できません。\n{}",
        paths::bundle_readme_hint()
    )
}

fn embedded_runtime_dir() -> PathBuf {
    data_dir()
        .join("runtime")
        .join(embedded::RUNTIME_BUNDLE_SHA256)
}

fn ensure_embedded_runtime_with_progress(
    on_progress: ProgressCallback<'_>,
    started: Instant,
) -> Result<PathBuf> {
    let dest = embedded_runtime_dir();
    let marker = dest.join(".glaux_runtime_ok");
    let server = dest.join("llama-server.exe");

    if server.is_file()
        && marker.is_file()
        && fs::read_to_string(&marker).ok().as_deref() == Some(embedded::RUNTIME_BUNDLE_SHA256)
    {
        on_progress(StartupProgress::at(
            StartupPhase::CheckingFiles,
            0.18,
            "ランタイムは展開済みです",
            started,
        ));
        return Ok(dest);
    }

    append_log(&format!(
        "extracting embedded runtime to {}",
        dest.display()
    ));
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H2",
        "extractor.rs:extract_start",
        "embedded runtime extract begin",
        serde_json::json!({
            "dest": dest.display().to_string(),
            "zip_entries": embedded::RUNTIME_BUNDLE_EMBEDDED,
        }),
    );
    // #endregion
    on_progress(StartupProgress::at(
        StartupPhase::ExtractingRuntime,
        0.18,
        "初回起動: 内蔵ランタイムを展開しています…",
        started,
    ));
    fs::create_dir_all(&dest).context("runtime extract dir")?;

    let cursor = Cursor::new(embedded::RUNTIME_BUNDLE_ZIP);
    let mut archive = ZipArchive::new(cursor).context("open embedded runtime zip")?;
    let total = archive.len();

    for i in 0..total {
        let mut file = archive.by_index(i).context("zip entry")?;
        let Some(relative) = file.enclosed_name() else {
            continue;
        };
        let outpath = dest.join(relative);
        if file.is_dir() {
            fs::create_dir_all(&outpath).context("zip mkdir")?;
            continue;
        }
        if let Some(parent) = outpath.parent() {
            fs::create_dir_all(parent).context("zip parent dir")?;
        }
        let mut outfile = File::create(&outpath).with_context(|| {
            format!("create extracted file {}", outpath.display())
        })?;
        copy(&mut file, &mut outfile).context("extract runtime file")?;

        let frac = 0.18 + 0.12 * ((i + 1) as f32 / total as f32);
        on_progress(StartupProgress::at(
            StartupPhase::ExtractingRuntime,
            frac,
            format!("ランタイム展開 {}/{} ファイル", i + 1, total),
            started,
        ));
    }

    if !server.is_file() {
        anyhow::bail!("embedded runtime missing llama-server.exe after extract");
    }

    fs::write(&marker, embedded::RUNTIME_BUNDLE_SHA256).context("runtime marker")?;
    append_log("embedded runtime ready");
    Ok(dest)
}
