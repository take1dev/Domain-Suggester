#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Domain Suggest & Checker — AI-powered domain name generation and availability checking.
///
/// Entry point: spawns a background Tokio runtime and launches the egui GUI on the main thread.

mod credentials;
mod domain;
mod llm;
mod ui;
mod worker;

use eframe::egui;

fn load_icon() -> std::sync::Arc<egui::IconData> {
    let icon = include_bytes!("../Icon.png");
    let image = image::load_from_memory(icon)
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    
    std::sync::Arc::new(egui::IconData {
        rgba,
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Domain Suggest & Checker");

    // Spawn Tokio runtime in the background and keep it alive in main's scope
    let _rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    let rt_handle = _rt.handle().clone();

    // Launch eframe
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Domain Suggest & Checker")
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    let result = eframe::run_native(
        "Domain Suggest & Checker",
        options,
        Box::new(move |cc| Ok(Box::new(ui::App::new(cc, rt_handle)))),
    );

    // Forcefully kill any lingering background threads (like orphaned HTTP requests) 
    // when the user closes the main UI window.
    std::process::exit(0);
}
