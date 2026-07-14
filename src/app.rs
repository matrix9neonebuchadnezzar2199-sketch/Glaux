//! メインアプリケーション

use crate::api::chat::{self, ChatRequest, StreamEvent};
use crate::config::{
    load_config, model_api_id, model_display_name, model_min_memory_mb, api_system_content,
    AppConfig, CONTEXT_LENGTH,
};
use crate::context::{ContextUsageLevel, ConversationContext};
use crate::model_state::ModelRuntimeState;
use crate::paths::{self, open_bundle_root};
use crate::runtime::LlamaServerHandle;
use crate::runtime::{append_log, resolve_runtime_with_progress, RuntimePaths};
use crate::startup_progress::StartupProgress;
use crate::theme::ThemeTokens;
use crate::ui;
use eframe::egui::{self, FontFamily};
use epaint::text::{FontData, FontInsert, FontPriority, InsertFontFamily};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const SYSTEM_PROMPT: &str = "日本語で簡潔かつ正確に答えてください。前置き・自己紹介・役割の宣言は不要です。ユーザーが指定した形式とルールに従って回答してください。";

pub enum WorkerMsg {
    StartupProgress(StartupProgress),
    ServerReady(Result<(LlamaServerHandle, RuntimePaths), String>),
    Stream(StreamEvent),
}

/// 設定保存前のスナップショット（ランタイム再起動判定用）
#[derive(Debug, Clone)]
pub struct SettingsSnapshot {
    pub model_file: String,
}

pub struct GlauxApp {
    pub config: AppConfig,
    pub theme: ThemeTokens,
    pub model_state: ModelRuntimeState,
    pub error_message: Option<String>,
    pub toast: Option<String>,

    pub conversation: ConversationContext,
    pub input: String,
    pub streaming_buffer: String,
    pub is_streaming: bool,

    pub runtime_paths: Option<RuntimePaths>,
    pub server: Option<LlamaServerHandle>,

    pub show_help: bool,
    pub show_settings: bool,
    pub show_send_confirm: bool,
    pub pending_force_send: bool,

    pub help_text: String,
    /// ストリーミング中にチャット末尾へ自動スクロールする
    pub chat_follow_bottom: bool,
    /// 設定反映のためモデル再読み込み中に表示するメッセージ
    pub reload_message: Option<String>,
    /// 起動フェーズの進捗（プログレスバー用）
    pub startup_progress: Option<StartupProgress>,
    /// Glaux + llama-server 合計 RSS（MB）。ライブ更新。
    pub runtime_memory_mb: Option<u64>,
    /// メモリ表示の最終更新時刻
    memory_refreshed_at: Instant,

    worker_rx: Receiver<WorkerMsg>,
    worker_tx: Sender<WorkerMsg>,
}

impl GlauxApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        let theme = ThemeTokens::from_preset(config.theme, config.font_size);
        let (tx, rx) = mpsc::channel();

        let help_text = include_str!("../assets/texts/help.md").to_string();
        configure_cjk_fonts(&cc.egui_ctx);

        let app = Self {
            config,
            theme,
            model_state: ModelRuntimeState::Stopped,
            error_message: None,
            toast: None,
            conversation: ConversationContext::new(SYSTEM_PROMPT),
            input: String::new(),
            streaming_buffer: String::new(),
            is_streaming: false,
            runtime_paths: None,
            server: None,
            show_help: false,
            show_settings: false,
            show_send_confirm: false,
            pending_force_send: false,
            help_text,
            chat_follow_bottom: true,
            reload_message: None,
            startup_progress: None,
            runtime_memory_mb: None,
            memory_refreshed_at: Instant::now()
                .checked_sub(Duration::from_secs(2))
                .unwrap_or_else(Instant::now),
            worker_rx: rx,
            worker_tx: tx,
        };

        // #region agent log
        crate::debug_agent_log::agent_log(
            "H0",
            "app.rs:new",
            "GlauxApp initialized",
            serde_json::json!({
                "model": app.config.model_filename(),
            }),
        );
        // #endregion

        app
    }

    pub fn request_start_model(&mut self) {
        if !self.model_state.can_start() {
            return;
        }
        self.model_state = ModelRuntimeState::Starting;
        self.error_message = None;
        self.startup_progress = Some(StartupProgress::initial(std::time::Instant::now()));

        let model_file = self.config.model_filename().to_string();
        let min_mem = model_min_memory_mb(&model_file);
        let tx = self.worker_tx.clone();

        // #region agent log
        {
            let (server_ok, model_ok, model_path, app_root) = self.artifact_diagnostics();
            crate::debug_agent_log::agent_log(
                "H1",
                "app.rs:request_start_model",
                "startup requested",
                serde_json::json!({
                    "model": model_file,
                    "server_ok": server_ok,
                    "model_ok": model_ok,
                    "model_path": model_path.display().to_string(),
                    "app_root": app_root.display().to_string(),
                    "min_mem_mb": min_mem,
                    "embedded": crate::runtime::runtime_embedded(),
                }),
            );
        }
        // #endregion

        thread::spawn(move || {
            let started = std::time::Instant::now();
            let report = |p: StartupProgress| {
                let _ = tx.send(WorkerMsg::StartupProgress(p));
            };

            let paths = match resolve_runtime_with_progress(&model_file, &report) {
                Ok(p) => {
                    // #region agent log
                    crate::debug_agent_log::agent_log(
                        "H1",
                        "app.rs:resolve_runtime",
                        "paths resolved",
                        serde_json::json!({
                            "server_exe": p.server_exe.display().to_string(),
                            "model_gguf": p.model_gguf.display().to_string(),
                            "server_exists": p.server_exe.is_file(),
                            "model_exists": p.model_gguf.is_file(),
                        }),
                    );
                    // #endregion
                    p
                }
                Err(e) => {
                    // #region agent log
                    crate::debug_agent_log::agent_log(
                        "H1",
                        "app.rs:resolve_runtime",
                        "resolve failed",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    // #endregion
                    let _ = tx.send(WorkerMsg::ServerReady(Err(e.to_string())));
                    return;
                }
            };

            match LlamaServerHandle::start_with_progress(
                &paths,
                CONTEXT_LENGTH,
                min_mem,
                &report,
            ) {
                Ok(handle) => {
                    // #region agent log
                    crate::debug_agent_log::agent_log(
                        "H5",
                        "app.rs:server_ready",
                        "llama-server started",
                        serde_json::json!({
                            "port": handle.port,
                            "base_url": handle.base_url,
                        }),
                    );
                    // #endregion
                    let _ = tx.send(WorkerMsg::ServerReady(Ok((handle, paths))));
                }
                Err(e) => {
                    // #region agent log
                    crate::debug_agent_log::agent_log(
                        "H3",
                        "app.rs:server_start",
                        "llama-server start failed",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    // #endregion
                    let _ = tx.send(WorkerMsg::ServerReady(Err(e.to_string())));
                }
            }
            let _ = started;
        });
    }

    pub fn stop_server(&mut self) {
        self.model_state = ModelRuntimeState::Stopping;
        if let Some(mut s) = self.server.take() {
            s.stop();
        }
        self.runtime_memory_mb = None;
        self.model_state = ModelRuntimeState::Stopped;
    }

    /// Glaux + llama-server の合計メモリを約1秒間隔で更新する。
    pub fn refresh_runtime_memory(&mut self) {
        let running = matches!(
            self.model_state,
            ModelRuntimeState::Ready
                | ModelRuntimeState::Starting
                | ModelRuntimeState::Generating
        );
        if !running {
            self.runtime_memory_mb = None;
            return;
        }
        if self.memory_refreshed_at.elapsed() < Duration::from_secs(1) {
            return;
        }
        let llama_pid = self.server.as_ref().map(|s| s.child_pid);
        // Starting 中はまだ handle が無いことがある → Glaux 本体のみ
        self.runtime_memory_mb = Some(crate::runtime::combined_runtime_memory_mb(llama_pid));
        self.memory_refreshed_at = Instant::now();
    }

    pub fn send_chat(&mut self) {
        self.send_chat_inner(false);
    }

    fn send_chat_inner(&mut self, force: bool) {
        if !self.model_state.can_chat() || self.is_streaming {
            return;
        }

        let level = self.conversation.usage_level();
        if level == ContextUsageLevel::Blocked && !force {
            self.show_send_confirm = true;
            return;
        }

        let t = self.input.trim().to_string();
        if t.is_empty() {
            return;
        }
        self.input.clear();
        self.conversation.push_user(t);

        self.conversation.push_assistant(String::new());
        self.streaming_buffer.clear();
        self.is_streaming = true;
        self.chat_follow_bottom = true;
        self.model_state = ModelRuntimeState::Generating;

        let base_url = self
            .server
            .as_ref()
            .map(|s| s.base_url.clone())
            .unwrap_or_default();

        let (temperature, top_p, top_k) = self.config.prompt_format.sampling();
        let model_file = self.config.model_filename().to_string();
        let system_for_api = api_system_content(
            self.config.prompt_format,
            &model_file,
            &self.conversation.system_prompt,
        );
        let req = ChatRequest {
            model: model_api_id(&model_file),
            messages: self
                .conversation
                .to_api_messages(self.config.prompt_format, &model_file),
            prompt_format: self.config.prompt_format,
            temperature,
            top_p,
            stream: true,
            top_k,
        };

        // #region agent log
        crate::debug_agent_log::debug_session_log(
            "H3",
            "app.rs:send_chat_inner",
            "chat request config",
            serde_json::json!({
                "runId": "post-fix-v2",
                "model_file": model_file,
                "prompt_format": format!("{:?}", self.config.prompt_format),
                "uses_jinja_chat_api": self.config.prompt_format.uses_jinja_chat_api(),
                "system_for_api": system_for_api,
                "temperature": req.temperature,
                "top_p": req.top_p,
                "top_k": req.top_k,
                "message_count": req.messages.len(),
            }),
        );
        // #endregion

        let tx = self.worker_tx.clone();
        let (stx, srx) = mpsc::channel();
        let forward_tx = tx.clone();
        thread::spawn(move || {
            while let Ok(ev) = srx.recv() {
                let _ = forward_tx.send(WorkerMsg::Stream(ev));
            }
        });
        thread::spawn(move || {
            if let Err(e) = chat::chat_completion_stream(&base_url, req, stx) {
                let _ = tx.send(WorkerMsg::Stream(StreamEvent::Error(e.to_string())));
            }
        });
    }

    pub fn clean_conversation(&mut self) {
        if self.is_streaming {
            self.is_streaming = false;
            self.model_state = if self.server.is_some() {
                ModelRuntimeState::Ready
            } else {
                ModelRuntimeState::Stopped
            };
        }
        self.conversation.clear();
        self.streaming_buffer.clear();
        self.toast = Some("会話をクリーンしました".into());
    }

    fn poll_worker(&mut self) {
        while let Ok(msg) = self.worker_rx.try_recv() {
            match msg {
                WorkerMsg::StartupProgress(p) => {
                    self.startup_progress = Some(p);
                }
                WorkerMsg::ServerReady(Ok((handle, paths))) => {
                    self.runtime_paths = Some(paths);
                    self.server = Some(handle);
                    self.model_state = ModelRuntimeState::Ready;
                    self.error_message = None;
                    self.startup_progress = None;
                    // 起動直後にすぐメモリを取る
                    self.memory_refreshed_at = Instant::now()
                        .checked_sub(Duration::from_secs(2))
                        .unwrap_or_else(Instant::now);
                    self.refresh_runtime_memory();
                    if self.reload_message.take().is_some() {
                        self.toast = Some("モデルの再読み込みが完了しました".into());
                    }
                }
                WorkerMsg::ServerReady(Err(e)) => {
                    self.model_state = ModelRuntimeState::Error;
                    self.error_message = Some(e);
                    self.reload_message = None;
                    self.startup_progress = None;
                    self.runtime_memory_mb = None;
                }
                WorkerMsg::Stream(ev) => match ev {
                    StreamEvent::Token(t) => {
                        self.streaming_buffer.push_str(&t);
                        self.conversation
                            .update_last_assistant(&self.streaming_buffer);
                    }
                    StreamEvent::Done => {
                        self.is_streaming = false;
                        self.model_state = ModelRuntimeState::Ready;
                    }
                    StreamEvent::Error(e) => {
                        self.is_streaming = false;
                        self.model_state = ModelRuntimeState::Ready;
                        self.error_message = Some(e);
                    }
                },
            }
        }

        if self.pending_force_send {
            self.pending_force_send = false;
            self.send_chat_inner(true);
        }
    }

    pub fn apply_config_theme(&mut self) {
        self.theme = ThemeTokens::from_preset(self.config.theme, self.config.font_size);
    }

    pub fn artifacts_ready(&self) -> (bool, bool) {
        let server_ok = crate::runtime::server_available();
        let model_ok = paths::resolve_model_path(self.config.model_filename()).is_file();
        (server_ok, model_ok)
    }

    /// エラー表示用: 参照パスと配置状態
    pub fn artifact_diagnostics(&self) -> (bool, bool, PathBuf, PathBuf) {
        let model_name = self.config.model_filename();
        let model_path = paths::resolve_model_path(model_name);
        let root = paths::app_root();
        let (server_ok, model_ok) = self.artifacts_ready();
        (server_ok, model_ok, model_path, root)
    }

    pub fn open_bundle_folder(&mut self) {
        let root = open_bundle_root();
        if let Err(e) = fs::create_dir_all(root.join("model")) {
            self.error_message = Some(format!("model フォルダを作成できません: {e}"));
            return;
        }
        if let Err(e) = Command::new("explorer").arg(&root).spawn() {
            self.error_message = Some(format!("フォルダを開けません: {e}"));
        }
    }

    /// 設定保存後にランタイムへ反映（モデル変更時は再読み込み）
    pub fn apply_saved_settings(&mut self, before: SettingsSnapshot) {
        let model_changed = before.model_file != self.config.model_file;

        if !model_changed {
            self.toast = Some("設定を保存しました".into());
            return;
        }

        let was_active = self.server.is_some()
            || matches!(
                self.model_state,
                ModelRuntimeState::Ready
                    | ModelRuntimeState::Generating
                    | ModelRuntimeState::Starting
            );

        self.stop_server();

        self.toast = Some("設定を保存しました".into());

        if was_active {
            self.reload_message = Some(format!(
                "{} に変更 — モデルを再読み込みしています…",
                model_display_name(self.config.model_filename())
            ));

            self.request_start_model();
        } else {
            self.toast = Some(
                "設定を保存しました。モデルの反映には「起動」が必要です。".into(),
            );
        }
    }
}

fn configure_cjk_fonts(ctx: &egui::Context) {
    // .ttc は環境によって 0xc0000005 の原因になるため .ttf / .otf のみ。
    // default_fonts（Ubuntu/Hack）は CJK 非対応のため日本語が □ になる。
    #[cfg(windows)]
    const CANDIDATES: &[(&str, &str)] = &[
        ("hgp_gothic", "C:/Windows/Fonts/HGRSMP.TTF"),
        ("yumin", "C:/Windows/Fonts/yumin.ttf"),
        ("noto_jp_vf", "C:/Windows/Fonts/NotoSansJP-VF.ttf"),
        ("meiryo_ui", "C:/Windows/Fonts/meiryo.ttf"),
    ];
    #[cfg(not(windows))]
    const CANDIDATES: &[(&str, &str)] = &[];

    for (name, path) in CANDIDATES {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".ttc") {
            continue;
        }
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        if bytes.len() < 1024 {
            continue;
        }
        // #region agent log
        crate::debug_agent_log::agent_log(
            "H7",
            "app.rs:configure_cjk_fonts",
            "loading font",
            serde_json::json!({ "name": name, "path": path, "bytes": bytes.len() }),
        );
        // #endregion
        ctx.add_font(FontInsert::new(
            *name,
            FontData::from_owned(bytes),
            vec![
                InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Highest,
                },
                InsertFontFamily {
                    family: FontFamily::Monospace,
                    priority: FontPriority::Lowest,
                },
            ],
        ));
        return;
    }

    if try_embedded_cjk_font(ctx) {
        return;
    }

    // #region agent log
    crate::debug_agent_log::agent_log(
        "H7",
        "app.rs:configure_cjk_fonts",
        "no CJK font found",
        serde_json::json!({ "fallback": "default_fonts_only" }),
    );
    // #endregion
}

/// `build.rs` が生成する同梱フォント（`assets/fonts/NotoSansJP-Regular.ttf` がある場合のみ）
fn try_embedded_cjk_font(ctx: &egui::Context) -> bool {
    let bytes = crate::glaux_embedded_font::NOTO_SANS_JP;
    if bytes.len() < 1024 {
        return false;
    }
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H7",
        "app.rs:try_embedded_cjk_font",
        "loading embedded font",
        serde_json::json!({ "bytes": bytes.len() }),
    );
    // #endregion
    ctx.add_font(FontInsert::new(
        "noto_jp_embedded",
        FontData::from_static(bytes),
        vec![
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Highest,
            },
            InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    ));
    true
}

impl eframe::App for GlauxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();
        self.refresh_runtime_memory();
        self.theme.apply_to_ctx(ctx);
        if self.toast.is_some()
            || self.is_streaming
            || self.reload_message.is_some()
            || self.model_state == ModelRuntimeState::Starting
        {
            ctx.request_repaint();
        } else if self.runtime_memory_mb.is_some() {
            // メモリライブ表示のため約1秒間隔で再描画
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        ui::draw_main(self, ctx);

        if self.show_help {
            ui::draw_help(self, ctx);
        }
        if self.show_settings {
            ui::draw_settings(self, ctx);
        }
        if self.show_send_confirm {
            ui::draw_send_confirm(self, ctx);
        }
    }

    fn on_exit(&mut self) {
        // #region agent log
        crate::debug_agent_log::agent_log(
            "H6",
            "app.rs:on_exit",
            "application exit",
            serde_json::json!({}),
        );
        // #endregion
        append_log("application exit");
        self.stop_server();
    }
}
