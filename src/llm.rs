//! Minimal client for the OpenAI-compatible `/chat/completions` endpoint.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::message::Message;

/// Timeout applied to a single chat-completions request.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

/// Async client bound to one endpoint and one model.
#[derive(Debug, Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl LlmClient {
    /// Build a client. The base URL must not include the `/chat/completions`
    /// path; it is appended by [`LlmClient::complete`].
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            model: model.into(),
            api_key,
        })
    }

    /// Model identifier this client requests.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Endpoint base URL, without a trailing slash.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Whether requests are authenticated with a bearer token.
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Send the full transcript and the tool definitions to the model.
    pub async fn complete(
        &self,
        messages: &[Message],
        tools: Value,
        temperature: f32,
    ) -> Result<AssistantTurn> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut request = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": self.model,
                "messages": messages,
                "tools": tools,
                "tool_choice": "auto",
                "temperature": temperature
            }));

        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Unable to reach LLM endpoint: {url}"))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .context("Unable to read the LLM response body")?;

        if !status.is_success() {
            bail!("LLM API returned HTTP {}:\n{}", status, body);
        }

        let parsed: ChatResponse = serde_json::from_str(&body)
            .with_context(|| format!("Unable to parse LLM API response:\n{}", body))?;
        let message = parsed
            .choices
            .first()
            .ok_or_else(|| anyhow!("LLM API response did not contain any choices"))?
            .message
            .clone();

        Ok(AssistantTurn {
            content: message.content,
            tool_calls: message.tool_calls,
        })
    }
}

/// One assistant reply: final text, tool calls, or both.
///
/// Tool calls are kept as raw JSON so they can be replayed verbatim in the
/// next request without losing provider-specific fields.
#[derive(Debug, Clone, Default)]
pub struct AssistantTurn {
    pub content: Option<Value>,
    pub tool_calls: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct AssistantMessage {
    #[serde(default)]
    content: Option<Value>,
    #[serde(default)]
    tool_calls: Option<Value>,
}
