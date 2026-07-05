//! Drawer / Modal

use crate::app::{GlauxApp, SettingsSnapshot};
use crate::config::{model_display_name, model_min_memory_mb, FontSizePreset, ThemePreset};
use crate::paths::{self, APP_TITLE};
use crate::runtime::check_memory_mb;
use egui::{
    Align, CornerRadius, Frame, Layout, Margin, RichText, ScrollArea, Stroke,
    Ui,
};

pub fn draw_help(app: &mut GlauxApp, ctx: &egui::Context) {
    let mut open = app.show_help;
    egui::Window::new(format!("{APP_TITLE} — ヘルプ"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size([520.0, 560.0])
        .show(ctx, |ui| {
            let theme = app.theme.clone();
            modal_body(
                ui,
                &theme,
                "Help",
                "履歴を残さないローカルチャットとしての使い方",
            );
            ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new(&app.help_text).color(theme.text));
            });
        });
    app.show_help = open;
}

pub fn draw_send_confirm(app: &mut GlauxApp, ctx: &egui::Context) {
    let mut open = app.show_send_confirm;
    let mut do_clean = false;
    let mut do_send = false;
    let mut request_close = false;
    egui::Window::new("コンテキスト上限")
        .open(&mut open)
        .collapsible(false)
        .show(ctx, |ui| {
            let theme = app.theme.clone();
            warning_box(
                ui,
                &theme,
                "コンテキスト概算が上限に達しています。ハルシネーションの恐れがあります。",
            );
            ui.label(
                RichText::new("クリーンするか、このまま送信するか選んでください。")
                    .color(theme.text),
            );
            ui.horizontal(|ui| {
                if ui.button("クリーン").clicked() {
                    do_clean = true;
                }
                if ui.button("送信する").clicked() {
                    do_send = true;
                }
                if ui.button("キャンセル").clicked() {
                    request_close = true;
                }
            });
        });
    if do_clean || do_send || request_close {
        open = false;
    }
    if do_clean {
        app.clean_conversation();
    }
    if do_send {
        app.pending_force_send = true;
    }
    app.show_send_confirm = open;
}

pub fn draw_settings(app: &mut GlauxApp, ctx: &egui::Context) {
    let mut open = app.show_settings;
    let mut close_requested = false;
    let mut save_requested = false;
    let snapshot_at_open = SettingsSnapshot {
        model_file: app.config.model_file.clone(),
    };

    egui::Window::new("設定")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_size([520.0, 480.0])
        .show(ctx, |ui| {
            let theme = app.theme.clone();

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(360.0)
                .show(ui, |ui| {
                    ui.set_min_width(480.0);

                    section(ui, &theme, "外観", |ui| {
                        field_label(ui, &theme, "テーマ");
                        segmented_row(ui, &theme, |ui| {
                            for preset in [ThemePreset::GlauxDark, ThemePreset::Light] {
                                ui.selectable_value(
                                    &mut app.config.theme,
                                    preset,
                                    preset.label_ja(),
                                );
                            }
                        });

                        ui.add_space(10.0);
                        field_label(ui, &theme, "文字サイズ");
                        segmented_row(ui, &theme, |ui| {
                            for size in [
                                FontSizePreset::Small,
                                FontSizePreset::Medium,
                                FontSizePreset::Large,
                                FontSizePreset::XLarge,
                            ] {
                                ui.selectable_value(
                                    &mut app.config.font_size,
                                    size,
                                    size.label_ja(),
                                );
                            }
                        });
                    });

                    section(ui, &theme, "モデル設定", |ui| {
                        field_label(ui, &theme, "使用モデル");
                        let available = paths::list_model_gguf_files();
                        if available.is_empty() {
                            ui.label(
                                RichText::new("model/ に GGUF ファイルがありません")
                                    .color(theme.warning),
                            );
                        } else {
                            let selected_label =
                                model_display_name(app.config.model_filename());
                            egui::ComboBox::from_id_salt("model_file_combo")
                                .selected_text(selected_label)
                                .width(440.0)
                                .show_ui(ui, |ui| {
                                    for name in &available {
                                        let label = model_display_name(name);
                                        ui.selectable_value(
                                            &mut app.config.model_file,
                                            name.clone(),
                                            label,
                                        );
                                    }
                                });

                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(model_display_name(app.config.model_filename()))
                                    .color(theme.text),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(format!(
                                    "ファイル: {}",
                                    app.config.model_filename()
                                ))
                                .small()
                                .color(theme.muted),
                            );
                        }
                    });

                    let min_mem = model_min_memory_mb(app.config.model_filename());
                    let mem = check_memory_mb(min_mem);
                    if !mem.sufficient {
                        warning_box(
                            ui,
                            &theme,
                            &format!(
                                "利用可能メモリ約 {} MB — {} 起動には {} MB 以上推奨",
                                mem.available_mb,
                                model_display_name(app.config.model_filename()),
                                min_mem
                            ),
                        );
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button(ui, &theme, "保存").clicked() {
                    save_requested = true;
                }
                ui.add_space(8.0);
                if ui.button("閉じる").clicked() {
                    close_requested = true;
                }
            });
        });

    if save_requested {
        if let Err(e) = crate::config::save_config(&app.config) {
            app.error_message = Some(e.to_string());
        } else {
            app.apply_config_theme();
            app.apply_saved_settings(snapshot_at_open);
            open = false;
        }
    }
    if close_requested {
        open = false;
    }
    app.show_settings = open;
}

fn modal_body(ui: &mut Ui, theme: &crate::theme::ThemeTokens, title: &str, subtitle: &str) {
    ui.label(RichText::new(title).heading().color(theme.text));
    ui.label(RichText::new(subtitle).color(theme.muted));
    ui.add_space(12.0);
}

fn section(
    ui: &mut Ui,
    theme: &crate::theme::ThemeTokens,
    title: &str,
    body: impl FnOnce(&mut Ui),
) {
    Frame::new()
        .fill(theme.surface_raised)
        .stroke(Stroke::new(1.0, theme.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(theme.accent));
            ui.add_space(10.0);
            body(ui);
        });
    ui.add_space(12.0);
}

fn field_label(ui: &mut Ui, theme: &crate::theme::ThemeTokens, text: &str) {
    ui.label(RichText::new(text).color(theme.text));
    ui.add_space(4.0);
}

fn segmented_row(ui: &mut Ui, theme: &crate::theme::ThemeTokens, body: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(theme.input_bg)
        .stroke(Stroke::new(1.0, theme.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(6, 4))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                body(ui);
            });
        });
}

fn toggle_row(ui: &mut Ui, theme: &crate::theme::ThemeTokens, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.checkbox(value, RichText::new(label).color(theme.text));
    });
    ui.add_space(4.0);
}

fn primary_button(ui: &mut Ui, theme: &crate::theme::ThemeTokens, label: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(label).color(if theme.is_light {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(0x12, 0x12, 0x10)
    }))
    .fill(theme.accent);
    ui.add(btn)
}

fn warning_box(ui: &mut Ui, theme: &crate::theme::ThemeTokens, text: &str) {
    Frame::new()
        .fill(theme.warning.linear_multiply(0.12))
        .stroke(Stroke::new(1.0, theme.warning))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(theme.warning));
        });
    ui.add_space(8.0);
}
