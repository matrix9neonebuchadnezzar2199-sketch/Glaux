//! モデル稼働状態

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRuntimeState {
    Stopped,
    Starting,
    Ready,
    Generating,
    Stopping,
    Error,
}

impl ModelRuntimeState {
    pub fn label_ja(self) -> &'static str {
        match self {
            Self::Stopped => "停止",
            Self::Starting => "起動中",
            Self::Ready => "起動完了",
            Self::Generating => "応答生成中",
            Self::Stopping => "停止中",
            Self::Error => "エラー",
        }
    }

    pub fn can_start(self) -> bool {
        matches!(self, Self::Stopped | Self::Error)
    }

    pub fn can_stop(self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Ready | Self::Generating | Self::Stopping
        )
    }

    pub fn can_chat(self) -> bool {
        matches!(self, Self::Ready | Self::Generating)
    }
}
