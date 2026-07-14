//! llama.cpp API クライアント

use crate::api::prompt;
use crate::config::{api_chat_template_kwargs, PromptFormat};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::Sender;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub prompt_format: PromptFormat,
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
    reasoning_content: Option<String>,
}

fn delta_token(delta: &StreamDelta) -> Option<String> {
    if let Some(content) = delta.content.as_ref() {
        if !content.is_empty() {
            return Some(content.clone());
        }
    }
    delta
        .reasoning_content
        .as_ref()
        .filter(|s| !s.is_empty())
        .cloned()
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

#[derive(Debug, Serialize)]
struct ChatCompletionsRequest<'a> {
    model: &'a str,
    messages: &'a [serde_json::Value],
    temperature: f32,
    top_p: f32,
    stream: bool,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<serde_json::Value>,
}

fn completion_request(req: &ChatRequest) -> CompletionRequest {
    let formatted = prompt::format_messages(req.prompt_format, &req.messages);
    // #region agent log
    crate::debug_agent_log::debug_session_log(
        "H2",
        "chat.rs:completion_request",
        "completion params",
        serde_json::json!({
            "runId": "post-fix",
            "api": "completion",
            "model": req.model,
            "prompt_format": format!("{:?}", req.prompt_format),
            "temperature": req.temperature,
            "top_p": req.top_p,
            "top_k": req.top_k,
            "stop": formatted.stop,
            "prompt_len": formatted.prompt.len(),
        }),
    );
    // #endregion
    CompletionRequest {
        prompt: formatted.prompt,
        temperature: req.temperature,
        top_p: req.top_p,
        stream: req.stream,
        n_predict: 1024,
        stop: formatted.stop,
        top_k: req.top_k,
    }
}

fn chat_completions_body(req: &ChatRequest) -> ChatCompletionsRequest<'_> {
    let chat_template_kwargs = api_chat_template_kwargs(req.prompt_format, &req.model);
    // #region agent log
    crate::debug_agent_log::debug_session_log(
        "H2",
        "chat.rs:chat_completions_body",
        "chat completions params",
        serde_json::json!({
            "runId": "post-fix-v3",
            "api": "v1/chat/completions",
            "model": req.model,
            "prompt_format": format!("{:?}", req.prompt_format),
            "temperature": req.temperature,
            "top_p": req.top_p,
            "top_k": req.top_k,
            "chat_template_kwargs": chat_template_kwargs,
            "message_count": req.messages.len(),
        }),
    );
    // #endregion
    ChatCompletionsRequest {
        model: &req.model,
        messages: &req.messages,
        temperature: req.temperature,
        top_p: req.top_p,
        stream: req.stream,
        max_tokens: 1024,
        top_k: req.top_k,
        chat_template_kwargs,
    }
}

fn log_stream_result(accumulated: &str, tag: &str) {
    // #region agent log
    crate::debug_agent_log::debug_session_log(
        "H4",
        "chat.rs:stream_result",
        tag,
        serde_json::json!({
            "runId": "post-fix",
            "content_len": accumulated.len(),
            "content_head": accumulated.chars().take(300).collect::<String>(),
            "has_think_open": accumulated.contains("think>"),
            "has_think_close": accumulated.contains("/think>"),
            "parrots_system": accumulated.contains("前置き・自己紹介・役割の宣言は不要です"),
            "only_whitespace": accumulated.trim().is_empty(),
        }),
    );
    // #endregion
}

fn process_stream_lines<R: BufRead>(reader: R, tx: &Sender<StreamEvent>) -> Result<()> {
    let mut accumulated = String::new();
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                log_stream_result(&accumulated, "stream finished");
                let _ = tx.send(StreamEvent::Done);
                break;
            }
            if let Ok(chunk) = serde_json::from_str::<StreamChunk>(data) {
                if let Some(content) = chunk.content {
                    if !content.is_empty() {
                        accumulated.push_str(&content);
                        let _ = tx.send(StreamEvent::Token(content));
                    }
                }
                if let Some(choices) = chunk.choices {
                    for choice in choices {
                        if let Some(content) = delta_token(&choice.delta) {
                            accumulated.push_str(&content);
                            let _ = tx.send(StreamEvent::Token(content));
                        }
                    }
                }
                if chunk.stop.unwrap_or(false) {
                    log_stream_result(&accumulated, "stream stopped");
                    let _ = tx.send(StreamEvent::Done);
                    break;
                }
            }
        }
    }
    log_stream_result(&accumulated, "stream loop ended");
    Ok(())
}

pub fn chat_completion_stream(
    base_url: &str,
    req: ChatRequest,
    tx: Sender<StreamEvent>,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    if req.prompt_format.uses_jinja_chat_api() {
        let url = format!("{base_url}/v1/chat/completions");
        let body = chat_completions_body(&req);
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .with_context(|| format!("POST {url}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            let _ = tx.send(StreamEvent::Error(format!("HTTP {status}: {body}")));
            return Ok(());
        }
        let reader = BufReader::new(resp);
        process_stream_lines(reader, &tx)?;
    } else {
        let url = format!("{base_url}/completion");
        let completion = completion_request(&req);
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
        process_stream_lines(reader, &tx)?;
    }

    let _ = tx.send(StreamEvent::Done);
    Ok(())
}

pub fn chat_completion_blocking(base_url: &str, req: ChatRequest) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    if req.prompt_format.uses_jinja_chat_api() {
        let url = format!("{base_url}/v1/chat/completions");
        let mut body = chat_completions_body(&req);
        body.stream = false;
        let resp: serde_json::Value = client.post(&url).json(&body).send()?.json()?;
        let msg = &resp["choices"][0]["message"];
        let content = msg["content"].as_str().unwrap_or("").trim();
        if !content.is_empty() {
            return Ok(content.to_string());
        }
        let reasoning = msg["reasoning_content"].as_str().unwrap_or("").trim();
        return Ok(reasoning.to_string());
    }

    let url = format!("{base_url}/completion");
    let mut completion = completion_request(&req);
    completion.stream = false;
    let resp: serde_json::Value = client.post(&url).json(&completion).send()?.json()?;
    let content = resp["content"].as_str().unwrap_or("").trim().to_string();
    Ok(content)
}
