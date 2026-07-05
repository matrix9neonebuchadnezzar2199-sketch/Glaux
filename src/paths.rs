//! ポータブル配布フォルダと開発用 artifacts のパス解決

use std::path::PathBuf;

/// アプリ表示名（ウィンドウタイトル等）
pub const APP_TITLE: &str = "Glaux -OFFLINE AI Chat-";

/// 実行ファイルのあるディレクトリ（ポータブル配布のルート）
pub fn app_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn dev_artifacts_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("artifacts")
}

/// GGUF モデル配置ディレクトリ（配布: model/、開発: artifacts/model/）
pub fn model_dir() -> PathBuf {
    let portable = app_root().join("model");
    if portable.is_dir() {
        return portable;
    }
    let legacy = app_root().join("models");
    if legacy.is_dir() {
        return legacy;
    }
    // 配布フォルダ（Glaux.exe + README.txt + model/）では dev artifacts にフォールバックしない
    if is_portable_distribution() {
        return portable;
    }
    dev_model_dir()
}

/// dist/Glaux 配布レイアウトか（exe 横に README または model がある）
fn is_portable_distribution() -> bool {
    let root = app_root();
    root.join("README.txt").is_file() || root.join("Glaux.exe").is_file()
}

fn dev_model_dir() -> PathBuf {
    let modern = dev_artifacts_root().join("model");
    if modern.is_dir() {
        return modern;
    }
    dev_artifacts_root().join("models")
}

/// 配布フォルダを開く
pub fn open_bundle_root() -> PathBuf {
    app_root()
}

pub fn model_file_path(filename: &str) -> PathBuf {
    model_dir().join(filename)
}

pub fn model_exists(filename: &str) -> bool {
    model_file_path(filename).is_file()
}

/// `model/` 内の `.gguf` ファイル名一覧（ソート済み）
pub fn list_model_gguf_files() -> Vec<String> {
    let dir = model_dir();
    let mut names = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return names;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_gguf = path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"));
        if !is_gguf {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            names.push(name.to_string());
        }
    }
    names.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));
    names
}

/// 開発用: 外部パス上書き
pub fn resolve_model_path(filename: &str) -> PathBuf {
    if let Ok(dev) = std::env::var("GLAUX_DEV_MODEL") {
        let p = PathBuf::from(&dev);
        if p.is_file() {
            return p;
        }
    }
    if let Ok(dir) = std::env::var("GLAUX_DEV_MODEL_DIR") {
        let p = PathBuf::from(dir).join(filename);
        if p.is_file() {
            return p;
        }
    }
    model_file_path(filename)
}

pub fn is_portable_layout() -> bool {
    app_root().join("model").is_dir() || app_root().join("models").is_dir()
}

pub fn bundle_readme_hint() -> &'static str {
    "model/ に GGUF を Glaux.exe と同じフォルダへ配置してください。ランタイムは EXE 内蔵です。"
}
