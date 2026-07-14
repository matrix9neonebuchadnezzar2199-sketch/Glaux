//! モデルファミリー別チャットプロンプト整形

use crate::config::PromptFormat;
use serde_json::Value;

/// ChatML 系のメッセージ終端トークン（`<|im_end|>`）
const IM_END: &str = concat!("<|", "im_end", "|>");

pub struct FormattedPrompt {
    pub prompt: String,
    pub stop: Vec<String>,
}

/// メッセージ配列を llama-server `/completion` 用プロンプトへ変換する。
pub fn format_messages(format: PromptFormat, messages: &[Value]) -> FormattedPrompt {
    let formatted = match format {
        PromptFormat::Gemma => gemma_prompt(messages),
        PromptFormat::Gemma2 => gemma2_prompt(messages),
        PromptFormat::ChatMl => chatml_prompt(messages),
        PromptFormat::Rakuten => rakuten_prompt(messages),
        PromptFormat::Qwen => qwen_prompt(messages),
        PromptFormat::Llama3 => llama3_prompt(messages),
    };
    // #region agent log
    crate::debug_agent_log::debug_session_log(
        "H1",
        "prompt.rs:format_messages",
        "formatted prompt",
        serde_json::json!({
            "format": format!("{format:?}"),
            "prompt_len": formatted.prompt.len(),
            "prompt_head": formatted.prompt.chars().take(240).collect::<String>(),
            "prompt_tail": formatted.prompt.chars().rev().take(120).collect::<String>().chars().rev().collect::<String>(),
            "stop": formatted.stop,
            "has_im_end_in_stop": formatted.stop.iter().any(|s| s.contains("im_end")),
            "assistant_suffix": formatted.prompt.chars().rev().take(40).collect::<String>().chars().rev().collect::<String>(),
        }),
    );
    // #endregion
    formatted
}

fn gemma_prompt(messages: &[Value]) -> FormattedPrompt {
    let mut prompt = String::new();
    for msg in messages {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }

        let turn_role = match role {
            "assistant" => "model",
            "system" => "system",
            _ => "user",
        };
        prompt.push_str("<|turn>");
        prompt.push_str(turn_role);
        prompt.push('\n');
        prompt.push_str(content);
        prompt.push_str("<turn|>\n");
    }
    prompt.push_str("<|turn>model\n");
    FormattedPrompt {
        prompt,
        stop: vec!["<turn|>".into()],
    }
}

/// Gemma 2 / JPN-IT（`<start_of_turn>` / `<end_of_turn>`）
fn gemma2_prompt(messages: &[Value]) -> FormattedPrompt {
    let mut prompt = String::new();
    for msg in messages {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }
        let turn_role = match role {
            "assistant" => "model",
            "system" => "user",
            _ => "user",
        };
        // Gemma 2 は system 専用ターンが弱いため、system は user ターンに載せる
        if role == "system" {
            prompt.push_str("<start_of_turn>user\n");
            prompt.push_str(content);
            prompt.push_str("<end_of_turn>\n");
            continue;
        }
        prompt.push_str("<start_of_turn>");
        prompt.push_str(turn_role);
        prompt.push('\n');
        prompt.push_str(content);
        prompt.push_str("<end_of_turn>\n");
    }
    prompt.push_str("<start_of_turn>model\n");
    FormattedPrompt {
        prompt,
        stop: vec!["<end_of_turn>".into()],
    }
}

/// Llama 3 / 3.2 Instruct
fn llama3_prompt(messages: &[Value]) -> FormattedPrompt {
    let header = |role: &str| -> String {
        format!("<|start_header_id|>{role}<|end_header_id|>\n\n")
    };
    let mut prompt = String::from("<|begin_of_text|>");
    for msg in messages {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }
        let turn_role = match role {
            "assistant" => "assistant",
            "system" => "system",
            _ => "user",
        };
        prompt.push_str(&header(turn_role));
        prompt.push_str(content);
        prompt.push_str("<|eot_id|>");
    }
    prompt.push_str(&header("assistant"));
    FormattedPrompt {
        prompt,
        stop: vec!["<|eot_id|>".into(), "<|end_of_text|>".into()],
    }
}

/// ChatML（MiniCPM5 等）
fn chatml_prompt(messages: &[Value]) -> FormattedPrompt {
    let mut prompt = String::new();
    for msg in messages {
        append_chatml_turn(&mut prompt, msg);
    }
    prompt.push_str("<|im_start|>assistant\n");
    FormattedPrompt {
        prompt,
        stop: vec![IM_END.into(), "</s>".into()],
    }
}

/// Qwen 2.5 Instruct 系
fn qwen_prompt(messages: &[Value]) -> FormattedPrompt {
    let mut prompt = String::new();
    for msg in messages {
        append_chatml_turn(&mut prompt, msg);
    }
    prompt.push_str("<|im_start|>assistant\n");
    FormattedPrompt {
        prompt,
        stop: vec![IM_END.into(), "<|endoftext|>".into(), "</s>".into()],
    }
}

fn append_chatml_turn(prompt: &mut String, msg: &Value) {
    let role = msg["role"].as_str().unwrap_or("user");
    let content = msg["content"].as_str().unwrap_or("").trim();
    if content.is_empty() {
        return;
    }
    prompt.push_str("<|im_start|>");
    prompt.push_str(role);
    prompt.push('\n');
    prompt.push_str(content);
    prompt.push_str("\n");
    prompt.push_str(IM_END);
    prompt.push('\n');
}

/// Rakuten AI 2.0 mini 系（USER / ASSISTANT プレフィックス）
fn rakuten_prompt(messages: &[Value]) -> FormattedPrompt {
    const DEFAULT_SYSTEM: &str = "A chat between a curious user and an artificial intelligence assistant. The assistant gives helpful, detailed, and polite answers to the user's questions. ";

    let mut system_message = DEFAULT_SYSTEM.to_string();
    let mut rest = messages;

    if let Some(first) = messages.first() {
        if first["role"].as_str() == Some("system") {
            if let Some(content) = first["content"].as_str() {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    system_message = format!("{trimmed} ");
                }
            }
            rest = &messages[1..];
        }
    }

    let mut prompt = system_message;
    for msg in rest {
        let role = msg["role"].as_str().unwrap_or("user");
        let content = msg["content"].as_str().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }
        match role {
            "user" => {
                prompt.push_str("USER: ");
                prompt.push_str(content);
            }
            "assistant" => {
                prompt.push_str(" ASSISTANT: ");
                prompt.push_str(content);
                prompt.push(' ');
            }
            _ => {}
        }
    }
    prompt.push_str(" ASSISTANT:");
    FormattedPrompt {
        prompt,
        stop: vec!["</s>".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn gemma_includes_generation_header() {
        let msgs = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "hello"}),
        ];
        let out = format_messages(PromptFormat::Gemma, &msgs);
        assert!(out.prompt.contains("<|turn>user\nhello<turn|>"));
        assert!(out.prompt.ends_with("<|turn>model\n"));
    }

    #[test]
    fn chatml_uses_im_start() {
        let msgs = vec![json!({"role": "user", "content": "こんにちは"})];
        let out = format_messages(PromptFormat::ChatMl, &msgs);
        assert!(out.prompt.contains("<|im_start|>user\nこんにちは\n"));
        assert!(out.prompt.contains(IM_END));
        assert!(out.prompt.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn rakuten_uses_user_assistant_prefix() {
        let msgs = vec![
            json!({"role": "system", "content": "日本語で答えて"}),
            json!({"role": "user", "content": "テスト"}),
        ];
        let out = format_messages(PromptFormat::Rakuten, &msgs);
        assert!(out.prompt.starts_with("日本語で答えて "));
        assert!(out.prompt.contains("USER: テスト"));
        assert!(out.prompt.ends_with(" ASSISTANT:"));
    }

    #[test]
    fn qwen_adds_endoftext_stop() {
        let out = format_messages(
            PromptFormat::Qwen,
            &[json!({"role": "user", "content": "hi"})],
        );
        assert!(out.stop.iter().any(|s| s == "<|endoftext|>"));
    }

    #[test]
    fn gemma2_uses_start_of_turn() {
        let msgs = vec![
            json!({"role": "system", "content": "日本語で"}),
            json!({"role": "user", "content": "こんにちは"}),
        ];
        let out = format_messages(PromptFormat::Gemma2, &msgs);
        assert!(out.prompt.contains("<start_of_turn>user\n日本語で<end_of_turn>\n"));
        assert!(out.prompt.contains("<start_of_turn>user\nこんにちは<end_of_turn>\n"));
        assert!(out.prompt.ends_with("<start_of_turn>model\n"));
        assert!(out.stop.iter().any(|s| s == "<end_of_turn>"));
    }

    #[test]
    fn llama3_uses_header_tokens() {
        let msgs = vec![
            json!({"role": "system", "content": "sys"}),
            json!({"role": "user", "content": "hi"}),
        ];
        let out = format_messages(PromptFormat::Llama3, &msgs);
        assert!(out.prompt.starts_with("<|begin_of_text|>"));
        assert!(out.prompt.contains("<|start_header_id|>system<|end_header_id|>\n\nsys<|eot_id|>"));
        assert!(out.prompt.contains("<|start_header_id|>user<|end_header_id|>\n\nhi<|eot_id|>"));
        assert!(out.prompt.ends_with("<|start_header_id|>assistant<|end_header_id|>\n\n"));
    }
}
