/// Background async worker that processes LLM and domain-check commands.
///
/// Communicates with the UI thread via mpsc channels.
/// Pipeline: Generate suggestions → Expand TLDs → Fan-out availability checks.

use crate::domain::{DomainChecker, DomainCheckerKind, DomainStatus};
use crate::domain::rdap::RdapChecker;
use crate::domain::whoisfreaks::WhoisFreaksChecker;
use crate::domain::dns::DnsChecker;
use crate::domain::tld;
use crate::llm::{LlmProvider, LlmProviderKind, SuggestionRequest};
use crate::llm::ollama::OllamaProvider;
use crate::llm::openrouter::OpenRouterProvider;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;

// ---------------------------------------------------------------------------
// Commands (UI → Worker)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum WorkerCommand {
    /// Generate domain suggestions from the LLM, then check availability.
    GenerateAndCheck {
        request: SuggestionRequest,
        tlds: Vec<String>,
        llm_kind: LlmProviderKind,
        openrouter_key: Option<String>,
        openrouter_model: Option<String>,
        ollama_model: Option<String>,
        checker_kind: DomainCheckerKind,
        whoisfreaks_key: Option<String>,
    },
    /// Check availability for manually entered domains.
    CheckDomains {
        domains: Vec<String>,
        checker_kind: DomainCheckerKind,
        whoisfreaks_key: Option<String>,
    },
    /// Fetch latest free OpenRouter models.
    FetchOpenRouterModels,
}

// ---------------------------------------------------------------------------
// Results (Worker → UI)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum WorkerResult {
    /// LLM generated these base name suggestions.
    Suggestions(Vec<String>),
    /// A single domain check result (streamed one at a time).
    DomainResult(DomainStatus),
    /// Progress update.
    Progress { checked: usize, total: usize },
    /// An error occurred.
    Error(String),
    /// All checks complete.
    Done,
    /// Latest Free Models
    OpenRouterModels(Vec<(String, String)>),
}

// ---------------------------------------------------------------------------
// Worker loop
// ---------------------------------------------------------------------------

/// Maximum concurrent domain checks.
const MAX_CONCURRENT_CHECKS: usize = 8;

/// Spawn the background worker. Returns the command sender.
pub fn spawn_worker(
    rt_handle: tokio::runtime::Handle,
    result_tx: mpsc::UnboundedSender<WorkerResult>,
    repaint_ctx: egui::Context,
) -> mpsc::UnboundedSender<WorkerCommand> {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<WorkerCommand>();

    rt_handle.spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WorkerCommand::GenerateAndCheck {
                    request,
                    tlds,
                    llm_kind,
                    openrouter_key,
                    openrouter_model,
                    ollama_model,
                    checker_kind,
                    whoisfreaks_key,
                } => {
                    // 1. Create LLM provider
                    let provider: Box<dyn LlmProvider> = match llm_kind {
                        LlmProviderKind::OpenRouter => {
                            let key = match openrouter_key {
                                Some(k) if !k.is_empty() => k,
                                _ => {
                                    let _ = result_tx.send(WorkerResult::Error(
                                        "OpenRouter API key is required. Get one free at openrouter.ai".into(),
                                    ));
                                    repaint_ctx.request_repaint();
                                    continue;
                                }
                            };
                            Box::new(OpenRouterProvider::new(key, openrouter_model))
                        }
                        LlmProviderKind::Ollama => {
                            Box::new(OllamaProvider::new(ollama_model, None))
                        }
                    };

                    // 2. Generate suggestions
                    tracing::info!("Generating suggestions via {}", provider.name());
                    match provider.generate_suggestions(&request).await {
                        Ok(suggestions) => {
                            let _ = result_tx.send(WorkerResult::Suggestions(suggestions.clone()));
                            repaint_ctx.request_repaint();

                            // 3. Expand with TLDs
                            let domains = tld::expand(&suggestions, &tlds);

                            // 4. Check availability
                            check_domains_inner(
                                domains,
                                checker_kind,
                                whoisfreaks_key,
                                &result_tx,
                                &repaint_ctx,
                            )
                            .await;
                        }
                        Err(e) => {
                            let _ = result_tx.send(WorkerResult::Error(format!(
                                "LLM generation failed: {e}"
                            )));
                            repaint_ctx.request_repaint();
                        }
                    }
                }
                WorkerCommand::CheckDomains {
                    domains,
                    checker_kind,
                    whoisfreaks_key,
                } => {
                    check_domains_inner(
                        domains,
                        checker_kind,
                        whoisfreaks_key,
                        &result_tx,
                        &repaint_ctx,
                    )
                    .await;
                }
                WorkerCommand::FetchOpenRouterModels => {
                    let tx = result_tx.clone();
                    let ctx = repaint_ctx.clone();
                    tokio::spawn(async move {
                        if let Ok(models) = crate::llm::openrouter::fetch_free_models().await {
                            let _ = tx.send(WorkerResult::OpenRouterModels(models));
                            ctx.request_repaint();
                        }
                    });
                }
            }
        }
    });

    cmd_tx
}

async fn create_checker(
    checker_kind: DomainCheckerKind,
    whoisfreaks_key: Option<String>,
) -> Result<Arc<dyn DomainChecker>, String> {
    match checker_kind {
        DomainCheckerKind::WhoisFreaks => {
            match whoisfreaks_key {
                Some(k) if !k.is_empty() => {
                    Ok(Arc::new(WhoisFreaksChecker::new(k)))
                }
                _ => {
                    // Fall back to RDAP when no WhoisFreaks key present
                    tracing::warn!("No WhoisFreaks key, falling back to RDAP");
                    RdapChecker::new()
                        .await
                        .map(|c| Arc::new(c) as Arc<dyn DomainChecker>)
                        .map_err(|e| format!("Failed to init RDAP checker: {e}"))
                }
            }
        }
        DomainCheckerKind::Rdap => {
            RdapChecker::new()
                .await
                .map(|c| Arc::new(c) as Arc<dyn DomainChecker>)
                .map_err(|e| format!("Failed to init RDAP checker: {e}"))
        }
        DomainCheckerKind::Dns => {
            DnsChecker::new()
                .map(|c| Arc::new(c) as Arc<dyn DomainChecker>)
                .map_err(|e| format!("Failed to init DNS checker: {e}"))
        }
    }
}

async fn check_domains_inner(
    domains: Vec<String>,
    checker_kind: DomainCheckerKind,
    whoisfreaks_key: Option<String>,
    result_tx: &mpsc::UnboundedSender<WorkerResult>,
    repaint_ctx: &egui::Context,
) {
    let total = domains.len();

    let checker = match create_checker(checker_kind, whoisfreaks_key).await {
        Ok(c) => c,
        Err(msg) => {
            let _ = result_tx.send(WorkerResult::Error(msg));
            repaint_ctx.request_repaint();
            return;
        }
    };

    // Fan-out checks with bounded concurrency
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CHECKS));
    let checked = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(total);

    for domain in domains {
        let sem = semaphore.clone();
        let checker = checker.clone();
        let tx = result_tx.clone();
        let ctx = repaint_ctx.clone();
        let count = checked.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await;
            let result = checker.check(&domain).await;
            let status = match result {
                Ok(s) => s,
                Err(e) => DomainStatus::error(domain, format!("{e}")),
            };

            let _ = tx.send(WorkerResult::DomainResult(status));
            let done = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let _ = tx.send(WorkerResult::Progress {
                checked: done,
                total,
            });
            ctx.request_repaint();
        });

        handles.push(handle);
    }

    // Wait for all checks to complete
    for h in handles {
        let _ = h.await;
    }

    let _ = result_tx.send(WorkerResult::Done);
    repaint_ctx.request_repaint();
}
