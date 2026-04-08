/// OpenRouter free-tier LLM provider.
///
/// Users sign up at openrouter.ai with just an email (no credit card).
/// Free models like `meta-llama/llama-3.3-70b-instruct:free` are available.

use super::{LlmProvider, SuggestionRequest};
use crate::llm::prompt;
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Default free model — the free router automatically picks the best available free model.
const DEFAULT_MODEL: &str = "openrouter/auto";

/// Known free models the user can select from.
pub const FREE_MODELS: &[(&str, &str)] = &[
    ("openrouter/auto", "Auto (Best Free)"),
    (
        "meta-llama/llama-3.3-70b-instruct:free",
        "Llama 3.3 70B (Free)",
    ),
    (
        "google/gemma-4-26b-a4b-it:free",
        "Gemma 4 26B (Free)",
    ),
    (
        "google/gemma-4-31b-it:free",
        "Gemma 4 31B (Free)",
    ),
    (
        "mistralai/mistral-7b-instruct:free",
        "Mistral 7B (Free)",
    ),

    (
        "qwen/qwen-2.5-72b-instruct:free",
        "Qwen 2.5 72B (Free)",
    ),
];

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelData>,
}

#[derive(Deserialize)]
struct ModelData {
    id: String,
    name: String,
}

pub async fn fetch_free_models() -> Result<Vec<(String, String)>> {
    let client = Client::new();
    let res = client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await
        .context("Failed to fetch models")?;
        
    let parse: ModelsResponse = res.json().await.context("Failed to parse models JSON")?;
    
    let mut models = vec![("openrouter/auto".to_string(), "Auto (Best Free)".to_string())];
    
    for m in parse.data {
        if m.id.ends_with(":free") && !m.id.contains("google/gemma-2-9b-it:free") {
            models.push((m.id.clone(), format!("{} (Free)", m.name)));
        }
    }
    
    Ok(models)
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
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
            max_tokens: 1024,
        };

        let response = self
            .client
            .post(BASE_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://domain-suggest.app")
            .header("X-Title", "Domain Suggest & Checker")
            .json(&body)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            anyhow::bail!("OpenRouter API error ({status}): {error_text}");
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse OpenRouter response JSON")?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        tracing::debug!("OpenRouter raw response: {content}");

        prompt::parse_suggestions(&content)
    }

    fn name(&self) -> &str {
        "OpenRouter (Free Tier)"
    }

    fn needs_credentials(&self) -> bool {
        true
    }
}
