<div align="center">
  <img src="Icon.png" alt="Icon" width="120" height="auto" />
  <h1>Domain Suggest & Checker</h1>
  
  <p>
    <strong>A high-performance, asynchronous Rust GUI application that leverages state-of-the-art Generative AI to brainstorm unique, brandable domain names and instantly verifies their registration availability.</strong>
  </p>
  
  <p>
    <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.75+-orange.svg?style=flat-square&logo=rust" alt="Rust Version" /></a>
    <a href="https://github.com/emilk/egui"><img src="https://img.shields.io/badge/GUI-egui_0.31-blue.svg?style=flat-square" alt="GUI Framework" /></a>
    <a href="https://tokio.rs"><img src="https://img.shields.io/badge/Async-Tokio-yellow.svg?style=flat-square" alt="Async Runtime" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg?style=flat-square" alt="License" /></a>
  </p>
</div>

---

## ✨ Key Features

* 🧠 **AI-Powered Generation**: Employs fine-tuned system prompts and Brand Archetypes to construct concise, catchy, and readable brand names avoiding typical generative hallucination.
* ⚡ **Ultra-Fast Mass Verification**: Utilizes a highly concurrent `tokio` multi-threading engine to fan out availability checks across Unlimited DNS, WhoisFreaks pipelines, or RDAP.
* 🌐 **Multi-LLM Architecture**: Run local inference securely via **Ollama** or tap into powerful cloud models for free using **OpenRouter** (Llama 3, Gemma, Mistral, Qwen). 
* 🎨 **Premium Native UI**: Beautiful Deep Navy aesthetic constructed entirely in Rust via the immediate-mode `eframe`/`egui` framework. Responsive, cross-platform, and blazing fast.
* 💾 **Persistent Contextual Engine**: Lightweight state-saving natively remembers your workflow configurations, selected models, and securely binds API keys through OS-level keychains.

## 🛠️ Tech Stack

- **Core**: [Rust](https://www.rust-lang.org/)
- **Frontend / GUI**: [egui](https://github.com/emilk/egui) / [eframe](https://crates.io/crates/eframe)
- **Concurrency / Async**: [Tokio](https://tokio.rs/)
- **Networking**: [Reqwest](https://docs.rs/reqwest/) HTTP client
- **AI Integration**: [OpenRouter API](https://openrouter.ai/) & [Ollama Local REST API](https://ollama.com/)

---

## 🚀 Installation & Setup

All you need is a standard Rust toolchain to get started.

### 1. Clone the repository
```bash
git clone https://github.com/take1dev/Domain-Suggester.git
cd Domain-Suggester
```

### 2. Build from Source
```bash
# Compile optimal release binary
cargo build --release
```

**Note for Windows Users:** The build process automatically bundles the `Icon.png` into the executable metadata utilizing the `winres` crate.

### 3. Run
You can launch the executable located in `target/release/domain-suggest.exe` or run directly using Cargo:
```bash
cargo run --release
```

---

## 🕹️ Quick Start Guide

1. **Configure Provider:** Navigate to the **⚙️ Settings** tab. Enter an optional but highly recommended [OpenRouter](https://openrouter.ai/) API key, or swap to **Ollama** if you have a local model running on `localhost:11434`.
2. **Select Scope:** In Settings, toggle which Top-Level Domains you want to target (e.g. `.com`, `.io`, `.ai`).
3. **Brainstorm:** Switch back to the **🧠 Generator** tab. Enter your core product keywords, industry format, and desired Brand Personality. 
4. **Deploy:** Click **Generate & Check**. The AI will stream optimized ideas directly back into the GUI, and the background worker thread will concurrently verify the real-world availability of every generated name without ever hanging the interface.

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!
Feel free to check the [issues page](https://github.com/take1dev/Domain-Suggester/issues) if you want to contribute.

## 📝 License

This project is licensed under the **MIT License**. See the `LICENSE` file for more details.
