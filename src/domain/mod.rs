/// Domain availability checking and pricing retrieval.
///
/// Supports WhoisFreaks (free tier, 500 credits) as primary
/// and direct RDAP as a free, unlimited fallback.

pub mod whoisfreaks;
pub mod rdap;
pub mod dns;
pub mod tld;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Availability status for a single domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Availability {
    /// Domain is available for registration.
    Available,
    /// Domain is already registered.
    Taken,
    /// Check is still in progress.
    Pending,
    /// Check failed with an error message.
    Error(String),
    /// Status could not be determined.
    Unknown,
}

impl Availability {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Available => "✅",
            Self::Taken => "❌",
            Self::Pending => "⏳",
            Self::Error(_) => "⚠️",
            Self::Unknown => "❓",
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Price information for a domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriceInfo {
    /// First-year registration price in USD.
    pub registration: Option<f64>,
    /// Annual renewal price in USD.
    pub renewal: Option<f64>,
    /// Currency code (usually "USD").
    pub currency: String,
}

/// Complete status for a single domain check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStatus {
    /// Full domain name including TLD (e.g. "nexora.com").
    pub domain: String,
    /// Availability status.
    pub availability: Availability,
    /// Pricing information (if retrieved).
    pub price: Option<PriceInfo>,
}

impl DomainStatus {
    pub fn pending(domain: String) -> Self {
        Self {
            domain,
            availability: Availability::Pending,
            price: None,
        }
    }

    pub fn error(domain: String, msg: impl Into<String>) -> Self {
        Self {
            domain,
            availability: Availability::Error(msg.into()),
            price: None,
        }
    }
}

/// Which domain checking backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainCheckerKind {
    WhoisFreaks,
    Rdap,
    Dns,
}

impl DomainCheckerKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::WhoisFreaks => "WhoisFreaks (Free Tier)",
            Self::Rdap => "RDAP (Direct, Free)",
            Self::Dns => "DNS (Fast & Unlimited)",
        }
    }
}

impl Default for DomainCheckerKind {
    fn default() -> Self {
        Self::WhoisFreaks
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Abstraction for domain availability checkers.
#[async_trait]
pub trait DomainChecker: Send + Sync {
    /// Check a single domain's availability and optionally retrieve pricing.
    async fn check(&self, domain: &str) -> Result<DomainStatus>;
}
