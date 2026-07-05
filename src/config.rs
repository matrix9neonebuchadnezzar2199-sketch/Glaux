//! アプリ設定の読み込み・保存

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// llama-server に渡すコンテキスト長（固定・UI 非公開）。低メモリ向けに 1000。
pub const CONTEXT_LENGTH: u32 = 1_000;
/// CPU 推論スレッド数（低メモリ向け）
pub const LLAMA_THREADS: u32 = 2;
/// 論理バッチサイズ（プロンプト処理時ピーク RAM 抑制）
pub const LLAMA_BATCH_SIZE: u32 = 256;
/// 物理バッチサイズ（計算バッファは ubatch に比例）
pub const LLAMA_UBATCH_SIZE: u32 = 64;
/// Flash Attention（-ctv q8_0 は FA 前提）
pub const LLAMA_FLASH_ATTN: &str = "on";
/// SWA コンテキスト checkpoint 数（0 = 無効・RAM 節約）
pub const LLAMA_CTX_CHECKPOINTS: u32 = 0;
/// KV キャッシュ量子化（f16 既定より RAM 節約）
pub const LLAMA_KV_CACHE_TYPE: &str = "q8_0";
pub const GEMMA_TEMPERATURE: f32 = 1.0;
pub const GEMMA_TOP_P: f32 = 0.95;
pub const GEMMA_TOP_K: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    #[default]
    GlauxDark,
    Light,
}

impl ThemePreset {
    pub fn label_ja(self) -> &'static str {
        match self {
            Self::GlauxDark => "Glaux（ダーク）",
            Self::Light => "ライトモード",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FontSizePreset {
    #[default]
    Medium,
    Small,
    Large,
    XLarge,
}

impl FontSizePreset {
    pub fn label_ja(self) -> &'static str {
        match self {
            Self::Small => "小",
            Self::Medium => "中",
            Self::Large => "大",
            Self::XLarge => "特大",
        }
    }

    pub fn px(self) -> f32 {
        match self {
            Self::Small => 13.0,
            Self::Medium => 15.0,
            Self::Large => 17.0,
            Self::XLarge => 19.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelPreset {
    /// 低メモリ端末向け（約 1GB、日本語特化）
    RakutenAi20Mini,
    /// 多言語・日本語可（約 1.3GB Q2）
    Qwen25_3B,
    /// Qwen 2.5 3B Q3（品質寄り、約 0.9GB）
    Qwen25_3BQ3,
    #[default]
    Gemma4E2B,
    Gemma4E4B,
}

impl ModelPreset {
    pub const ALL: [Self; 5] = [
        Self::RakutenAi20Mini,
        Self::Qwen25_3B,
        Self::Qwen25_3BQ3,
        Self::Gemma4E2B,
        Self::Gemma4E4B,
    ];

    pub fn filename(self) -> &'static str {
        match self {
            Self::RakutenAi20Mini => "RakutenAI-2.0-mini-instruct-Q4_K_M.gguf",
            Self::Qwen25_3B => "qwen2.5-3b-instruct-q2_k.gguf",
            Self::Qwen25_3BQ3 => "qwen2.5-3b-instruct-q3_k_m.gguf",
            Self::Gemma4E2B => "gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf",
            Self::Gemma4E4B => "gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::RakutenAi20Mini => "Rakuten AI 2.0 mini (Q4)",
            Self::Qwen25_3B => "Qwen 2.5 3B Instruct (Q2_K)",
            Self::Qwen25_3BQ3 => "Qwen 2.5 3B Instruct (Q3_K_M)",
            Self::Gemma4E2B => "Google Gemma 4 E2B (Unsloth QAT Q2)",
            Self::Gemma4E4B => "Google Gemma 4 E4B (Unsloth QAT)",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::RakutenAi20Mini => "Mini",
            Self::Qwen25_3B => "Qwen3B",
            Self::Qwen25_3BQ3 => "QwenQ3",
            Self::Gemma4E2B => "E2B",
            Self::Gemma4E4B => "E4B",
        }
    }

    pub fn api_model_id(self) -> &'static str {
        match self {
            Self::RakutenAi20Mini => "rakutenai_2_mini",
            Self::Qwen25_3B => "qwen2_5_3b",
            Self::Qwen25_3BQ3 => "qwen2_5_3b_q3",
            Self::Gemma4E2B => "gemma4_e2b",
            Self::Gemma4E4B => "gemma4_e4b",
        }
    }

    pub fn from_filename(name: &str) -> Option<Self> {
        match name {
            "RakutenAI-2.0-mini-instruct-Q4_K_M.gguf" => Some(Self::RakutenAi20Mini),
            "qwen2.5-3b-instruct-q2_k.gguf" | "Qwen2.5-3B-Instruct-Q2_K.gguf" => Some(Self::Qwen25_3B),
            "qwen2.5-3b-instruct-q3_k_m.gguf" | "Qwen2.5-3B-Instruct-Q3_K_M.gguf" => {
                Some(Self::Qwen25_3BQ3)
            }
            "gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf" | "gemma-4-e2b-it.gguf" => Some(Self::Gemma4E2B),
            "gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf" | "gemma-4-e4b-it.Q4_K_M.gguf" => {
                Some(Self::Gemma4E4B)
            }
            _ => None,
        }
    }

    /// 起動推奨の最小空きメモリ（MB）。実際の判定はモデルファイルサイズから動的に見積もる。
    pub fn min_memory_mb(self) -> u64 {
        match self {
            Self::RakutenAi20Mini => 1_536,
            Self::Qwen25_3B => 2_560,
            Self::Qwen25_3BQ3 => 3_072,
            Self::Gemma4E2B => 3_072,
            Self::Gemma4E4B => 6_000,
        }
    }
}

/// 既知プリセットがあれば表示名、なければファイル名
pub fn model_display_name(filename: &str) -> String {
    ModelPreset::from_filename(filename)
        .map(|p| p.display_name().to_string())
        .unwrap_or_else(|| filename.to_string())
}

/// llama-server API 用 model id
pub fn model_api_id(filename: &str) -> String {
    ModelPreset::from_filename(filename)
        .map(|p| p.api_model_id().to_string())
        .unwrap_or_else(|| {
            filename
                .trim_end_matches(".gguf")
                .replace('.', "_")
                .replace('-', "_")
        })
}

/// 起動推奨の最小空きメモリ（MB）
pub fn model_min_memory_mb(filename: &str) -> u64 {
    if let Some(preset) = ModelPreset::from_filename(filename) {
        return preset.min_memory_mb();
    }
    let path = crate::paths::resolve_model_path(filename);
    let model_mb = std::fs::metadata(&path)
        .map(|m| m.len() / 1024 / 1024)
        .unwrap_or(0)
        .max(512);
    (model_mb + 512).max(2_048)
}

fn default_model_file() -> String {
    let available = crate::paths::list_model_gguf_files();
    if let Some(first) = available.first() {
        return first.clone();
    }
    ModelPreset::Gemma4E2B.filename().to_string()
}

fn preset_id_to_filename(id: &str) -> Option<&'static str> {
    match id {
        "rakuten_ai_20_mini" => Some(ModelPreset::RakutenAi20Mini.filename()),
        "qwen25_3b" => Some(ModelPreset::Qwen25_3B.filename()),
        "qwen25_3bq3" => Some(ModelPreset::Qwen25_3BQ3.filename()),
        "gemma4_e2b" => Some(ModelPreset::Gemma4E2B.filename()),
        "gemma4_e4b" => Some(ModelPreset::Gemma4E4B.filename()),
        _ => None,
    }
}

fn ensure_model_file_on_disk(cfg: &mut AppConfig) {
    let available = crate::paths::list_model_gguf_files();
    if available.is_empty() {
        return;
    }
    if !available.iter().any(|n| n == &cfg.model_file) {
        cfg.model_file = available[0].clone();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemePreset,
    pub font_size: FontSizePreset,
    /// `model/` 内の GGUF ファイル名
    #[serde(default = "default_model_file")]
    pub model_file: String,
    /// 起動時に選択中モデルで llama-server を自動起動する（既定: オフ）
    #[serde(default = "default_auto_start_model")]
    pub auto_start_model: bool,
}

fn default_auto_start_model() -> bool {
    false
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemePreset::default(),
            font_size: FontSizePreset::default(),
            model_file: default_model_file(),
            auto_start_model: false,
        }
    }
}

impl AppConfig {
    pub fn model_filename(&self) -> &str {
        &self.model_file
    }
}

/// 設定スキーマ版（マイグレーション用）
const CONFIG_VERSION: u32 = 5;

/// 旧 config.json からの移行
fn migrate_raw_config(mut v: serde_json::Value) -> AppConfig {
    if let Some(obj) = v.as_object_mut() {
        if let Some(m) = obj.get("model").and_then(|v| v.as_str()) {
            if m == "gemma4_e4_b" {
                obj.insert("model".into(), serde_json::json!("gemma4_e4b"));
            }
        }
        if !obj.contains_key("model_file") {
            if let Some(path) = obj.get("model_path").and_then(|p| p.as_str()) {
                let filename = std::path::Path::new(path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(path);
                obj.insert("model_file".into(), serde_json::json!(filename));
            } else if let Some(id) = obj.get("model").and_then(|m| m.as_str()) {
                if let Some(filename) = preset_id_to_filename(id) {
                    obj.insert("model_file".into(), serde_json::json!(filename));
                } else if id.ends_with(".gguf") {
                    obj.insert("model_file".into(), serde_json::json!(id));
                }
            }
        }
        obj.remove("model");
        obj.remove("model_path");
        if let Some(theme) = obj.get("theme").and_then(|t| t.as_str()) {
            let mapped = match theme {
                "glaux" | "glaux_dark" => "glaux_dark",
                "light" => "light",
                "chat_gpt" | "claude" | "github" | "vscode_dark" => "glaux_dark",
                other => other,
            };
            obj.insert("theme".into(), serde_json::json!(mapped));
        }
        // 旧キー（UI 削除済み）は読み込み時に破棄
        for key in [
            "temperature",
            "top_p",
            "top_k",
            "streaming",
            "context_length",
        ] {
            obj.remove(key);
        }

        let version = obj
            .get("config_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
        if version < 5 {
            obj.insert("auto_start_model".into(), serde_json::json!(false));
        } else if !obj.contains_key("auto_start_model") {
            obj.insert("auto_start_model".into(), serde_json::json!(false));
        }
        obj.insert("config_version".into(), serde_json::json!(CONFIG_VERSION));
    }
    let mut cfg: AppConfig = serde_json::from_value(v).unwrap_or_default();
    ensure_model_file_on_disk(&mut cfg);
    cfg
}

pub fn config_path() -> PathBuf {
    directories::ProjectDirs::from("", "", "Glaux")
        .map(|d| d.config_dir().join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                return migrate_raw_config(v);
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(cfg: &AppConfig) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut v = serde_json::to_value(cfg)?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert("config_version".into(), serde_json::json!(CONFIG_VERSION));
    }
    let json = serde_json::to_string_pretty(&v)?;
    fs::write(path, json)?;
    Ok(())
}
