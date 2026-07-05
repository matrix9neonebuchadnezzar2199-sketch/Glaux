//! モデル起動時のフェーズ表示用

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupPhase {
    CheckingFiles,
    ExtractingRuntime,
    CheckingMemory,
    SpawningServer,
    LoadingModel,
    Done,
}

impl StartupPhase {
    pub fn label_ja(self) -> &'static str {
        match self {
            Self::CheckingFiles => "ファイルを確認しています",
            Self::ExtractingRuntime => "ランタイムを展開しています",
            Self::CheckingMemory => "メモリを確認しています",
            Self::SpawningServer => "llama-server を起動しています",
            Self::LoadingModel => "モデルをメモリに読み込んでいます",
            Self::Done => "起動完了",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StartupProgress {
    pub phase: StartupPhase,
    /// 0.0〜1.0（全体進捗）
    pub fraction: f32,
    pub detail: String,
    pub elapsed_secs: f32,
}

impl StartupProgress {
    pub fn initial(started: Instant) -> Self {
        Self::at(
            StartupPhase::CheckingFiles,
            0.02,
            "モデルとランタイムの配置を確認中…",
            started,
        )
    }

    pub fn at(
        phase: StartupPhase,
        fraction: f32,
        detail: impl Into<String>,
        started: Instant,
    ) -> Self {
        Self {
            phase,
            fraction: fraction.clamp(0.0, 1.0),
            detail: detail.into(),
            elapsed_secs: started.elapsed().as_secs_f32(),
        }
    }
}

pub type ProgressCallback<'a> = &'a dyn Fn(StartupProgress);
