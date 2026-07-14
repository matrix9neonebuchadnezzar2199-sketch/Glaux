//! アプリ設定の読み込み・保存

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// llama-server に渡すコンテキスト長（固定・UI 非公開）
pub const CONTEXT_LENGTH: u32 = 3_000;
/// CPU 推論スレッド数（低メモリ向け）
pub const LLAMA_THREADS: u32 = 2;
/// 論理バッチサイズ
pub const LLAMA_BATCH_SIZE: u32 = 512;
/// 物理バッチサイズ
pub const LLAMA_UBATCH_SIZE: u32 = 128;
/// Flash Attention（未指定時は llama-server 既定 auto と同等）
pub const LLAMA_FLASH_ATTN: &str = "auto";
/// KV キャッシュ量子化（f16 既定より RAM 節約）
pub const LLAMA_KV_CACHE_TYPE: &str = "q8_0";
pub const GEMMA_TEMPERATURE: f32 = 1.0;
pub const GEMMA_TOP_P: f32 = 0.95;
pub const GEMMA_TOP_K: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PromptFormat {
    #[default]
    /// Gemma 4（`<|turn>`）
    Gemma,
    /// Gemma 2（`<start_of_turn>`）
    Gemma2,
    /// ChatML（MiniCPM5 等）
    ChatMl,
    Rakuten,
    Qwen,
    /// Llama 3 / 3.2 Instruct（`<|start_header_id|>`）
    Llama3,
}

impl PromptFormat {
    pub const ALL: [Self; 6] = [
        Self::Gemma,
        Self::Gemma2,
        Self::ChatMl,
        Self::Rakuten,
        Self::Qwen,
        Self::Llama3,
    ];

    pub fn label_ja(self) -> &'static str {
        match self {
            Self::Gemma => "Gemma4系",
            Self::Gemma2 => "Gemma2系",
            Self::ChatMl => "ChatML系（MiniCPM5）",
            Self::Rakuten => "Rakuten系",
            Self::Qwen => "Qwen系",
            Self::Llama3 => "Llama3系",
        }
    }

    pub fn hint_ja(self) -> &'static str {
        match self {
            Self::Gemma => "Gemma 4 など `<|turn>` 形式",
            Self::Gemma2 => "Gemma 2 / JPN-IT など `<start_of_turn>` 形式",
            Self::ChatMl => "MiniCPM5 など（GGUF 内蔵テンプレート）",
            Self::Rakuten => "Rakuten AI 2.0 mini（USER / ASSISTANT）",
            Self::Qwen => "Qwen 2.5 Instruct など",
            Self::Llama3 => "Llama 3 / 3.2 Instruct（Unsloth 等）",
        }
    }

    /// `/v1/chat/completions` + `--jinja` を使う形式
    pub fn uses_jinja_chat_api(self) -> bool {
        matches!(self, Self::ChatMl | Self::Qwen)
    }

    /// (temperature, top_p, top_k)
    pub fn sampling(self) -> (f32, f32, Option<u32>) {
        match self {
            Self::Gemma => (GEMMA_TEMPERATURE, GEMMA_TOP_P, Some(GEMMA_TOP_K)),
            Self::Gemma2 => (0.7, 0.95, Some(64)),
            Self::ChatMl => (0.7, 0.95, None),
            Self::Rakuten => (0.7, 0.95, None),
            Self::Qwen => (0.7, 0.8, Some(20)),
            Self::Llama3 => (0.7, 0.9, None),
        }
    }
}

/// `/v1/chat/completions` 向け chat_template_kwargs（モデル別）
pub fn api_chat_template_kwargs(prompt_format: PromptFormat, model: &str) -> Option<serde_json::Value> {
    // MiniCPM5 は thinking 既定 ON → トークンが reasoning_content のみに入り content が空になる
    if prompt_format == PromptFormat::ChatMl && model.to_ascii_lowercase().contains("minicpm") {
        Some(serde_json::json!({ "enable_thinking": false }))
    } else {
        None
    }
}

/// API 送信用 system メッセージ（モデル別に短縮）
pub fn api_system_content(
    prompt_format: PromptFormat,
    model_file: &str,
    default: &str,
) -> String {
    if prompt_format == PromptFormat::ChatMl
        && model_file.to_ascii_lowercase().contains("minicpm")
    {
        // 長いルール文は MiniCPM5 でメタ発話・ハルシネーションの诱因になりやすい
        return "日本語で簡潔かつ正確に答えてください。".to_string();
    }
    default.to_string()
}

/// モデルファイル名から推奨プロンプト形式を推定する。
pub fn suggested_prompt_format_for_filename(filename: &str) -> PromptFormat {
    if let Some(preset) = ModelPreset::from_filename(filename) {
        return preset.suggested_prompt_format();
    }
    let lower = filename.to_ascii_lowercase();
    if lower.contains("minicpm") {
        return PromptFormat::ChatMl;
    }
    if lower.contains("rakuten") {
        return PromptFormat::Rakuten;
    }
    if lower.contains("qwen") {
        return PromptFormat::Qwen;
    }
    // Gemma 2 を Gemma 4 より先に判定（ファイル名に gemma が共通）
    if lower.contains("gemma-2") || lower.contains("gemma2") || lower.contains("jpn-it") {
        return PromptFormat::Gemma2;
    }
    if lower.contains("gemma") {
        return PromptFormat::Gemma;
    }
    if lower.contains("llama-3")
        || lower.contains("llama3")
        || lower.contains("llama_3")
        || lower == "unsloth.q4_k_m.gguf"
        || (lower.starts_with("unsloth.") && lower.ends_with(".gguf"))
    {
        return PromptFormat::Llama3;
    }
    PromptFormat::Gemma
}

fn default_prompt_format() -> PromptFormat {
    suggested_prompt_format_for_filename(&default_model_file())
}

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
    /// 超軽量・多言語（約 1.1GB Q8_0、MiniCPM5 1B）
    MiniCpm5_1B,
    /// 高品質・非量子化（約 2.1GB F16、MiniCPM5 1B）
    MiniCpm5_1BF16,
    /// 低メモリ端末向け（約 1GB、日本語特化）
    RakutenAi20Mini,
    RakutenAi20MiniQ5,
    RakutenAi20MiniQ8,
    /// 多言語・日本語可（約 1.3GB Q2）
    Qwen25_3B,
    /// Qwen 2.5 3B Q3（品質寄り、約 0.9GB）
    Qwen25_3BQ3,
    /// Gemma 2 2B 日本語 IT（Q4_K_M）
    Gemma2Jpn,
    /// Unsloth Llama 3.2 1B（ファイル名 unsloth.Q4_K_M.gguf）
    Llama32_1BUnsloth,
    #[default]
    Gemma4E2B,
    Gemma4E4B,
}

impl ModelPreset {
    pub const ALL: [Self; 11] = [
        Self::MiniCpm5_1B,
        Self::MiniCpm5_1BF16,
        Self::RakutenAi20Mini,
        Self::RakutenAi20MiniQ5,
        Self::RakutenAi20MiniQ8,
        Self::Qwen25_3B,
        Self::Qwen25_3BQ3,
        Self::Gemma2Jpn,
        Self::Llama32_1BUnsloth,
        Self::Gemma4E2B,
        Self::Gemma4E4B,
    ];

    pub fn filename(self) -> &'static str {
        match self {
            Self::MiniCpm5_1B => "minicpm5-1b-Q8_0.gguf",
            Self::MiniCpm5_1BF16 => "minicpm5-1b-f16.gguf",
            Self::RakutenAi20Mini => "RakutenAI-2.0-mini-instruct-Q4_K_M.gguf",
            Self::RakutenAi20MiniQ5 => "RakutenAI-2.0-mini-instruct-Q5_K_M.gguf",
            Self::RakutenAi20MiniQ8 => "RakutenAI-2.0-mini-instruct-Q8_0.gguf",
            Self::Qwen25_3B => "qwen2.5-3b-instruct-q2_k.gguf",
            Self::Qwen25_3BQ3 => "qwen2.5-3b-instruct-q3_k_m.gguf",
            Self::Gemma2Jpn => "gemma-2-2b-jpn-it-Q4_K_M.gguf",
            Self::Llama32_1BUnsloth => "unsloth.Q4_K_M.gguf",
            Self::Gemma4E2B => "gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf",
            Self::Gemma4E4B => "gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::MiniCpm5_1B => "MiniCPM5 1B (Q8_0)",
            Self::MiniCpm5_1BF16 => "MiniCPM5 1B (F16)",
            Self::RakutenAi20Mini => "Rakuten AI 2.0 mini (Q4)",
            Self::RakutenAi20MiniQ5 => "Rakuten AI 2.0 mini (Q5)",
            Self::RakutenAi20MiniQ8 => "Rakuten AI 2.0 mini (Q8)",
            Self::Qwen25_3B => "Qwen 2.5 3B Instruct (Q2_K)",
            Self::Qwen25_3BQ3 => "Qwen 2.5 3B Instruct (Q3_K_M)",
            Self::Gemma2Jpn => "Gemma 2 2B JPN-IT (Q4_K_M)",
            Self::Llama32_1BUnsloth => "Llama 3.2 1B (Unsloth Q4)",
            Self::Gemma4E2B => "Google Gemma 4 E2B (Unsloth QAT Q2)",
            Self::Gemma4E4B => "Google Gemma 4 E4B (Unsloth QAT)",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::MiniCpm5_1B => "MiniCPM",
            Self::MiniCpm5_1BF16 => "CPMF16",
            Self::RakutenAi20Mini => "Mini",
            Self::RakutenAi20MiniQ5 => "MiniQ5",
            Self::RakutenAi20MiniQ8 => "MiniQ8",
            Self::Qwen25_3B => "Qwen3B",
            Self::Qwen25_3BQ3 => "QwenQ3",
            Self::Gemma2Jpn => "Gemma2",
            Self::Llama32_1BUnsloth => "L32",
            Self::Gemma4E2B => "E2B",
            Self::Gemma4E4B => "E4B",
        }
    }

    pub fn api_model_id(self) -> &'static str {
        match self {
            Self::MiniCpm5_1B => "minicpm5_1b",
            Self::MiniCpm5_1BF16 => "minicpm5_1b_f16",
            Self::RakutenAi20Mini => "rakutenai_2_mini",
            Self::RakutenAi20MiniQ5 => "rakutenai_2_mini_q5",
            Self::RakutenAi20MiniQ8 => "rakutenai_2_mini_q8",
            Self::Qwen25_3B => "qwen2_5_3b",
            Self::Qwen25_3BQ3 => "qwen2_5_3b_q3",
            Self::Gemma2Jpn => "gemma2_2b_jpn",
            Self::Llama32_1BUnsloth => "llama32_1b_unsloth",
            Self::Gemma4E2B => "gemma4_e2b",
            Self::Gemma4E4B => "gemma4_e4b",
        }
    }

    pub fn from_filename(name: &str) -> Option<Self> {
        match name {
            "minicpm5-1b-Q8_0.gguf" => Some(Self::MiniCpm5_1B),
            "minicpm5-1b-f16.gguf" => Some(Self::MiniCpm5_1BF16),
            "RakutenAI-2.0-mini-instruct-Q4_K_M.gguf" => Some(Self::RakutenAi20Mini),
            "RakutenAI-2.0-mini-instruct-Q5_K_M.gguf" => Some(Self::RakutenAi20MiniQ5),
            "RakutenAI-2.0-mini-instruct-Q8_0.gguf" => Some(Self::RakutenAi20MiniQ8),
            "qwen2.5-3b-instruct-q2_k.gguf" | "Qwen2.5-3B-Instruct-Q2_K.gguf" => Some(Self::Qwen25_3B),
            "qwen2.5-3b-instruct-q3_k_m.gguf" | "Qwen2.5-3B-Instruct-Q3_K_M.gguf" => {
                Some(Self::Qwen25_3BQ3)
            }
            "gemma-2-2b-jpn-it-Q4_K_M.gguf" => Some(Self::Gemma2Jpn),
            "unsloth.Q4_K_M.gguf" => Some(Self::Llama32_1BUnsloth),
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
            Self::MiniCpm5_1B => 1_792,
            Self::MiniCpm5_1BF16 => 2_560,
            Self::RakutenAi20Mini => 1_536,
            Self::RakutenAi20MiniQ5 => 1_792,
            Self::RakutenAi20MiniQ8 => 2_048,
            Self::Qwen25_3B => 2_560,
            Self::Qwen25_3BQ3 => 3_072,
            Self::Gemma2Jpn => 2_560,
            Self::Llama32_1BUnsloth => 1_536,
            Self::Gemma4E2B => 3_072,
            Self::Gemma4E4B => 6_000,
        }
    }

    pub fn suggested_prompt_format(self) -> PromptFormat {
        match self {
            Self::MiniCpm5_1B | Self::MiniCpm5_1BF16 => PromptFormat::ChatMl,
            Self::RakutenAi20Mini | Self::RakutenAi20MiniQ5 | Self::RakutenAi20MiniQ8 => {
                PromptFormat::Rakuten
            }
            Self::Qwen25_3B | Self::Qwen25_3BQ3 => PromptFormat::Qwen,
            Self::Gemma2Jpn => PromptFormat::Gemma2,
            Self::Llama32_1BUnsloth => PromptFormat::Llama3,
            Self::Gemma4E2B | Self::Gemma4E4B => PromptFormat::Gemma,
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
        "minicpm5_1b" => Some(ModelPreset::MiniCpm5_1B.filename()),
        "minicpm5_1b_f16" => Some(ModelPreset::MiniCpm5_1BF16.filename()),
        "rakuten_ai_20_mini" => Some(ModelPreset::RakutenAi20Mini.filename()),
        "rakuten_ai_20_mini_q5" => Some(ModelPreset::RakutenAi20MiniQ5.filename()),
        "rakuten_ai_20_mini_q8" => Some(ModelPreset::RakutenAi20MiniQ8.filename()),
        "qwen25_3b" => Some(ModelPreset::Qwen25_3B.filename()),
        "qwen25_3bq3" => Some(ModelPreset::Qwen25_3BQ3.filename()),
        "gemma2_jpn" => Some(ModelPreset::Gemma2Jpn.filename()),
        "llama32_1b_unsloth" => Some(ModelPreset::Llama32_1BUnsloth.filename()),
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
    /// チャットプロンプト形式（モデルファミリー別）
    #[serde(default = "default_prompt_format")]
    pub prompt_format: PromptFormat,
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
            prompt_format: default_prompt_format(),
        }
    }
}

impl AppConfig {
    pub fn model_filename(&self) -> &str {
        &self.model_file
    }
}

/// 設定スキーマ版（マイグレーション用）
const CONFIG_VERSION: u32 = 6;

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
        if version < 6 && !obj.contains_key("prompt_format") {
            let filename = obj
                .get("model_file")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let suggested = suggested_prompt_format_for_filename(filename);
            obj.insert(
                "prompt_format".into(),
                serde_json::to_value(suggested).unwrap_or(serde_json::json!("gemma")),
            );
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
