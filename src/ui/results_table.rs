/// Results table — sortable domain results with availability, pricing, and actions.

use crate::domain::{Availability, DomainStatus};
use crate::ui::settings_panel::PreferredRegistrar;
use crate::ui::theme::colors;
use eframe::egui;

/// Column to sort by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Domain,
    Availability,
    RegPrice,
    RenewPrice,
}

/// Render the results table.
pub fn render(
    ui: &mut egui::Ui,
    results: &[DomainStatus],
    sort_col: &mut SortColumn,
    sort_ascending: &mut bool,
    filter_available_only: &mut bool,
    progress: Option<(usize, usize)>,
    registrar: PreferredRegistrar,
) {
    egui::Frame::new()
        .fill(colors::BG_PANEL)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, colors::BORDER))
        .show(ui, |ui: &mut egui::Ui| {
            // Filter
            let filtered: Vec<&DomainStatus> = if *filter_available_only {
                results
                    .iter()
                    .filter(|r| r.availability.is_available())
                    .collect()
            } else {
                results.iter().collect()
            };

            // Sort
            let mut sorted: Vec<&DomainStatus> = filtered;
            sorted.sort_by(|a, b| {
                let cmp = match sort_col {
                    SortColumn::Domain => {
                        let tld_a = a.domain.rsplit('.').next().unwrap_or("");
                        let tld_b = b.domain.rsplit('.').next().unwrap_or("");
                        match tld_a.cmp(tld_b) {
                            std::cmp::Ordering::Equal => a.domain.cmp(&b.domain),
                            other => other,
                        }
                    }
                    SortColumn::Availability => {
                        avail_sort_key(&a.availability).cmp(&avail_sort_key(&b.availability))
                    }
                    SortColumn::RegPrice => {
                        let pa = a
                            .price
                            .as_ref()
                            .and_then(|p| p.registration)
                            .unwrap_or(f64::MAX);
                        let pb = b
                            .price
                            .as_ref()
                            .and_then(|p| p.registration)
                            .unwrap_or(f64::MAX);
                        pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    SortColumn::RenewPrice => {
                        let pa = a
                            .price
                            .as_ref()
                            .and_then(|p| p.renewal)
                            .unwrap_or(f64::MAX);
                        let pb = b
                            .price
                            .as_ref()
                            .and_then(|p| p.renewal)
                            .unwrap_or(f64::MAX);
                        pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
                    }
                };
                if *sort_ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });

            // Header row
            ui.horizontal(|ui: &mut egui::Ui| {
                ui.label(
                    egui::RichText::new("📋  Results")
                        .size(16.0)
                        .color(colors::TEXT_BRIGHT)
                        .strong(),
                );

                ui.add_space(16.0);

                // Progress indicator
                if let Some((checked, total)) = progress {
                    let pct = if total > 0 {
                        (checked as f32 / total as f32 * 100.0) as u32
                    } else {
                        0
                    };

                    ui.label(
                        egui::RichText::new(format!("{checked}/{total} checked ({pct}%)"))
                            .color(colors::ACCENT_CYAN)
                            .size(12.0),
                    );

                    if checked < total {
                        ui.spinner();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui: &mut egui::Ui| {
                    if !sorted.is_empty() {
                        if crate::ui::theme::secondary_button(ui, "💾 Export .txt").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Text File", &["txt"])
                                .set_file_name("domains.txt")
                                .save_file()
                            {
                                let mut out = String::new();
                                for res in &sorted {
                                    let status_str = match &res.availability {
                                        Availability::Available => "Available",
                                        Availability::Taken => "Taken",
                                        Availability::Pending => "Pending",
                                        Availability::Unknown => "Unknown",
                                        Availability::Error(e) => e,
                                    };
                                    out.push_str(&format!("{:<25} | {}\n", res.domain, status_str));
                                }
                                let _ = std::fs::write(&path, out);
                            }
                        }
                        ui.add_space(8.0);
                    }

                    ui.checkbox(
                        filter_available_only,
                        egui::RichText::new("Available only")
                            .color(colors::TEXT_SECONDARY)
                            .size(12.0),
                    );

                    // Count summary
                    let available_count = results
                        .iter()
                        .filter(|r| r.availability.is_available())
                        .count();
                    let total_count = results.len();
                    if total_count > 0 {
                        ui.label(
                            egui::RichText::new(format!(
                                "✅ {available_count} available / {total_count} total"
                            ))
                            .color(if available_count > 0 {
                                colors::ACCENT_GREEN
                            } else {
                                colors::TEXT_MUTED
                            })
                            .size(12.0),
                        );
                    }
                });
            });

            ui.add_space(8.0);

            if results.is_empty() {
                ui.centered_and_justified(|ui: &mut egui::Ui| {
                    ui.label(
                        egui::RichText::new("No results yet. Enter keywords and click Generate!")
                            .color(colors::TEXT_MUTED)
                            .size(14.0)
                            .italics(),
                    );
                });
                return;
            }

            // Table
            let row_height = 28.0;
            let available_width = ui.available_width();

            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::initial(available_width * 0.35).at_least(120.0))
                .column(egui_extras::Column::initial(80.0).at_least(60.0))
                .column(egui_extras::Column::initial(100.0).at_least(70.0))
                .column(egui_extras::Column::initial(100.0).at_least(70.0))
                .column(egui_extras::Column::remainder().at_least(110.0))
                .header(row_height, |mut header| {
                    header.col(|ui: &mut egui::Ui| {
                        if sort_header_button(ui, "Domain", *sort_col == SortColumn::Domain, *sort_ascending) {
                            toggle_sort(sort_col, sort_ascending, SortColumn::Domain);
                        }
                    });
                    header.col(|ui: &mut egui::Ui| {
                        if sort_header_button(ui, "Status", *sort_col == SortColumn::Availability, *sort_ascending) {
                            toggle_sort(sort_col, sort_ascending, SortColumn::Availability);
                        }
                    });
                    header.col(|ui: &mut egui::Ui| {
                        if sort_header_button(ui, "Reg. Price", *sort_col == SortColumn::RegPrice, *sort_ascending) {
                            toggle_sort(sort_col, sort_ascending, SortColumn::RegPrice);
                        }
                    });
                    header.col(|ui: &mut egui::Ui| {
                        if sort_header_button(ui, "Renewal", *sort_col == SortColumn::RenewPrice, *sort_ascending) {
                            toggle_sort(sort_col, sort_ascending, SortColumn::RenewPrice);
                        }
                    });
                    header.col(|ui: &mut egui::Ui| {
                        ui.label(
                            egui::RichText::new("Actions")
                                .color(colors::TEXT_SECONDARY)
                                .size(12.0)
                                .strong(),
                        );
                    });
                })
                .body(|body| {
                    body.rows(row_height, sorted.len(), |mut row| {
                        let idx = row.index();
                        let status = sorted[idx];

                        // Domain name
                        row.col(|ui: &mut egui::Ui| {
                            let color = match &status.availability {
                                Availability::Available => colors::ACCENT_GREEN,
                                Availability::Taken => colors::TEXT_MUTED,
                                Availability::Pending => colors::TEXT_SECONDARY,
                                _ => colors::ACCENT_AMBER,
                            };
                            ui.label(
                                egui::RichText::new(&status.domain)
                                    .color(color)
                                    .size(13.0)
                                    .strong(),
                            );
                        });

                        // Availability status
                        row.col(|ui: &mut egui::Ui| {
                            let (symbol, color) = match &status.availability {
                                Availability::Available => ("✅ Available", colors::ACCENT_GREEN),
                                Availability::Taken => ("❌ Taken", colors::ACCENT_RED),
                                Availability::Pending => ("⏳ Checking", colors::ACCENT_AMBER),
                                Availability::Error(_) => ("⚠️ Error", colors::ACCENT_RED),
                                Availability::Unknown => ("❓ Unknown", colors::TEXT_MUTED),
                            };
                            let resp = ui.label(egui::RichText::new(symbol).color(color).size(12.0));
                            if let Availability::Error(e) = &status.availability {
                                resp.on_hover_text(e);
                            }
                        });

                        // Registration price
                        row.col(|ui: &mut egui::Ui| {
                            let text = status
                                .price
                                .as_ref()
                                .and_then(|p| p.registration.map(|r| format!("${:.2}", r)))
                                .unwrap_or_else(|| "—".into());
                            ui.label(
                                egui::RichText::new(text)
                                    .color(colors::TEXT_SECONDARY)
                                    .size(12.0),
                            );
                        });

                        // Renewal price
                        row.col(|ui: &mut egui::Ui| {
                            let text = status
                                .price
                                .as_ref()
                                .and_then(|p| p.renewal.map(|r| format!("${:.2}", r)))
                                .unwrap_or_else(|| "—".into());
                            ui.label(
                                egui::RichText::new(text)
                                    .color(colors::TEXT_SECONDARY)
                                    .size(12.0),
                            );
                        });

                        // Actions
                        row.col(|ui: &mut egui::Ui| {
                            ui.horizontal(|ui: &mut egui::Ui| {
                                if ui
                                    .small_button("📋")
                                    .on_hover_text("Copy domain name")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(status.domain.clone());
                                }

                                if status.availability.is_available() {
                                    if ui
                                        .small_button("🛒 Buy")
                                        .on_hover_text(format!("Register via {}", registrar.label()))
                                        .clicked()
                                    {
                                        ui.ctx().open_url(egui::output::OpenUrl::new_tab(registrar.purchase_url(&status.domain)));
                                    }
                                }
                            });
                        });
                    });
                });
        });
}

fn sort_header_button(ui: &mut egui::Ui, text: &str, is_active: bool, ascending: bool) -> bool {
    let arrow = if is_active {
        if ascending { " ↑" } else { " ↓" }
    } else {
        " ↕"
    };
    let label = format!("{text}{arrow}");
    let color = if is_active {
        colors::ACCENT_BLUE
    } else {
        colors::TEXT_SECONDARY
    };
    ui.add(
        egui::Label::new(egui::RichText::new(label).color(color).size(12.0).strong())
            .sense(egui::Sense::click()),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
    .clicked()
}

fn toggle_sort(col: &mut SortColumn, ascending: &mut bool, target: SortColumn) {
    if *col == target {
        *ascending = !*ascending;
    } else {
        *col = target;
        *ascending = true;
    }
}

fn avail_sort_key(a: &Availability) -> u8 {
    match a {
        Availability::Available => 0,
        Availability::Pending => 1,
        Availability::Unknown => 2,
        Availability::Taken => 3,
        Availability::Error(_) => 4,
    }
}
