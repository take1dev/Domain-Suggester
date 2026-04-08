/// Premium dark theme for the Domain Suggest & Checker app.
///
/// Deep navy background, electric blue accents, soft typography.

use eframe::egui;

/// Color palette constants.
pub mod colors {
    use eframe::egui::Color32;

    // Backgrounds
    pub const BG_DARKEST: Color32 = Color32::from_rgb(10, 12, 20);
    pub const BG_DARK: Color32 = Color32::from_rgb(16, 20, 32);
    pub const BG_PANEL: Color32 = Color32::from_rgb(22, 27, 45);
    pub const BG_CARD: Color32 = Color32::from_rgb(28, 34, 55);
    pub const BG_HOVER: Color32 = Color32::from_rgb(35, 42, 68);
    pub const BG_INPUT: Color32 = Color32::from_rgb(18, 22, 38);

    // Accents
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
    pub const ACCENT_BLUE_HOVER: Color32 = Color32::from_rgb(96, 165, 250);
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(34, 211, 238);
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(139, 92, 246);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(239, 68, 68);
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(245, 158, 11);

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(226, 232, 240);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
    pub const TEXT_BRIGHT: Color32 = Color32::from_rgb(248, 250, 252);

    // Borders
    pub const BORDER: Color32 = Color32::from_rgb(40, 48, 75);
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(59, 130, 246);
}

/// Apply the custom dark theme to the egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = egui::Style::default();

    // Visuals
    let v = &mut style.visuals;
    v.dark_mode = true;
    v.override_text_color = Some(colors::TEXT_PRIMARY);

    // Panel backgrounds
    v.panel_fill = colors::BG_DARK;
    v.window_fill = colors::BG_PANEL;
    v.extreme_bg_color = colors::BG_INPUT;
    v.faint_bg_color = colors::BG_CARD;

    // Widgets
    v.widgets.noninteractive.bg_fill = colors::BG_PANEL;
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_SECONDARY);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, colors::BORDER);
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);

    v.widgets.inactive.bg_fill = colors::BG_CARD;
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_PRIMARY);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, colors::BORDER);
    v.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

    v.widgets.hovered.bg_fill = colors::BG_HOVER;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_BRIGHT);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, colors::ACCENT_BLUE);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

    v.widgets.active.bg_fill = colors::ACCENT_BLUE;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_BRIGHT);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, colors::ACCENT_BLUE_HOVER);
    v.widgets.active.corner_radius = egui::CornerRadius::same(6);

    v.widgets.open.bg_fill = colors::BG_HOVER;
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0, colors::TEXT_BRIGHT);
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, colors::ACCENT_BLUE);
    v.widgets.open.corner_radius = egui::CornerRadius::same(6);

    // Selection
    v.selection.bg_fill = colors::ACCENT_BLUE.gamma_multiply(0.3);
    v.selection.stroke = egui::Stroke::new(1.0, colors::ACCENT_BLUE);

    // Window
    v.window_corner_radius = egui::CornerRadius::same(10);
    v.window_shadow = egui::epaint::Shadow {
        spread: 8,
        blur: 20,
        color: egui::Color32::from_black_alpha(100),
        offset: [0, 4],
    };
    v.window_stroke = egui::Stroke::new(1.0, colors::BORDER);

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.text_edit_width = 280.0;

    ctx.set_style(style);
}

/// Draw a section header with accent underline.
pub fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui: &mut egui::Ui| {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        painter.line_segment(
            [
                egui::pos2(rect.left(), rect.bottom() + 4.0),
                egui::pos2(rect.left() + 40.0, rect.bottom() + 4.0),
            ],
            egui::Stroke::new(2.0, colors::ACCENT_BLUE),
        );
        ui.label(
            egui::RichText::new(text)
                .size(16.0)
                .color(colors::TEXT_BRIGHT)
                .strong(),
        );
    });
    ui.add_space(4.0);
}

/// Styled primary button.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(colors::TEXT_BRIGHT)
            .strong()
            .size(14.0),
    )
    .fill(colors::ACCENT_BLUE)
    .corner_radius(egui::CornerRadius::same(8))
    .min_size(egui::vec2(120.0, 32.0));

    ui.add(button)
}

/// Styled danger / secondary button.
pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(colors::TEXT_BRIGHT)
            .size(13.0),
    )
    .fill(colors::ACCENT_RED.gamma_multiply(0.7))
    .corner_radius(egui::CornerRadius::same(8));

    ui.add(button)
}

/// A subtle secondary button.
pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .color(colors::TEXT_SECONDARY)
            .size(13.0),
    )
    .fill(colors::BG_CARD)
    .stroke(egui::Stroke::new(1.0, colors::BORDER))
    .corner_radius(egui::CornerRadius::same(8));

    ui.add(button)
}
