use super::{Availability, DomainChecker, DomainStatus};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct DohResponse {
    /// 0 = NOERROR, 3 = NXDOMAIN
    #[serde(rename = "Status")]
    status: u16,
}

pub struct DnsChecker {
    client: Client,
}

impl DnsChecker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }
}

#[async_trait]
impl DomainChecker for DnsChecker {
    async fn check(&self, domain: &str) -> Result<DomainStatus> {
        // Query DNS over HTTPS via Cloudflare (Type 2 = NS record)
        let url = format!("https://cloudflare-dns.com/dns-query?name={}&type=NS", domain);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/dns-json")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(doh) = resp.json::<DohResponse>().await {
                    if doh.status == 3 {
                        // NXDOMAIN => Domain does not exist (Available)
                        return Ok(DomainStatus {
                            domain: domain.to_string(),
                            availability: Availability::Available,
                            price: None,
                        });
                    } else if doh.status == 0 {
                        // NOERROR => Domain likely exists (Registered)
                        return Ok(DomainStatus {
                            domain: domain.to_string(),
                            availability: Availability::Taken,
                            price: None,
                        });
                    } else {
                        // SERVFAIL, REFUSED, etc.
                        return Ok(DomainStatus {
                            domain: domain.to_string(),
                            availability: Availability::Unknown,
                            price: None,
                        });
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!("DoH check returned HTTP {}", resp.status());
            }
            Err(e) => {
                tracing::warn!("DoH network error: {}", e);
            }
        }
        
        // Fallback on network or parsing failure
        Ok(DomainStatus {
            domain: domain.to_string(),
            availability: Availability::Unknown,
            price: None,
        })
    }
}
