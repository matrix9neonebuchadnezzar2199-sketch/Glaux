//! llama.cpp API クライアント

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub temperature: f32,
    pub top_p: f32,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    content: Option<String>,
    stop: Option<bool>,
    choices: Option<Vec<StreamChoice>>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

pub enum StreamEvent {
    Token(String),
    Done,
    Error(String),
}

#[derive(Debug, Serialize)]
struct CompletionRequest {
    prompt: String,
    temperature: f32,
    top_p: f32,
    stream: bool,
    n_predict: u32,
    stop: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
}

fn gemma_prompt(messages: &[serde_json::Value]) -> String {
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
    prompt
}

fn completion_request(req: ChatRequest) -> CompletionRequest {
    CompletionRequest {
        prompt: gemma_prompt(&req.messages),
        temperature: req.temperature,
        top_p: req.top_p,
        stream: req.stream,
        n_predict: 1024,
        stop: vec!["<turn|>".into()],
        top_k: req.top_k,
    }
}

pub fn chat_completion_stream(
    base_url: &str,
    req: ChatRequest,
    tx: Sender<StreamEvent>,
) -> Result<()> {
    let url = format!("{base_url}/completion");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    let completion = completion_request(req);

    let resp = client
        .post(&url)
        .json(&completion)
        .send()
        .with_context(|| format!("POST {url}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        let _ = tx.send(StreamEvent::Error(format!("HTTP {status}: {body}")));
        return Ok(());
    }

    let reader = BufReader::new(resp);
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                let _ = tx.send(StreamEvent::Done);
                break;
            }
            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                if let Some(content) = chunk.content {
                    if !content.is_empty() {
                        let _ = tx.send(StreamEvent::Token(content));
                    }
                }
                if let Some(choices) = chunk.choices {
                    for choice in choices {
                        if let Some(content) = choice.delta.content {
                            if !content.is_empty() {
                                let _ = tx.send(StreamEvent::Token(content));
                            }
                        }
                    }
                }
                if chunk.stop.unwrap_or(false) {
                    let _ = tx.send(StreamEvent::Done);
                    break;
                }
            }
        }
    }
    let _ = tx.send(StreamEvent::Done);
    Ok(())
}

pub fn chat_completion_blocking(base_url: &str, req: ChatRequest) -> Result<String> {
    let url = format!("{base_url}/completion");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    let mut completion = completion_request(req);
    completion.stream = false;
    let resp: serde_json::Value = client.post(&url).json(&completion).send()?.json()?;
    let content = resp["content"].as_str().unwrap_or("").trim().to_string();
    Ok(content)
}
