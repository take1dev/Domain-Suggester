/// WhoisFreaks API integration for domain availability and pricing.
///
/// Free tier: 500 lifetime credits (no credit card required).
/// Endpoint: GET https://api.whoisfreaks.com/v1.0/domain/availability

use super::{Availability, DomainChecker, DomainStatus, PriceInfo};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const AVAILABILITY_URL: &str = "https://api.whoisfreaks.com/v1.0/domain/availability";

// ---------------------------------------------------------------------------
// API response types
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct AvailabilityResponse {
    #[serde(default)]
    domain: String,
    #[serde(rename = "domainAvailability", default)]
    domain_availability: String,
    // Some responses include pricing
    #[serde(default)]
    price: Option<PriceData>,
}

#[derive(Deserialize, Debug)]
struct PriceData {
    #[serde(default)]
    registration: Option<f64>,
    #[serde(default)]
    renewal: Option<f64>,
    #[serde(default)]
    currency: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    status: Option<u16>,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

pub struct WhoisFreaksChecker {
    client: Client,
    api_key: String,
}

impl WhoisFreaksChecker {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl DomainChecker for WhoisFreaksChecker {
    async fn check(&self, domain: &str) -> Result<DomainStatus> {
        let response = self
            .client
            .get(AVAILABILITY_URL)
            .query(&[("domain", domain), ("apiKey", &self.api_key)])
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .context("WhoisFreaks request failed")?;

        let status_code = response.status();

        if !status_code.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            // Try to parse error message
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                let msg = err.message.unwrap_or_else(|| format!("HTTP {status_code}"));
                return Ok(DomainStatus::error(domain.to_string(), msg));
            }
            return Ok(DomainStatus::error(
                domain.to_string(),
                format!("HTTP {status_code}: {error_text}"),
            ));
        }

        let body = response.text().await.context("Read WhoisFreaks body")?;

        // The API may return a single object or an array
        let availability_result: Result<AvailabilityResponse, _> = serde_json::from_str(&body);

        match availability_result {
            Ok(data) => {
                let avail = match data.domain_availability.to_uppercase().as_str() {
                    "AVAILABLE" => Availability::Available,
                    "UNAVAILABLE" | "NOT AVAILABLE" | "REGISTERED" => Availability::Taken,
                    other => {
                        tracing::warn!("Unknown availability value: {other}");
                        Availability::Unknown
                    }
                };

                let price = data.price.map(|p| PriceInfo {
                    registration: p.registration,
                    renewal: p.renewal,
                    currency: p.currency.unwrap_or_else(|| "USD".into()),
                });

                Ok(DomainStatus {
                    domain: if data.domain.is_empty() {
                        domain.to_string()
                    } else {
                        data.domain
                    },
                    availability: avail,
                    price,
                })
            }
            Err(e) => {
                tracing::warn!("Failed to parse WhoisFreaks response for {domain}: {e}");
                Ok(DomainStatus::error(
                    domain.to_string(),
                    format!("Parse error: {e}"),
                ))
            }
        }
    }
}
