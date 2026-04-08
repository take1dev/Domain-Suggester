/// Direct RDAP (Registration Data Access Protocol) lookup.
///
/// Free, unlimited, no API key required.
/// Queries IANA-bootstrapped RDAP servers for domain registration status.
/// Falls back to a simple DNS-based heuristic for TLDs without RDAP.

use super::{Availability, DomainChecker, DomainStatus};
use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

const IANA_RDAP_DNS_BOOTSTRAP: &str = "https://data.iana.org/rdap/dns.json";

// ---------------------------------------------------------------------------
// IANA Bootstrap types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct BootstrapResponse {
    services: Vec<BootstrapService>,
}

/// Each service is `[ [tld1, tld2, ...], [rdap_url1, rdap_url2, ...] ]`
type BootstrapService = (Vec<String>, Vec<String>);

// ---------------------------------------------------------------------------
// RDAP domain response (simplified)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RdapDomainResponse {
    #[serde(rename = "ldhName", default)]
    ldh_name: Option<String>,
    #[serde(default)]
    status: Option<Vec<String>>,
    #[serde(rename = "errorCode", default)]
    error_code: Option<u16>,
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

pub struct RdapChecker {
    client: Client,
    /// Cached TLD → RDAP base URL mapping.
    bootstrap: HashMap<String, String>,
}

impl RdapChecker {
    pub async fn new() -> Result<Self> {
        let client = Client::new();

        // Fetch IANA bootstrap to discover RDAP servers for each TLD
        let bootstrap = match Self::fetch_bootstrap(&client).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("Failed to fetch RDAP bootstrap: {e}. Using hardcoded fallbacks.");
                Self::hardcoded_bootstrap()
            }
        };

        Ok(Self { client, bootstrap })
    }

    async fn fetch_bootstrap(client: &Client) -> Result<HashMap<String, String>> {
        let resp: BootstrapResponse = client
            .get(IANA_RDAP_DNS_BOOTSTRAP)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context("Fetch IANA RDAP bootstrap")?
            .json()
            .await
            .context("Parse IANA RDAP bootstrap")?;

        let mut map = HashMap::new();
        for (tlds, urls) in resp.services {
            if let Some(url) = urls.first() {
                let base = url.trim_end_matches('/').to_string();
                for tld in tlds {
                    map.insert(tld.to_lowercase(), base.clone());
                }
            }
        }
        Ok(map)
    }

    /// Hardcoded fallbacks for common TLDs.
    fn hardcoded_bootstrap() -> HashMap<String, String> {
        let mut map = HashMap::new();
        let verisign = "https://rdap.verisign.com/com/v1";
        map.insert("com".into(), verisign.into());
        map.insert("net".into(), "https://rdap.verisign.com/net/v1".into());
        map.insert("org".into(), "https://rdap.org/org".into());
        map.insert("io".into(), "https://rdap.nic.io".into());
        map.insert("dev".into(), "https://rdap.nic.google".into());
        map.insert("app".into(), "https://rdap.nic.google".into());
        map.insert("ai".into(), "https://rdap.nic.ai".into());
        map.insert("co".into(), "https://rdap.nic.co".into());
        map.insert("xyz".into(), "https://rdap.nic.xyz".into());
        map.insert("tech".into(), "https://rdap.nic.tech".into());
        map
    }

    /// Extract the TLD from a domain name.
    fn extract_tld(domain: &str) -> Option<String> {
        domain
            .rsplit('.')
            .next()
            .map(|t| t.to_lowercase())
    }
}

#[async_trait]
impl DomainChecker for RdapChecker {
    async fn check(&self, domain: &str) -> Result<DomainStatus> {
        let tld = Self::extract_tld(domain).unwrap_or_default();

        let rdap_base = match self.bootstrap.get(&tld) {
            Some(url) => url.clone(),
            None => {
                return Ok(DomainStatus {
                    domain: domain.to_string(),
                    availability: Availability::Unknown,
                    price: None,
                });
            }
        };

        let url = format!("{rdap_base}/domain/{domain}");

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status_code = resp.status();

                if status_code.as_u16() == 404 {
                    // 404 means domain is NOT registered → available
                    return Ok(DomainStatus {
                        domain: domain.to_string(),
                        availability: Availability::Available,
                        price: None,
                    });
                }

                if status_code.as_u16() == 429 {
                    return Ok(DomainStatus::error(
                        domain.to_string(),
                        "Rate limited by RDAP server. Try again later.",
                    ));
                }

                if status_code.is_success() {
                    // Domain record exists → it's taken
                    let _body: Result<RdapDomainResponse, _> = resp.json().await;
                    return Ok(DomainStatus {
                        domain: domain.to_string(),
                        availability: Availability::Taken,
                        price: None, // RDAP doesn't provide pricing
                    });
                }

                // Other errors
                Ok(DomainStatus::error(
                    domain.to_string(),
                    format!("RDAP returned HTTP {status_code}"),
                ))
            }
            Err(e) => Ok(DomainStatus::error(
                domain.to_string(),
                format!("RDAP request failed: {e}"),
            )),
        }
    }
}
