/// Main application struct and eframe::App implementation.
///
/// Orchestrates the input panel, results table, and settings panel.
/// Communicates with the background Tokio worker via mpsc channels.

pub mod theme;
pub mod input_panel;
pub mod results_table;
pub mod settings_panel;

use crate::domain::DomainStatus;
use crate::worker::{self, WorkerCommand, WorkerResult};
use eframe::egui;
use input_panel::InputPanelState;
use results_table::SortColumn;
use settings_panel::SettingsState;
use tokio::sync::mpsc;

/// Which tab/panel the user is viewing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Generator,
    Settings,
}

pub struct App {
    // Worker channels
    cmd_tx: mpsc::UnboundedSender<WorkerCommand>,
    result_rx: mpsc::UnboundedReceiver<WorkerResult>,

    // UI state
    tab: Tab,
    input: InputPanelState,
    settings: SettingsState,

    // Results
    suggestions: Vec<String>,
    results: Vec<DomainStatus>,
    sort_col: SortColumn,
    sort_ascending: bool,
    filter_available_only: bool,
    progress: Option<(usize, usize)>,

    // Status
    is_busy: bool,
    error_message: Option<String>,

    // Ollama detection
    ollama_checked: bool,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rt_handle: tokio::runtime::Handle,
    ) -> Self {
        // Apply theme
        theme::apply_theme(&cc.egui_ctx);

        // Create channels
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        let cmd_tx = worker::spawn_worker(rt_handle, result_tx, cc.egui_ctx.clone());

        Self {
            cmd_tx,
            result_rx,
            tab: Tab::Generator,
            input: InputPanelState::default(),
            settings: SettingsState::load_or_default(),
            suggestions: Vec::new(),
            results: Vec::new(),
            sort_col: SortColumn::Domain,
            sort_ascending: true,
            filter_available_only: false,
            progress: None,
            is_busy: false,
            error_message: None,
            ollama_checked: false,
        }
    }

    /// Poll the result channel for new data from the worker.
    fn poll_results(&mut self) {
        while let Ok(result) = self.result_rx.try_recv() {
            match result {
                WorkerResult::Suggestions(names) => {
                    self.suggestions = names;
                    self.error_message = None;
                }
                WorkerResult::DomainResult(status) => {
                    if let Some(existing) = self.results.iter_mut().find(|r| r.domain == status.domain) {
                        *existing = status;
                    } else {
                        self.results.push(status);
                    }
                }
                WorkerResult::Progress { checked, total } => {
                    self.progress = Some((checked, total));
                }
                WorkerResult::OpenRouterModels(models) => {
                    self.settings.openrouter_models = models;
                    self.settings.openrouter_model_idx = 0;
                }
                WorkerResult::Error(msg) => {
                    self.error_message = Some(msg);
                    self.is_busy = false;
                }
                WorkerResult::Done => {
                    self.is_busy = false;
                }
            }
        }
    }

    /// Fire a generate-and-check command to the worker.
    fn trigger_generate(&mut self) {
        self.is_busy = true;
        self.results.clear();
        self.suggestions.clear();
        self.progress = None;
        self.error_message = None;

        let tlds = self.settings.enabled_tlds();
        if tlds.is_empty() {
            self.error_message = Some("No TLDs selected! Enable at least one in Settings.".into());
            self.is_busy = false;
            return;
        }

        let cmd = WorkerCommand::GenerateAndCheck {
            request: self.input.to_request(),
            tlds,
            llm_kind: self.settings.llm_kind.clone(),
            openrouter_key: Some(self.settings.openrouter_key.clone()),
            openrouter_model: Some(self.settings.selected_openrouter_model()),
            ollama_model: Some(self.settings.ollama_model.clone()),
            checker_kind: self.settings.checker_kind,
            whoisfreaks_key: Some(self.settings.whoisfreaks_key.clone()),
        };

        let _ = self.cmd_tx.send(cmd);
    }

    /// Ask worker to fetch OpenRouter models.
    pub fn fetch_openrouter_models(&mut self) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchOpenRouterModels);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll worker results
        self.poll_results();

        // One-time Ollama detection (non-blocking)
        if !self.ollama_checked {
            self.ollama_checked = true;
        }

        // Top bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui: &mut egui::Ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("🌐 Domain Suggest & Checker")
                        .size(20.0)
                        .color(theme::colors::ACCENT_CYAN)
                        .strong(),
                );

                ui.add_space(24.0);

                // Tab buttons
                let gen_text = egui::RichText::new("🧠 Generator").size(14.0);
                let set_text = egui::RichText::new("⚙️ Settings").size(14.0);

                if ui
                    .selectable_label(self.tab == Tab::Generator, gen_text)
                    .clicked()
                {
                    self.tab = Tab::Generator;
                }
                if ui
                    .selectable_label(self.tab == Tab::Settings, set_text)
                    .clicked()
                {
                    self.tab = Tab::Settings;
                }

                // Right side — status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui: &mut egui::Ui| {
                    ui.add_space(8.0);
                    if self.is_busy {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new("Working…")
                                .color(theme::colors::ACCENT_AMBER)
                                .size(12.0),
                        );
                    }
                });
            });
            ui.add_space(4.0);
        });

        // Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::colors::BG_DARKEST)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui: &mut egui::Ui| {
                match self.tab {
                    Tab::Generator => {
                        // Error banner
                        if let Some(err) = &self.error_message.clone() {
                            egui::Frame::new()
                                .fill(theme::colors::ACCENT_RED.gamma_multiply(0.15))
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::same(10))
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    theme::colors::ACCENT_RED.gamma_multiply(0.5),
                                ))
                                .show(ui, |ui: &mut egui::Ui| {
                                    ui.horizontal(|ui: &mut egui::Ui| {
                                        ui.label(
                                            egui::RichText::new("⚠️")
                                                .color(theme::colors::ACCENT_RED)
                                                .size(16.0),
                                        );
                                        ui.label(
                                            egui::RichText::new(err)
                                                .color(theme::colors::ACCENT_RED)
                                                .size(13.0),
                                        );
                                    });
                                });
                            ui.add_space(8.0);
                        }

                        // Input panel
                        if input_panel::render(ui, &mut self.input, self.is_busy) {
                            self.trigger_generate();
                        }

                        ui.add_space(12.0);

                        // Suggestions chips
                        if !self.suggestions.is_empty() {
                            egui::Frame::new()
                                .fill(theme::colors::BG_PANEL)
                                .corner_radius(egui::CornerRadius::same(10))
                                .inner_margin(egui::Margin::same(12))
                                .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                                .show(ui, |ui: &mut egui::Ui| {
                                    ui.label(
                                        egui::RichText::new("💡 AI Suggestions")
                                            .color(theme::colors::TEXT_BRIGHT)
                                            .size(13.0)
                                            .strong(),
                                    );
                                    ui.add_space(4.0);
                                    ui.horizontal_wrapped(|ui: &mut egui::Ui| {
                                        for name in &self.suggestions {
                                            let chip = egui::Button::new(
                                                egui::RichText::new(name)
                                                    .color(theme::colors::ACCENT_CYAN)
                                                    .size(12.0),
                                            )
                                            .fill(theme::colors::ACCENT_BLUE.gamma_multiply(0.15))
                                            .corner_radius(egui::CornerRadius::same(12))
                                            .stroke(egui::Stroke::new(
                                                1.0,
                                                theme::colors::ACCENT_BLUE.gamma_multiply(0.4),
                                            ));
                                            if ui.add(chip).on_hover_text("Click to copy").clicked()
                                            {
                                                ui.ctx().copy_text(name.clone());
                                            }
                                        }
                                    });
                                });

                            ui.add_space(12.0);
                        }

                        // Results table
                        results_table::render(
                            ui,
                            &self.results,
                            &mut self.sort_col,
                            &mut self.sort_ascending,
                            &mut self.filter_available_only,
                            self.progress,
                            self.settings.preferred_registrar,
                        );
                    }
                    Tab::Settings => {
                        let did_fetch = settings_panel::render(ui, &mut self.settings);
                        if did_fetch {
                            self.fetch_openrouter_models();
                        }
                    }
                }
            });
            
        // Auto-save settings if they were mutated during the frame
        self.settings.save_if_changed();
    }
}
