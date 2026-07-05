//! UI テーマトークン

use crate::config::{FontSizePreset, ThemePreset};
use egui::style::WidgetVisuals;
use egui::{Color32, CornerRadius, Stroke};

#[derive(Clone)]
pub struct ThemeTokens {
    pub header_bg: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub bot_bubble: Color32,
    pub user_bubble: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub panel_bg: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub input_bg: Color32,
    pub button_bg: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub font_size: f32,
    pub is_light: bool,
}

impl ThemeTokens {
    pub fn from_preset(theme: ThemePreset, font_size: FontSizePreset) -> Self {
        let font_size = font_size.px();
        match theme {
            ThemePreset::GlauxDark => Self {
                header_bg: Color32::from_rgb(0x06, 0x18, 0x12),
                accent: Color32::from_rgb(0xC9, 0xA2, 0x3C),
                accent_soft: Color32::from_rgb(0x2A, 0x24, 0x16),
                bot_bubble: Color32::from_rgb(0x1E, 0x22, 0x20),
                user_bubble: Color32::from_rgb(0x2C, 0x26, 0x18),
                text: Color32::from_rgb(0xF0, 0xEE, 0xE8),
                muted: Color32::from_rgb(0x9A, 0x9E, 0x98),
                panel_bg: Color32::from_rgb(0x10, 0x12, 0x11),
                surface: Color32::from_rgb(0x16, 0x19, 0x17),
                surface_raised: Color32::from_rgb(0x1F, 0x23, 0x21),
                border: Color32::from_rgb(0x3A, 0x40, 0x3C),
                input_bg: Color32::from_rgb(0x12, 0x15, 0x14),
                button_bg: Color32::from_rgb(0x28, 0x2C, 0x2A),
                danger: Color32::from_rgb(0xE0, 0x5F, 0x5F),
                success: Color32::from_rgb(0x66, 0xB5, 0x87),
                warning: Color32::from_rgb(0xD9, 0xA9, 0x45),
                font_size,
                is_light: false,
            },
            ThemePreset::Light => Self {
                header_bg: Color32::from_rgb(0xFA, 0xF8, 0xF4),
                accent: Color32::from_rgb(0x9A, 0x7B, 0x1E),
                accent_soft: Color32::from_rgb(0xF3, 0xEB, 0xD4),
                bot_bubble: Color32::from_rgb(0xF5, 0xF3, 0xEF),
                user_bubble: Color32::from_rgb(0xFB, 0xF6, 0xE8),
                text: Color32::from_rgb(0x1A, 0x1A, 0x18),
                muted: Color32::from_rgb(0x5C, 0x5C, 0x58),
                panel_bg: Color32::from_rgb(0xFF, 0xFF, 0xFC),
                surface: Color32::from_rgb(0xFF, 0xFF, 0xFC),
                surface_raised: Color32::from_rgb(0xF7, 0xF5, 0xF0),
                border: Color32::from_rgb(0xD8, 0xD4, 0xCC),
                input_bg: Color32::from_rgb(0xFF, 0xFF, 0xFE),
                button_bg: Color32::from_rgb(0xF0, 0xED, 0xE6),
                danger: Color32::from_rgb(0xB8, 0x32, 0x32),
                success: Color32::from_rgb(0x2E, 0x7D, 0x52),
                warning: Color32::from_rgb(0x9A, 0x6B, 0x12),
                font_size,
                is_light: true,
            },
        }
    }

    fn widget_inactive(&self) -> WidgetVisuals {
        widget_visuals(self.input_bg, self.button_bg, self.border, self.text, 6)
    }

    fn widget_hovered(&self) -> WidgetVisuals {
        widget_visuals(
            self.accent_soft,
            self.accent_soft,
            self.accent,
            self.text,
            6,
        )
    }

    fn widget_active(&self) -> WidgetVisuals {
        let fg = if self.is_light {
            Color32::WHITE
        } else {
            Color32::from_rgb(0x12, 0x12, 0x10)
        };
        widget_visuals(self.accent, self.accent, self.accent, fg, 6)
    }

    fn widget_open(&self) -> WidgetVisuals {
        widget_visuals(self.input_bg, self.button_bg, self.accent, self.text, 6)
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(self.font_size + 6.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(self.font_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional((self.font_size - 2.0).max(11.0)),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(self.font_size),
        );

        // egui 既定の dark を clone すると weak_bg_fill が暗色のまま残る。
        // light/dark の正規テンプレートをベースにしてから Glaux 色を上書きする。
        let mut visuals = if self.is_light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };

        visuals.dark_mode = !self.is_light;
        visuals.override_text_color = Some(self.text);
        visuals.window_fill = self.surface;
        visuals.panel_fill = self.panel_bg;
        visuals.extreme_bg_color = self.input_bg;
        visuals.faint_bg_color = self.surface_raised;
        visuals.code_bg_color = self.surface_raised;
        visuals.warn_fg_color = self.warning;
        visuals.error_fg_color = self.danger;
        visuals.hyperlink_color = self.accent;
        visuals.window_stroke.color = self.border;

        visuals.widgets.noninteractive =
            widget_visuals(self.surface, self.surface, self.border, self.text, 4);
        visuals.widgets.inactive = self.widget_inactive();
        visuals.widgets.hovered = self.widget_hovered();
        visuals.widgets.active = self.widget_active();
        visuals.widgets.open = self.widget_open();

        visuals.selection.bg_fill = self.accent;
        visuals.selection.stroke.color = if self.is_light {
            Color32::WHITE
        } else {
            self.text
        };

        style.visuals = visuals;
        style.spacing.slider_width = 180.0;
        style.spacing.interact_size = egui::vec2(44.0, 28.0);
        ctx.set_style(style);
    }
}

fn widget_visuals(
    bg_fill: Color32,
    weak_bg_fill: Color32,
    border: Color32,
    text: Color32,
    radius: u8,
) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill,
        weak_bg_fill,
        bg_stroke: Stroke::new(1.0, border),
        fg_stroke: Stroke::new(1.0, text),
        corner_radius: CornerRadius::same(radius),
        expansion: 0.0,
    }
}
