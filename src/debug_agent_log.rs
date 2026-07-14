//! Debug-mode NDJSON logger (session 5a290a + 0c72de)
//!
//! 出力を再開するときは `ENABLED` を `true` に戻す。

use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

/// デバッグ NDJSON をファイルへ書くか。配布・通常運用では false。
const ENABLED: bool = false;

// #region agent log
fn log_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    paths.push(crate::paths::app_root().join("debug-5a290a.log"));
    paths.push(crate::runtime::data_dir().join("debug-5a290a.log"));
    paths
}

fn debug_session_paths() -> Vec<PathBuf> {
    let mut paths = vec![];
    paths.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("debug-0c72de.log"));
    paths.push(crate::paths::app_root().join("debug-0c72de.log"));
    paths.push(crate::runtime::data_dir().join("debug-0c72de.log"));
    paths
}

pub fn debug_session_log(
    hypothesis_id: &str,
    location: &str,
    message: &str,
    data: serde_json::Value,
) {
    if !ENABLED {
        return;
    }
    let entry = json!({
        "sessionId": "0c72de",
        "hypothesisId": hypothesis_id,
        "location": location,
        "message": message,
        "data": data,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    });
    let line = entry.to_string();
    for path in debug_session_paths() {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "{line}");
        }
    }
}

pub fn agent_log(hypothesis_id: &str, location: &str, message: &str, data: serde_json::Value) {
    if !ENABLED {
        return;
    }
    let entry = json!({
        "sessionId": "5a290a",
        "hypothesisId": hypothesis_id,
        "location": location,
        "message": message,
        "data": data,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    });
    let line = entry.to_string();
    for path in log_paths() {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "{line}");
        }
    }
}

pub fn log_input_events(
    ctx: &egui::Context,
    focused: bool,
    len_before: usize,
    nl_before: usize,
    len_after: usize,
    nl_after: usize,
) {
    if !ENABLED {
        return;
    }
    let input_changed = len_before != len_after || nl_before != nl_after;
    ctx.input(|i| {
        for event in &i.events {
            match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if *key == egui::Key::Enter {
                        agent_log(
                            "H1",
                            "chat.rs:KeyEnter",
                            "Enter key pressed",
                            json!({
                                "focused": focused,
                                "shift": modifiers.shift,
                                "ctrl": modifiers.ctrl,
                                "alt": modifiers.alt,
                            }),
                        );
                    }
                }
                egui::Event::Ime(ime) => {
                    let (kind, preview_len, has_nl) = match ime {
                        egui::ImeEvent::Enabled => ("Enabled", 0usize, false),
                        egui::ImeEvent::Disabled => ("Disabled", 0, false),
                        egui::ImeEvent::Preedit(s) => ("Preedit", s.chars().count(), s.contains('\n')),
                        egui::ImeEvent::Commit(s) => ("Commit", s.chars().count(), s.contains('\n')),
                    };
                    agent_log(
                        "H2",
                        "chat.rs:Ime",
                        "IME event",
                        json!({
                            "kind": kind,
                            "preview_len": preview_len,
                            "has_newline": has_nl,
                            "focused": focused,
                        }),
                    );
                }
                egui::Event::Text(s) => {
                    agent_log(
                        "H4",
                        "chat.rs:EventText",
                        "Text event",
                        json!({
                            "len": s.len(),
                            "has_newline": s.contains('\n'),
                            "focused": focused,
                        }),
                    );
                }
                _ => {}
            }
        }
    });

    if input_changed {
        agent_log(
            "H1",
            "chat.rs:input_delta",
            "Input buffer changed",
            json!({
                "focused": focused,
                "len_before": len_before,
                "len_after": len_after,
                "nl_before": nl_before,
                "nl_after": nl_after,
                "nl_delta": nl_after as i64 - nl_before as i64,
            }),
        );
    }
}
// #endregion
