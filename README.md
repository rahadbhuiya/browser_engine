# Browser Engine

A modular, security-first web browser engine built from scratch in Rust.

## Features

- **DOM & HTML Parser**: Arena-based DOM tree representation with a resilient HTML tokenizer.
- **CSS Engine**: CSSOM generation, cascade resolution, specificity calculation, and selector matching.
- **Layout & Style Engine**: Box model calculation (margin, border, padding, content) with block formatting contexts.
- **GPU Compositor**: Modern graphics rendering pipeline powered by `wgpu`.
- **JavaScript Engine**: Minimal bytecode compiler and stack-based virtual machine with mark-sweep garbage collection.
- **Network Stack**: Native HTTP/1.1 client with TLS 1.3 encryption using `rustls`.
- **Security Architecture**: Multi-process design separating UI and sandboxed renderers with IPC.

## Tech Stack

- **Language**: Rust (Edition 2021)
- **Graphics**: `wgpu`, `winit`, `wgpu_glyph`
- **Crypto & Networking**: `rustls`, `webpki-roots`, `url`
- **Image Processing**: `image`

## Getting Started

### Prerequisites

Ensure you have Rust and Cargo installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/rahadbhuiya/browser_engine.git
cd browser_engine

# Run test suite
cargo test

# Build and run in release mode
cargo run --release
```

## License

MIT
