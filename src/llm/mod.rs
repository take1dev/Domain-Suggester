/// LLM provider abstraction for domain name suggestion generation.
///
/// Supports multiple backends: OpenRouter (free-tier), local Ollama, and custom endpoints.

pub mod openrouter;
pub mod ollama;
pub mod prompt;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Creativity mode controls the LLM temperature parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreativityMode {
    /// Temperature ~0.2 — deterministic, safe suggestions
    Conservative,
    /// Temperature ~0.6 — more creative, diverse suggestions
    Creative,
}

impl Default for CreativityMode {
    fn default() -> Self {
        Self::Creative
    }
}

impl CreativityMode {
    pub fn temperature(self) -> f32 {
        match self {
            Self::Conservative => 0.2,
            Self::Creative => 0.6,
        }
    }
}

/// Brand personality archetypes that guide the LLM's naming style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrandPersonality {
    Minimalist,
    Bold,
    Traditional,
    Playful,
    Luxurious,
    Technical,
    Friendly,
}

impl BrandPersonality {
    pub const ALL: &[Self] = &[
        Self::Minimalist,
        Self::Bold,
        Self::Traditional,
        Self::Playful,
        Self::Luxurious,
        Self::Technical,
        Self::Friendly,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Minimalist => "Minimalist",
            Self::Bold => "Bold",
            Self::Traditional => "Traditional",
            Self::Playful => "Playful",
            Self::Luxurious => "Luxurious",
            Self::Technical => "Technical",
            Self::Friendly => "Friendly",
        }
    }
}

impl Default for BrandPersonality {
    fn default() -> Self {
        Self::Bold
    }
}

/// A request to generate domain name suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionRequest {
    /// Comma-separated keywords describing the brand or product.
    pub keywords: String,
    /// Industry or niche (e.g. "Fintech", "Health & Wellness", "SaaS").
    pub industry: String,
    /// Desired brand personality.
    pub personality: BrandPersonality,
    /// Maximum character length for suggested names (excluding TLD).
    pub max_length: usize,
    /// Number of suggestions to generate.
    pub count: usize,
    /// Creativity mode (conservative vs creative).
    pub creativity: CreativityMode,
}

impl Default for SuggestionRequest {
    fn default() -> Self {
        Self {
            keywords: String::new(),
            industry: String::new(),
            personality: BrandPersonality::default(),
            max_length: 14,
            count: 15,
            creativity: CreativityMode::default(),
        }
    }
}

/// Which LLM provider backend to use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProviderKind {
    OpenRouter,
    Ollama,
}

impl LlmProviderKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::OpenRouter => "OpenRouter (Free Tier)",
            Self::Ollama => "Ollama (Local)",
        }
    }
}

impl Default for LlmProviderKind {
    fn default() -> Self {
        Self::OpenRouter
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Abstraction over LLM backends.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a list of domain name suggestions (base names without TLDs).
    async fn generate_suggestions(&self, request: &SuggestionRequest) -> Result<Vec<String>>;

    /// Human-readable name of this provider.
    fn name(&self) -> &str;

    /// Whether this provider requires an API key or credentials.
    fn needs_credentials(&self) -> bool;
}
