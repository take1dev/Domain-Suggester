# Domain Suggest & Checker

A high-performance, asynchronous Rust GUI application that leverages state-of-the-art Generative AI to brainstorm unique, brandable domain names and instantly verifies their registration availability utilizing concurrent RDAP and WhoisFreaks lookup pipelines.

## Key Features

*   **AI-Powered Generation**: Employs fine-tuned system prompts to construct concise, catchy, and readable brand names while avoiding typical generative hallucination.
*   **Multi-LLM Architecture**: Supports both local inference via **Ollama** and cloud inference via **OpenRouter**, including dynamic fetching of the newest available free-tier cloud models.
*   **Highly Concurrent Validation**: Utilizes a bounded, multi-threaded `Tokio` backend to fan-out asynchronous domain availability checks (RDAP/Whois) without blocking the user interface.
*   **Responsive Native UI**: Constructed fully in Rust utilizing the immediate-mode `eframe`/`egui` framework for a seamless, cross-platform user experience.
*   **Persistent User Context**: Features lightweight I/O endpoints to locally save and restore search criteria profiles and securely maintain API credentials across runtimes.

## Installation

This application requires the standard Rust toolchain to compile. 

**Dependencies:**
*   `cargo` & `rustc` (Edition 2021)
*   `eframe` / `egui_extras` (UI Framework)
*   `tokio` (Asynchronous runtime engine)
*   `reqwest` (HTTP Client for LLM and RDAP polling)
*   `winres` (Automated Windows `.ico` embedding)

To install and compile the software:
```bash
# Clone the repository
git clone https://github.com/your-username/domain-suggest.git
cd domain-suggest

# Build the release binary
cargo build --release

# The compiled output will be available at target/release/domain-suggest.exe
```

## Quick Start & Usage

Simply run the application using `cargo run`. The application architecture explicitly spins up a dedicated `tokio` runtime on the primary thread before orchestrating the native windowing event loop.

```rust
use eframe::egui;

fn main() -> eframe::Result<()> {
    // 1. Spawn the asynchronous Tokio runtime in the background
    let _rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime");

    // 2. Configure windowing options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Domain Suggest & Checker")
            .with_inner_size([1100.0, 750.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    // 3. Mount the GUI and pass the runtime handle to background asynchronous workers
    let result = eframe::run_native(
        "Domain Suggest & Checker",
        options,
        Box::new(move |cc| Ok(Box::new(ui::App::new(cc, _rt.handle().clone())))),
    );

    // Ensure pristine shutdown of orphan HTTP socket processes
    std::process::exit(0);
}
```

Once launched:
1. Navigate to the **Settings** tab to input your free OpenRouter API key.
2. Select your target **TLDs** (e.g., `.com`, `.io`, `.ai`).
3. Return to the **Generator**, specify your keywords/industry, and click **Generate & Check**.

## License

This project is open-sourced under the **MIT License**. See the `LICENSE` file for full details.
