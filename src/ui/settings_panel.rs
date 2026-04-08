/// Settings panel — LLM provider config, API keys, TLD selection, domain checker config.

use crate::credentials;
use crate::domain::DomainCheckerKind;
use crate::domain::tld::DEFAULT_TLDS;
use crate::llm::LlmProviderKind;
use crate::llm::openrouter::FREE_MODELS;
use crate::ui::theme::{self, colors};
use eframe::egui;
use serde::{Deserialize, Serialize};

/// Preferred domain registrar for outgoing links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredRegistrar {
    Namecheap,
    GoDaddy,
    Porkbun,
    Cloudflare,
}

impl PreferredRegistrar {
    pub const ALL: &[Self] = &[
        Self::Namecheap,
        Self::GoDaddy,
        Self::Porkbun,
        Self::Cloudflare,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Namecheap => "Namecheap",
            Self::GoDaddy => "GoDaddy",
            Self::Porkbun => "Porkbun",
            Self::Cloudflare => "Cloudflare",
        }
    }

    pub fn purchase_url(self, domain: &str) -> String {
        match self {
            Self::Namecheap => format!("https://www.namecheap.com/domains/registration/results/?domain={}", domain),
            Self::GoDaddy => format!("https://www.godaddy.com/domainsearch/find?checkAvail=1&domainToGo={}", domain),
            Self::Porkbun => format!("https://porkbun.com/checkout/search?q={}", domain),
            Self::Cloudflare => format!("https://dash.cloudflare.com/?to=/:account/domains/register/?searchTerm={}", domain),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SavedSettings {
    llm_kind: Option<LlmProviderKind>,
    checker_kind: Option<DomainCheckerKind>,
    active_tlds: Option<Vec<(String, bool)>>,
    custom_tld: Option<String>,
    custom_tld_enabled: Option<bool>,
    preferred_registrar: Option<PreferredRegistrar>,
}

/// Persistent settings state.
#[derive(Debug, Clone)]
pub struct SettingsState {
    pub llm_kind: LlmProviderKind,
    pub openrouter_key: String,
    pub openrouter_key_visible: bool,
    pub openrouter_model_idx: usize,
    pub openrouter_models: Vec<(String, String)>,
    pub ollama_model: String,
    pub ollama_available: bool,
    pub ollama_models: Vec<String>,

    // Domain checker
    pub checker_kind: DomainCheckerKind,
    pub whoisfreaks_key: String,
    pub whoisfreaks_key_visible: bool,

    // TLDs
    pub active_tlds: Vec<(String, bool)>,
    pub custom_tld: String,
    pub custom_tld_enabled: bool,

    // Registrar links
    pub preferred_registrar: PreferredRegistrar,

    // UI state
    pub status_message: Option<String>,
    
    // Serialization cache
    pub last_saved_json: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        let active_tlds = DEFAULT_TLDS
            .iter()
            .map(|t| (t.to_string(), true))
            .collect();

        // Try to load saved credentials
        let openrouter_key =
            credentials::load_credential(credentials::KEY_OPENROUTER).unwrap_or_default();
        let whoisfreaks_key =
            credentials::load_credential(credentials::KEY_WHOISFREAKS).unwrap_or_default();

        Self {
            llm_kind: LlmProviderKind::OpenRouter,
            openrouter_key,
            openrouter_key_visible: false,
            openrouter_model_idx: 0,
            openrouter_models: FREE_MODELS.iter().map(|(id, name)| (id.to_string(), name.to_string())).collect(),
            ollama_model: "llama3".into(),
            ollama_available: false,
            ollama_models: Vec::new(),
            checker_kind: DomainCheckerKind::WhoisFreaks,
            whoisfreaks_key,
            whoisfreaks_key_visible: false,
            active_tlds,
            custom_tld: String::new(),
            custom_tld_enabled: false,
            preferred_registrar: PreferredRegistrar::Namecheap,
            status_message: None,
            last_saved_json: String::new(),
        }
    }
}

impl SettingsState {
    pub fn load_or_default() -> Self {
        let mut state = Self::default();
        if let Ok(json) = std::fs::read_to_string("settings.json") {
            if let Ok(saved) = serde_json::from_str::<SavedSettings>(&json) {
                if let Some(v) = saved.llm_kind { state.llm_kind = v; }
                if let Some(v) = saved.checker_kind { state.checker_kind = v; }
                if let Some(v) = saved.active_tlds { state.active_tlds = v; }
                if let Some(v) = saved.custom_tld { state.custom_tld = v; }
                if let Some(v) = saved.custom_tld_enabled { state.custom_tld_enabled = v; }
                if let Some(v) = saved.preferred_registrar { state.preferred_registrar = v; }
            }
        }
        
        let saved = SavedSettings {
            llm_kind: Some(state.llm_kind.clone()),
            checker_kind: Some(state.checker_kind),
            active_tlds: Some(state.active_tlds.clone()),
            custom_tld: Some(state.custom_tld.clone()),
            custom_tld_enabled: Some(state.custom_tld_enabled),
            preferred_registrar: Some(state.preferred_registrar),
        };
        state.last_saved_json = serde_json::to_string(&saved).unwrap_or_default();
        
        state
    }

    pub fn save_if_changed(&mut self) -> bool {
        let saved = SavedSettings {
            llm_kind: Some(self.llm_kind.clone()),
            checker_kind: Some(self.checker_kind),
            active_tlds: Some(self.active_tlds.clone()),
            custom_tld: Some(self.custom_tld.clone()),
            custom_tld_enabled: Some(self.custom_tld_enabled),
            preferred_registrar: Some(self.preferred_registrar),
        };
        let json = serde_json::to_string(&saved).unwrap_or_default();
        if self.last_saved_json != json {
            let _ = std::fs::write("settings.json", &json);
            self.last_saved_json = json;
            true
        } else {
            false
        }
    }

    /// Get the list of enabled TLD strings.
    pub fn enabled_tlds(&self) -> Vec<String> {
        let mut tlds: Vec<String> = self.active_tlds
            .iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(tld, _)| tld.clone())
            .collect();
            
        if self.custom_tld_enabled && !self.custom_tld.trim().is_empty() {
            let custom = self.custom_tld.trim();
            let clean = if custom.starts_with('.') {
                custom.to_string()
            } else {
                format!(".{}", custom)
            };
            if !tlds.contains(&clean) {
                tlds.push(clean);
            }
        }
        
        tlds
    }

    /// Get the selected OpenRouter model ID.
    pub fn selected_openrouter_model(&self) -> String {
        self.openrouter_models
            .get(self.openrouter_model_idx)
            .map(|(id, _)| id.to_string())
            .unwrap_or_else(|| "openrouter/auto".to_string())
    }
}

/// Render the settings panel.
/// Returns true if the user wants to fetch new models.
pub fn render(ui: &mut egui::Ui, state: &mut SettingsState) -> bool {
    let mut fetch_requested = false;

    egui::Frame::new()
        .fill(colors::BG_PANEL)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui: &mut egui::Ui| {
            theme::section_header(ui, "⚙️  Settings");

            ui.add_space(4.0);

            // ---- LLM Provider ----
            ui.label(
                egui::RichText::new("LLM Provider")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.selectable_value(
                    &mut state.llm_kind,
                    LlmProviderKind::OpenRouter,
                    egui::RichText::new("🌐 OpenRouter").size(13.0),
                );
                ui.selectable_value(
                    &mut state.llm_kind,
                    LlmProviderKind::Ollama,
                    egui::RichText::new("🖥 Ollama").size(13.0),
                );
            });

            ui.add_space(6.0);

            match state.llm_kind {
                LlmProviderKind::OpenRouter => {
                    if render_openrouter_settings(ui, state) {
                        fetch_requested = true;
                    }
                }
                LlmProviderKind::Ollama => {
                    render_ollama_settings(ui, state);
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // ---- Domain Checker ----
            ui.label(
                egui::RichText::new("Domain Checker")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.selectable_value(
                    &mut state.checker_kind,
                    DomainCheckerKind::WhoisFreaks,
                    egui::RichText::new("🔍 WhoisFreaks").size(13.0),
                );
                ui.selectable_value(
                    &mut state.checker_kind,
                    DomainCheckerKind::Rdap,
                    egui::RichText::new("🌍 RDAP (Free)").size(13.0),
                );
                ui.selectable_value(
                    &mut state.checker_kind,
                    DomainCheckerKind::Dns,
                    egui::RichText::new("⚡ DNS (Unlimited)").size(13.0),
                );
            });

            if state.checker_kind == DomainCheckerKind::WhoisFreaks {
                ui.add_space(4.0);
                render_key_input(
                    ui,
                    "WhoisFreaks API Key",
                    &mut state.whoisfreaks_key,
                    &mut state.whoisfreaks_key_visible,
                    credentials::KEY_WHOISFREAKS,
                    &mut state.status_message,
                );
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // ---- Outgoing Links ----
            ui.label(
                egui::RichText::new("Buy Links")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label(
                    egui::RichText::new("Registrar")
                        .color(colors::TEXT_SECONDARY)
                        .size(13.0),
                );
                egui::ComboBox::from_id_salt("preferred_registrar")
                    .selected_text(state.preferred_registrar.label())
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        for &r in PreferredRegistrar::ALL {
                            ui.selectable_value(&mut state.preferred_registrar, r, r.label());
                        }
                    });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // ---- TLDs ----
            ui.label(
                egui::RichText::new("TLDs to Check")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            ui.add_space(4.0);

            // Show TLD chips in a horizontal wrap
            ui.horizontal_wrapped(|ui: &mut egui::Ui| {
                for (tld, enabled) in state.active_tlds.iter_mut() {
                    let color = if *enabled {
                        colors::ACCENT_BLUE
                    } else {
                        colors::BG_CARD
                    };
                    let text_color = if *enabled {
                        colors::TEXT_BRIGHT
                    } else {
                        colors::TEXT_MUTED
                    };

                    let btn = egui::Button::new(
                        egui::RichText::new(tld.as_str())
                            .color(text_color)
                            .size(12.0),
                    )
                    .fill(color)
                    .corner_radius(egui::CornerRadius::same(12))
                    .min_size(egui::vec2(0.0, 24.0));

                    if ui.add(btn).clicked() {
                        *enabled = !*enabled;
                    }
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.checkbox(&mut state.custom_tld_enabled, "Custom TLD:");
                ui.add_enabled(
                    state.custom_tld_enabled,
                    egui::TextEdit::singleline(&mut state.custom_tld)
                        .desired_width(120.0)
                        .hint_text(".yourtld"),
                );
            });

            // Status message
            if let Some(msg) = &state.status_message {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(msg)
                        .color(colors::ACCENT_GREEN)
                        .size(11.0)
                        .italics(),
                );
            }
        });

    fetch_requested
}

fn render_openrouter_settings(ui: &mut egui::Ui, state: &mut SettingsState) -> bool {
    let mut fetch_requested = false;

    // API Key
    render_key_input(
        ui,
        "OpenRouter API Key",
        &mut state.openrouter_key,
        &mut state.openrouter_key_visible,
        credentials::KEY_OPENROUTER,
        &mut state.status_message,
    );

    ui.add_space(4.0);

    // Model selector
    ui.horizontal(|ui: &mut egui::Ui| {
        ui.label(
            egui::RichText::new("Model")
                .color(colors::TEXT_SECONDARY)
                .size(12.0),
        );
        let current_label = state.openrouter_models
            .get(state.openrouter_model_idx)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| "Auto".to_string());

        egui::ComboBox::from_id_salt("openrouter_model")
            .selected_text(current_label)
            .width(200.0)
            .show_ui(ui, |ui: &mut egui::Ui| {
                for (i, (_, label)) in state.openrouter_models.iter().enumerate() {
                    ui.selectable_value(&mut state.openrouter_model_idx, i, label);
                }
            });
            
        if theme::secondary_button(ui, "Refresh").clicked() {
            fetch_requested = true;
            state.status_message = Some("Fetching latest free models...".to_string());
        }
    });

    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("💡 Sign up free at openrouter.ai — no credit card required")
            .color(colors::TEXT_MUTED)
            .size(11.0)
            .italics(),
    );
    
    fetch_requested
}

fn render_ollama_settings(ui: &mut egui::Ui, state: &mut SettingsState) {
    ui.horizontal(|ui: &mut egui::Ui| {
        let (icon, text, color) = if state.ollama_available {
            ("🟢", "Ollama is running", colors::ACCENT_GREEN)
        } else {
            (
                "🔴",
                "Ollama not detected on localhost:11434",
                colors::ACCENT_RED,
            )
        };
        ui.label(egui::RichText::new(format!("{icon} {text}")).color(color).size(12.0));
    });

    if state.ollama_available && !state.ollama_models.is_empty() {
        ui.add_space(4.0);
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label(
                egui::RichText::new("Model")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            egui::ComboBox::from_id_salt("ollama_model")
                .selected_text(&state.ollama_model)
                .width(200.0)
                .show_ui(ui, |ui: &mut egui::Ui| {
                    for model in &state.ollama_models {
                        ui.selectable_value(&mut state.ollama_model, model.clone(), model);
                    }
                });
        });
    }

    ui.add_space(2.0);
    ui.label(
        egui::RichText::new("💡 Install Ollama and run: ollama pull llama3")
            .color(colors::TEXT_MUTED)
            .size(11.0)
            .italics(),
    );
}

fn render_key_input(
    ui: &mut egui::Ui,
    label: &str,
    key: &mut String,
    visible: &mut bool,
    credential_key: &str,
    status_message: &mut Option<String>,
) {
    ui.label(
        egui::RichText::new(label)
            .color(colors::TEXT_SECONDARY)
            .size(12.0),
    );
    ui.horizontal(|ui: &mut egui::Ui| {
        let edit = if *visible {
            egui::TextEdit::singleline(key)
                .desired_width(200.0)
                .text_color(colors::TEXT_BRIGHT)
        } else {
            egui::TextEdit::singleline(key)
                .password(true)
                .desired_width(200.0)
                .text_color(colors::TEXT_BRIGHT)
        };
        ui.add(edit);

        if ui.small_button(if *visible { "🙈" } else { "👁" }).clicked() {
            *visible = !*visible;
        }

        if theme::secondary_button(ui, "Save").clicked() {
            if let Err(e) = credentials::save_credential(credential_key, key) {
                tracing::error!("Failed to save credential: {e}");
                *status_message = Some(format!("Error saving key: {e}"));
            } else {
                *status_message = Some(format!("{label} saved successfully!"));
            }
        }

        if theme::secondary_button(ui, "Clear").clicked() {
            key.clear();
            if let Err(e) = credentials::delete_credential(credential_key) {
                *status_message = Some(format!("Error clearing key: {e}"));
            } else {
                *status_message = Some(format!("{label} cleared!"));
            }
        }
    });
}
