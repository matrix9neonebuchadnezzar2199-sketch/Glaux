//! メインチャット画面

use crate::app::GlauxApp;
use crate::context::ContextUsageLevel;
use crate::config::{model_display_name, CONTEXT_LENGTH};
use crate::debug_agent_log;
use crate::model_state::ModelRuntimeState;
use crate::paths::APP_TITLE;
use crate::ui::prompt_assist;
use egui::{Align, Color32, CornerRadius, Frame, Label, Layout, Margin, RichText, ScrollArea, Stroke};

/// 吹き出し幅: 親の 82%、ただし 280〜560px にクランプ
const BUBBLE_WIDTH_RATIO: f32 = 0.82;
const BUBBLE_MAX_WIDTH: f32 = 560.0;
const BUBBLE_MIN_WIDTH: f32 = 280.0;
/// チャット末尾（コピー行の下）と入力パネル枠の間の隙間
const CHAT_BOTTOM_GAP: f32 = 20.0;

fn bubble_max_width(available: f32) -> f32 {
    (available * BUBBLE_WIDTH_RATIO).clamp(BUBBLE_MIN_WIDTH, BUBBLE_MAX_WIDTH)
}

pub fn draw_main(app: &mut GlauxApp, ctx: &egui::Context) {
    let theme = app.theme.clone();

    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.visuals_mut().override_text_color = Some(theme.text);
        Frame::new()
            .fill(theme.header_bg)
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(APP_TITLE)
                            .size(18.0)
                            .strong()
                            .color(theme.text),
                    );
                    ui.label(
                        RichText::new("Local offline workspace")
                            .size(12.0)
                            .color(theme.muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(RichText::new("📖").size(18.0))
                            .on_hover_text("ヘルプ")
                            .clicked()
                        {
                            app.show_help = true;
                        }
                        nav_button(ui, "設定", || app.show_settings = true);
                    });
                });
            });
    });

    egui::TopBottomPanel::top("model_bar").show(ctx, |ui| {
        Frame::new()
            .fill(theme.surface)
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    status_pill(
                        ui,
                        app.model_state.label_ja(),
                        state_color(app.model_state, &theme),
                    );
                    ui.label(
                        RichText::new(model_display_name(app.config.model_filename()))
                            .strong()
                            .color(theme.text),
                    );
                    ui.add_space(10.0);
                    if ui
                        .add_enabled(app.model_state.can_start(), egui::Button::new("起動"))
                        .clicked()
                    {
                        app.request_start_model();
                    }
                    if ui
                        .add_enabled(app.model_state.can_stop(), egui::Button::new("停止"))
                        .clicked()
                    {
                        app.stop_server();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let est = app.conversation.estimated_tokens();
                        let max = CONTEXT_LENGTH;
                        let level = app.conversation.usage_level();
                        let color = match level {
                            ContextUsageLevel::Normal => theme.muted,
                            ContextUsageLevel::Caution => theme.warning,
                            ContextUsageLevel::Warning | ContextUsageLevel::Blocked => theme.danger,
                        };
                        ui.label(RichText::new(format!("Context 約 {est} / {max}")).color(color));
                    });
                });
            });

        if let Some(err) = app.error_message.clone() {
            setup_alert(app, ui, &err);
        }
        let level = app.conversation.usage_level();
        if level == ContextUsageLevel::Warning || level == ContextUsageLevel::Blocked {
            alert_frame(
                ui,
                &theme,
                theme.warning,
                "クリーン推奨",
                "コンテキスト概算が90%を超えています。新しい話題に移る前にクリーンしてください。",
            );
        }
        if app.model_state == ModelRuntimeState::Starting {
            draw_startup_progress(ui, app, &theme);
        }
        if let Some(toast) = &app.toast.take() {
            alert_frame(ui, &theme, theme.success, "完了", toast);
        }
    });

    // CentralPanel は全パネルの後に置く必要があるため、入力パネルを先に確保する
    // （後置きだと入力パネルがチャット表示域の下端に重なり、末尾が隠れる）
    egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
        Frame::new()
            .fill(theme.surface)
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::symmetric(14, 12))
            .show(ui, |ui| {
                let len_before = app.input.len();
                let nl_before = app.input.matches('\n').count();
                let response = ui.add(
                    egui::TextEdit::multiline(&mut app.input)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text("メッセージを入力…（改行は Shift+Enter）")
                        // Enter は IME 確定専用。改行は Shift+Enter のみ（ログ H1 確認済み）
                        .return_key(egui::KeyboardShortcut::new(
                            egui::Modifiers::SHIFT,
                            egui::Key::Enter,
                        ))
                        .frame(true),
                );
                debug_agent_log::log_input_events(
                    ctx,
                    response.has_focus(),
                    len_before,
                    nl_before,
                    app.input.len(),
                    app.input.matches('\n').count(),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let can_send = app.model_state.can_chat() && !app.is_streaming;
                    if ui
                        .add_enabled(can_send, egui::Button::new("送信"))
                        .clicked()
                    {
                        app.send_chat();
                    }
                    if ui.button("クリーン").clicked() {
                        app.clean_conversation();
                    }
                    prompt_assist::draw_prompt_assist(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("会話は保存されません")
                                .small()
                                .color(theme.muted),
                        );
                    });
                });
                if let Some(template) = prompt_assist::take_applied_template(ctx) {
                    app.input = template;
                }
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.visuals_mut().panel_fill = theme.panel_bg;

        if app.conversation.messages.is_empty() && !app.is_streaming {
            draw_empty_state(app, ui);
        } else {
            let is_streaming = app.is_streaming;
            // 生成中は常に末尾追従（手動スクロールより優先）
            if is_streaming {
                app.chat_follow_bottom = true;
            }
            let stick = app.chat_follow_bottom || is_streaming;

            let scroll_out = ScrollArea::vertical()
                .id_salt("chat_messages")
                .auto_shrink([false, false])
                .stick_to_bottom(stick)
                .show(ui, |ui| {
                    let bubble_max = bubble_max_width(ui.available_width());
                    let mut pending_copy: Option<String> = None;
                    for msg in &app.conversation.messages {
                        let is_user = msg.role == "user";
                        if is_user {
                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                draw_bubble(
                                    ui,
                                    &msg.content,
                                    &msg.timestamp,
                                    theme.user_bubble,
                                    theme.text,
                                    theme.muted,
                                    bubble_max,
                                    false,
                                );
                            });
                        } else {
                            draw_bubble(
                                ui,
                                &msg.content,
                                &msg.timestamp,
                                theme.bot_bubble,
                                theme.text,
                                theme.muted,
                                bubble_max,
                                true,
                            );
                            ui.horizontal(|ui| {
                                if ui.small_button("コピー").clicked() {
                                    pending_copy = Some(msg.content.clone());
                                }
                            });
                        }
                        ui.add_space(8.0);
                    }
                    if let Some(text) = pending_copy {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(text);
                            app.toast = Some("コピーしました".into());
                        }
                    }

                    // コピー行までスクロール可能にし、入力パネル枠との間に隙間を確保
                    ui.add_space(CHAT_BOTTOM_GAP);
                    let bottom_anchor = ui.allocate_response(egui::vec2(1.0, 1.0), egui::Sense::hover());
                    if stick {
                        bottom_anchor.scroll_to_me(Some(Align::BOTTOM));
                    }
                });

            let max_y =
                (scroll_out.content_size.y - scroll_out.inner_rect.height()).max(0.0);
            let scroll_state = egui::scroll_area::State::load(ctx, scroll_out.id);

            // ストリーミングでバブル高さだけ増える場合、stick_to_bottom だけでは足りないことがある
            if stick {
                if let Some(mut state) = scroll_state {
                    if state.offset.y < max_y - 0.5 {
                        state.offset.y = max_y;
                        state.store(ctx, scroll_out.id);
                    }
                }
            }

            // 上方向へ手動スクロールしたら自動追従を止める（生成中は上で follow を維持）
            if !is_streaming && ui.input(|i| i.smooth_scroll_delta.y < -1.0) {
                app.chat_follow_bottom = false;
            }
        }
    });
}

fn nav_button(ui: &mut egui::Ui, label: &str, mut action: impl FnMut()) {
    if ui.button(label).clicked() {
        action();
    }
}

fn status_pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(color.linear_multiply(0.22))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(99))
        .inner_margin(Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).strong());
        });
}

fn draw_startup_progress(ui: &mut egui::Ui, app: &GlauxApp, theme: &crate::theme::ThemeTokens) {
    let title = if app.reload_message.is_some() {
        "モデル再読み込み"
    } else {
        "モデル起動中"
    };

    Frame::new()
        .fill(theme.accent.linear_multiply(0.10))
        .stroke(Stroke::new(1.0, theme.accent))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(theme.accent));
            if let Some(msg) = &app.reload_message {
                ui.label(RichText::new(msg).small().color(theme.muted));
            }

            if let Some(prog) = &app.startup_progress {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(prog.phase.label_ja())
                        .color(theme.text),
                );
                ui.add(
                    egui::ProgressBar::new(prog.fraction)
                        .text(format!("{:.0}%", prog.fraction * 100.0))
                        .fill(theme.accent),
                );
                ui.label(RichText::new(&prog.detail).small().color(theme.muted));
                if prog.elapsed_secs >= 30.0 {
                    ui.label(
                        RichText::new(format!(
                            "経過 {:.0} 秒 — 低速な PC や初回展開では数分かかることがあります",
                            prog.elapsed_secs
                        ))
                        .small()
                        .color(theme.warning),
                    );
                }
            } else {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("準備を開始しています…")
                        .color(theme.muted),
                );
                ui.add(egui::ProgressBar::new(0.05).animate(true));
            }
        });
    ui.add_space(8.0);
}

fn state_color(state: ModelRuntimeState, theme: &crate::theme::ThemeTokens) -> Color32 {
    match state {
        ModelRuntimeState::Ready | ModelRuntimeState::Generating => theme.success,
        ModelRuntimeState::Error => theme.danger,
        ModelRuntimeState::Starting | ModelRuntimeState::Stopping => theme.warning,
        ModelRuntimeState::Stopped => theme.muted,
    }
}

fn alert_frame(
    ui: &mut egui::Ui,
    theme: &crate::theme::ThemeTokens,
    color: Color32,
    title: &str,
    body: &str,
) {
    Frame::new()
        .fill(color.linear_multiply(0.12))
        .stroke(Stroke::new(1.0, color.linear_multiply(0.8)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(color));
            ui.label(RichText::new(body).color(theme.text));
        });
}

fn setup_alert(app: &mut GlauxApp, ui: &mut egui::Ui, error: &str) {
    let theme = app.theme.clone();
    let (server_ok, model_ok, model_path, app_root) = app.artifact_diagnostics();
    let model_name = app.config.model_filename().to_string();
    let files_missing = !server_ok || !model_ok;
    let title = if files_missing {
        "モデル起動に必要なファイルが未配置です"
    } else {
        "モデル起動に失敗しました"
    };
    let subtitle = if files_missing {
        "Glaux.exe と同じフォルダに model/ を作り、GGUF を入れてください。"
    } else {
        "ファイルは見つかっています。起動処理でエラーが発生しました。"
    };

    Frame::new()
        .fill(theme.danger.linear_multiply(0.10))
        .stroke(Stroke::new(1.0, theme.danger.linear_multiply(0.85)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).strong().color(theme.danger));
                    ui.label(RichText::new(subtitle).small().color(theme.muted));
                    ui.add_space(4.0);
                    ui.label(RichText::new(error).color(theme.text));
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "llama-server.exe: {}",
                            if server_ok {
                                "利用可能（EXE 内蔵）"
                            } else {
                                "未配置"
                            }
                        ))
                        .small()
                        .color(theme.text),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{model_name}: {}",
                            if model_ok {
                                "配置済み"
                            } else {
                                "未配置"
                            }
                        ))
                        .small()
                        .color(theme.text),
                    );
                    ui.label(
                        RichText::new(format!("参照フォルダ: {}", app_root.display()))
                            .small()
                            .color(theme.muted),
                    );
                    if model_ok {
                        ui.label(
                            RichText::new(format!("モデルパス: {}", model_path.display()))
                                .small()
                                .color(theme.muted),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "期待パス: {}",
                                app_root.join("model").join(model_name).display()
                            ))
                            .small()
                            .color(theme.muted),
                        );
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("配置フォルダを開く").clicked() {
                        app.open_bundle_folder();
                    }
                });
            });
        });
}

fn draw_empty_state(app: &mut GlauxApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let (server_ok, model_ok) = app.artifacts_ready();
    ui.vertical_centered(|ui| {
        ui.add_space(64.0);
        Frame::new()
            .fill(theme.surface)
            .stroke(Stroke::new(1.0, theme.border))
            .corner_radius(CornerRadius::same(18))
            .inner_margin(Margin::symmetric(28, 24))
            .show(ui, |ui| {
                ui.set_max_width(560.0);
                ui.label(RichText::new(APP_TITLE).heading().color(theme.text));
                ui.label(
                    RichText::new("ローカルで完結する、履歴を残さないチャット環境です。")
                        .color(theme.muted),
                );
                ui.add_space(14.0);
                if server_ok && model_ok {
                    let hint = match app.model_state {
                        ModelRuntimeState::Starting => {
                            "モデルを読み込んでいます。しばらくお待ちください…"
                        }
                        ModelRuntimeState::Error => {
                            if let Some(err) = &app.error_message {
                                err.as_str()
                            } else {
                                "モデルの起動に失敗しました。上部の「起動」を押すか、エラー表示を確認してください。"
                            }
                        }
                        ModelRuntimeState::Stopped => {
                            "ランタイムファイルは配置済みです。上部の「起動」でモデルを開始してください。"
                        }
                        _ => "モデルは起動済みです。メッセージを入力して送信できます。",
                    };
                    let color = if app.model_state == ModelRuntimeState::Error {
                        theme.warning
                    } else {
                        theme.success
                    };
                    ui.label(RichText::new(hint).color(color));
                } else {
                    ui.label(
                        RichText::new(
                            "model/ に GGUF が未配置です。Glaux.exe と同じフォルダに model/ を作ってください。",
                        )
                        .color(theme.warning),
                    );
                    ui.monospace(format!("model/{}", app.config.model_filename()));
                    ui.add_space(8.0);
                    if ui.button("配置フォルダを開く").clicked() {
                        app.open_bundle_folder();
                    }
                }
                ui.add_space(10.0);
                ui.label(
                    RichText::new("会話はメモリ上のみ。アプリ終了またはクリーンで破棄されます。")
                        .small()
                        .color(theme.muted),
                );
            });
    });
}

fn draw_bubble(
    ui: &mut egui::Ui,
    content: &str,
    time: &str,
    bg: Color32,
    text: Color32,
    muted: Color32,
    max_width: f32,
    // AI 応答は max_width いっぱい。ユーザー吹き出しは内容に応じて縮む
    use_full_width: bool,
) {
    if use_full_width {
        ui.set_width(max_width);
    } else {
        ui.set_max_width(max_width);
    }
    Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0, Color32::from_black_alpha(25)))
        .corner_radius(14.0)
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, |ui| {
            let inner_w = (max_width - 28.0).max(120.0);
            ui.set_max_width(inner_w);
            ui.add(Label::new(RichText::new(content).color(text)).wrap());
            ui.label(RichText::new(time).size(12.0).color(muted));
        });
}
