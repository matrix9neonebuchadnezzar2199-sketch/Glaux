//! ランタイム管理と llama-server 子プロセス

mod embedded;
mod extractor;
mod process;

pub use extractor::{resolve_runtime_with_progress, RuntimePaths};
pub use process::{check_memory_mb, LlamaServerHandle};

use crate::paths::{self, dev_artifacts_root};
use std::path::PathBuf;

/// llama-server が利用可能か（埋め込み・同梱・開発 artifacts）
pub fn server_available() -> bool {
    if std::env::var("GLAUX_DEV_SERVER")
        .ok()
        .map(PathBuf::from)
        .is_some_and(|p| p.is_file())
    {
        return true;
    }
    if paths::app_root()
        .join("runtime")
        .join("llama-server.exe")
        .is_file()
    {
        return true;
    }
    if embedded::RUNTIME_BUNDLE_EMBEDDED {
        return true;
    }
    dev_artifacts_root().join("llama-server.exe").is_file()
}

/// 埋め込みランタイム配布ビルドか
pub fn runtime_embedded() -> bool {
    embedded::RUNTIME_BUNDLE_EMBEDDED
}

pub fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "Glaux")
        .map(|d| d.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("GlauxData"))
}

pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

pub fn append_log(line: &str) {
    let dir = log_dir();
    let _ = std::fs::create_dir_all(&dir);
    let name = format!("glaux-{}.log", chrono::Local::now().format("%Y%m%d"));
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(name))
    {
        let _ = writeln!(f, "[{}] {line}", chrono::Local::now().format("%H:%M:%S"));
    }
}
