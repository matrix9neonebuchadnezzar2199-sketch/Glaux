//! 会話コンテキスト（メモリのみ）

use crate::config::CONTEXT_LENGTH;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

pub struct ConversationContext {
    pub messages: Vec<ChatMessage>,
    pub system_prompt: String,
}

impl ConversationContext {
    pub fn new(system_prompt: impl Into<String>) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: system_prompt.into(),
        }
    }

    pub fn total_chars(&self) -> usize {
        self.system_prompt.chars().count()
            + self
                .messages
                .iter()
                .map(|m| m.content.chars().count())
                .sum::<usize>()
    }

    /// 日本語混在を想定した軽量概算
    pub fn estimated_tokens(&self) -> u32 {
        ((self.total_chars() as f32) / 2.0).ceil() as u32
    }

    pub fn usage_ratio(&self) -> f32 {
        self.estimated_tokens() as f32 / CONTEXT_LENGTH as f32
    }

    pub fn usage_level(&self) -> ContextUsageLevel {
        let r = self.usage_ratio();
        if r >= 1.0 {
            ContextUsageLevel::Blocked
        } else if r >= 0.9 {
            ContextUsageLevel::Warning
        } else if r >= 0.7 {
            ContextUsageLevel::Caution
        } else {
            ContextUsageLevel::Normal
        }
    }

    pub fn push_user(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "user".into(),
            content,
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
        });
    }

    pub fn push_assistant(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: "assistant".into(),
            content,
            timestamp: chrono::Local::now().format("%H:%M").to_string(),
        });
    }

    pub fn update_last_assistant(&mut self, content: &str) {
        if let Some(last) = self
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.role == "assistant")
        {
            last.content = content.to_string();
        } else {
            self.push_assistant(content.to_string());
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn to_api_messages(&self) -> Vec<serde_json::Value> {
        let mut out = vec![serde_json::json!({
            "role": "system",
            "content": self.system_prompt,
        })];
        for m in &self.messages {
            out.push(serde_json::json!({
                "role": m.role,
                "content": m.content,
            }));
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextUsageLevel {
    Normal,
    Caution,
    Warning,
    Blocked,
}
