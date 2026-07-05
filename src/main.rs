//! Glaux — ローカルオフライン AI チャット

// Windows: コンソール窓を出さない GUI アプリとして起動
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod api;
mod app;
mod config;
mod context;
mod debug_agent_log;
mod model_state;
mod paths;
mod startup_progress;
mod runtime;
mod theme;
mod ui;

// build.rs が生成（同梱 CJK フォント用）
include!(concat!(env!("OUT_DIR"), "/embedded_font.rs"));

use anyhow::Result;
use paths::APP_TITLE;
use std::sync::Arc;

fn viewport_icon() -> Arc<egui::IconData> {
    let image = image::load_from_memory(include_bytes!("../Owl-Bot.png")).expect("Owl-Bot.png");
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Arc::new(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn install_panic_hook() {
    // #region agent log
    std::panic::set_hook(Box::new(|info| {
        crate::debug_agent_log::agent_log(
            "H6",
            "main.rs:panic",
            "panic",
            serde_json::json!({ "info": info.to_string() }),
        );
    }));
    // #endregion
}

fn main() -> Result<()> {
    install_panic_hook();
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H6",
        "main.rs:entry",
        "process start",
        serde_json::json!({ "renderer": "wgpu" }),
    );
    // #endregion

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 720.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title(APP_TITLE)
            .with_icon(viewport_icon()),
        ..Default::default()
    };
    let run_result = eframe::run_native(
        APP_TITLE,
        native_options,
        Box::new(|cc| Ok(Box::new(app::GlauxApp::new(cc)))),
    );
    // #region agent log
    crate::debug_agent_log::agent_log(
        "H6",
        "main.rs:exit",
        "run_native finished",
        serde_json::json!({
            "ok": run_result.is_ok(),
            "err": run_result.as_ref().err().map(|e| e.to_string()),
        }),
    );
    // #endregion
    run_result.map_err(|e| anyhow::anyhow!("{e}"))
}
