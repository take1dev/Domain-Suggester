/// Input panel — keyword entry, industry, brand personality, and generation controls.

use crate::llm::{BrandPersonality, CreativityMode, SuggestionRequest};
use crate::ui::theme::{self, colors};
use eframe::egui;
use serde::{Deserialize, Serialize};

/// Persistent state for the input panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputPanelState {
    pub keywords: String,
    pub industry: String,
    pub personality: BrandPersonality,
    pub max_length: usize,
    pub count: usize,
    pub creativity: CreativityMode,
    #[serde(skip)]
    pub status_message: Option<String>,
}

impl Default for InputPanelState {
    fn default() -> Self {
        Self {
            keywords: String::new(),
            industry: String::new(),
            personality: BrandPersonality::Bold,
            max_length: 14,
            count: 15,
            creativity: CreativityMode::Creative,
            status_message: None,
        }
    }
}

impl InputPanelState {
    /// Convert to a `SuggestionRequest` for the worker.
    pub fn to_request(&self) -> SuggestionRequest {
        SuggestionRequest {
            keywords: self.keywords.clone(),
            industry: self.industry.clone(),
            personality: self.personality,
            max_length: self.max_length,
            count: self.count,
            creativity: self.creativity,
        }
    }

    /// Whether the input is valid for generation.
    pub fn is_valid(&self) -> bool {
        !self.keywords.trim().is_empty()
    }

    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Option<Self> {
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }
}

/// Render the input panel. Returns `true` if the Generate button was pressed.
pub fn render(ui: &mut egui::Ui, state: &mut InputPanelState, is_busy: bool) -> bool {
    let mut generate_pressed = false;

    egui::Frame::new()
        .fill(colors::BG_PANEL)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui: &mut egui::Ui| {
            theme::section_header(ui, "🧠  Domain Name Generator");

            ui.add_space(8.0);

            // Keywords
            ui.label(
                egui::RichText::new("Keywords")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            let kw_edit = egui::TextEdit::singleline(&mut state.keywords)
                .hint_text("e.g. cloud, sync, productivity, fast")
                .desired_width(f32::INFINITY)
                .text_color(colors::TEXT_BRIGHT);
            ui.add(kw_edit);

            ui.add_space(6.0);

            // Industry
            ui.label(
                egui::RichText::new("Industry / Niche")
                    .color(colors::TEXT_SECONDARY)
                    .size(12.0),
            );
            let ind_edit = egui::TextEdit::singleline(&mut state.industry)
                .hint_text("e.g. SaaS, Health & Wellness, Fintech")
                .desired_width(f32::INFINITY)
                .text_color(colors::TEXT_BRIGHT);
            ui.add(ind_edit);

            ui.add_space(8.0);

            // Brand personality
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label(
                    egui::RichText::new("Brand Personality")
                        .color(colors::TEXT_SECONDARY)
                        .size(12.0),
                );
                ui.add_space(8.0);
                egui::ComboBox::from_id_salt("personality_combo")
                    .selected_text(state.personality.label())
                    .width(140.0)
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        for &p in BrandPersonality::ALL {
                            ui.selectable_value(&mut state.personality, p, p.label());
                        }
                    });
            });

            ui.add_space(6.0);

            // Sliders row
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label(
                    egui::RichText::new("Max Length")
                        .color(colors::TEXT_SECONDARY)
                        .size(12.0),
                );
                let mut len = state.max_length as f64;
                ui.add(egui::Slider::new(&mut len, 4.0..=24.0).integer());
                state.max_length = len as usize;

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("Count")
                        .color(colors::TEXT_SECONDARY)
                        .size(12.0),
                );
                let mut cnt = state.count as f64;
                ui.add(egui::Slider::new(&mut cnt, 3.0..=50.0).integer());
                state.count = cnt as usize;
            });

            ui.add_space(6.0);

            // Creativity toggle
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label(
                    egui::RichText::new("Mode")
                        .color(colors::TEXT_SECONDARY)
                        .size(12.0),
                );
                ui.selectable_value(
                    &mut state.creativity,
                    CreativityMode::Conservative,
                    egui::RichText::new("🎯 Conservative").size(13.0),
                );
                ui.selectable_value(
                    &mut state.creativity,
                    CreativityMode::Creative,
                    egui::RichText::new("✨ Creative").size(13.0),
                );
            });

            ui.add_space(12.0);

            // Generate button
            ui.horizontal(|ui: &mut egui::Ui| {
                let enabled = state.is_valid() && !is_busy;
                ui.add_enabled_ui(enabled, |ui: &mut egui::Ui| {
                    if theme::primary_button(ui, if is_busy { "⏳ Generating…" } else { "🚀 Generate & Check" }).clicked() {
                        generate_pressed = true;
                    }
                });

                ui.add_space(16.0);

                if theme::secondary_button(ui, "💾 Save As...").on_hover_text("Save criteria to a file").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_file_name("criteria.json")
                        .save_file()
                    {
                        match state.save_to_file(path.to_str().unwrap_or("criteria.json")) {
                            Ok(_) => state.status_message = Some(format!("Saved to {}", path.file_name().unwrap_or_default().to_string_lossy())),
                            Err(e) => state.status_message = Some(format!("Error saving: {e}")),
                        }
                    }
                }
                
                if theme::secondary_button(ui, "📂 Load...").on_hover_text("Load criteria from a file").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file()
                    {
                        if let Some(mut loaded) = InputPanelState::load_from_file(path.to_str().unwrap_or_default()) {
                            loaded.status_message = Some(format!("Loaded from {}", path.file_name().unwrap_or_default().to_string_lossy()));
                            *state = loaded;
                        } else {
                            state.status_message = Some("Failed to load or parse the file.".to_string());
                        }
                    }
                }

                if state.keywords.is_empty() {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("Enter keywords to get started")
                            .color(colors::TEXT_MUTED)
                            .size(12.0)
                            .italics(),
                    );
                }

                if let Some(msg) = &state.status_message {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(msg)
                            .color(colors::ACCENT_GREEN)
                            .size(11.0)
                            .italics(),
                    );
                }
            });
        });

    generate_pressed
}
