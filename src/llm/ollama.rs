/// Local Ollama LLM provider — zero authentication, fully offline.
///
/// Requires the user to have Ollama installed and running on localhost:11434.
/// Uses the OpenAI-compatible API endpoint.

use super::{LlmProvider, SuggestionRequest};
use crate::llm::prompt;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "llama3";

// ---------------------------------------------------------------------------
// API types (OpenAI-compatible)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<Choice>>,
    // Ollama native format fallback
    message: Option<ResponseMessage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

// For listing available models
#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize, Clone)]
pub struct ModelInfo {
    pub name: String,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(model: Option<String>, base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    /// Check if Ollama is running and reachable.
    pub async fn is_available(base_url: Option<&str>) -> bool {
        let url = base_url.unwrap_or(DEFAULT_BASE_URL);
        let client = Client::new();
        client
            .get(format!("{url}/api/tags"))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .is_ok()
    }

    /// List available models from the running Ollama instance.
    pub async fn list_models(base_url: Option<&str>) -> Result<Vec<String>> {
        let url = base_url.unwrap_or(DEFAULT_BASE_URL);
        let client = Client::new();
        let resp: TagsResponse = client
            .get(format!("{url}/api/tags"))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .context("Cannot connect to Ollama")?
            .json()
            .await
            .context("Failed to parse Ollama model list")?;

        Ok(resp.models.into_iter().map(|m| m.name).collect())
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_suggestions(&self, request: &SuggestionRequest) -> Result<Vec<String>> {
        let system_prompt = prompt::build_system_prompt();
        let user_prompt = prompt::build_user_prompt(request);

        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: system_prompt,
                },
                Message {
                    role: "user".into(),
                    content: user_prompt,
                },
            ],
            temperature: request.creativity.temperature(),
            stream: false,
        };

        // Try OpenAI-compatible endpoint first
        let url = format!("{}/v1/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await;

        let content = match response {
            Ok(resp) if resp.status().is_success() => {
                let chat: ChatResponse = resp.json().await.context("Parse Ollama response")?;
                chat.choices
                    .and_then(|c| c.into_iter().next().map(|ch| ch.message.content))
                    .or_else(|| chat.message.map(|m| m.content))
                    .unwrap_or_default()
            }
            _ => {
                // Fallback to Ollama native /api/chat endpoint
                tracing::info!("Falling back to Ollama native /api/chat endpoint");
                let native_url = format!("{}/api/chat", self.base_url);
                let resp = self
                    .client
                    .post(&native_url)
                    .json(&body)
                    .timeout(std::time::Duration::from_secs(120))
                    .send()
                    .await
                    .context("Failed to reach Ollama")?;

                if !resp.status().is_success() {
                    let err = resp.text().await.unwrap_or_default();
                    anyhow::bail!("Ollama error: {err}");
                }

                let chat: ChatResponse =
                    resp.json().await.context("Parse Ollama native response")?;
                chat.message
                    .map(|m| m.content)
                    .or_else(|| {
                        chat.choices
                            .and_then(|c| c.into_iter().next().map(|ch| ch.message.content))
                    })
                    .unwrap_or_default()
            }
        };

        tracing::debug!("Ollama raw response: {content}");

        prompt::parse_suggestions(&content)
    }

    fn name(&self) -> &str {
        "Ollama (Local)"
    }

    fn needs_credentials(&self) -> bool {
        false
    }
}
