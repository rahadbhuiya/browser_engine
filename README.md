# Browser Engine

[![CI](https://github.com/rahadbhuiya/browser_engine/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/rahadbhuiya/browser_engine/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Language: Rust 2021](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Graphics: wgpu](https://img.shields.io/badge/Graphics-wgpu-purple.svg)](https://wgpu.rs/)
[![Security: rustls](https://img.shields.io/badge/Security-rustls-green.svg)](https://github.com/rustls/rustls)

A modular, GPU-accelerated, security-first web browser engine built completely from scratch in pure Rust.

The engine does not rely on Chromium, WebKit, or Gecko internals. It implements its own HTML tokenizer, arena-based DOM tree, CSSOM cascade resolver, box-model layout calculator, hardware-accelerated paint pipeline via `wgpu`, custom stack-based JavaScript bytecode VM, native TLS 1.3 networking, and multi-process security architecture.

---

## Architectural Pipeline

```
[ Raw Network Stream / File Input ]
                 |
                 v
   +-----------------------------+         +-----------------------------+
   |  HTML5 Tokenizer & Builder  |         |   CSS Tokenizer & Parser    |
   +--------------+--------------+         +--------------+--------------+
                  |                                       |
                  v                                       v
         [ Arena DOM Tree ]                      [ CSSOM Stylesheet ]
                  \                                       /
                   \                                     /
                    v                                   v
          +-------------------------------------------------------+
          |      Style Resolution & Specificity Matching          |
          |       - Element Selector Matching (ID/Class/Type)     |
          |       - Cascade Resolution & Computed Property Map    |
          +---------------------------+---------------------------+
                                      |
                                      v
          +-------------------------------------------------------+
          |            Box Model Layout Calculation               |
          |       - Content, Padding, Border, Margin Rects        |
          |       - Block Formatting Context & Inline Runs        |
          +---------------------------+---------------------------+
                                      |
                                      v
          +-------------------------------------------------------+
          |            Display List Paint Engine                  |
          |       - Draw Rectangles, Border Lines, Text Glyphs    |
          |       - Decoded Texture Images (PNG / JPEG)           |
          +---------------------------+---------------------------+
                                      |
                                      v
          +-------------------------------------------------------+
          |          GPU Hardware Compositor (wgpu)               |
          |       - Low-level Vulkan / Metal / DirectX 12         |
          |       - Vertex Buffers, Layer Blending & Viewport     |
          +---------------------------+---------------------------+
                                      |
                                      v
                           [ Native Window Frame ]
```

---

## Engine Subsystems

### 1. HTML Tokenizer & Arena-Based DOM
- **Resilient Parsing:** Resilient tokenizer handling malformed HTML tags, unclosed attributes, comments, and doctypes without panicking.
- **Arena Tree Structure:** The DOM is represented as an index-linked memory arena (`Vec<DomNode>`), eliminating raw cyclic reference-counting overhead and guaranteeing cache-locality.

### 2. CSSOM & Specificity Cascade
- **Selector Matching:** Right-to-left evaluation supporting type, class, ID selectors, and combinators (`>`, `+`, `~`, descendant).
- **Cascade Specificity:** Strict specificity tuple tracking `(id_count, class_count, type_count)` honoring `!important` declarations, stylesheet origin, and source order.

### 3. Box Model Layout Engine
- **CSS Box Model:** Computes exact coordinates for content, padding, border, and margin geometry.
- **Formatting Contexts:** Supports vertical block stacking, explicit and auto dimensions, overflow wrapping, and `display: none` pruning.

### 4. GPU-Accelerated Hardware Compositor
- **Direct GPU Rendering:** Powered by `wgpu` targeting cross-platform native graphics APIs (DirectX 12, Vulkan, Metal).
- **Text & Texture Pipeline:** Hardware-accelerated glyph caching via `wgpu_glyph` and native image texture decoders (PNG and JPEG).

### 5. Custom JavaScript Bytecode VM
- **Compiler & Bytecode:** Custom lexer, Pratt parser, and compiler translating ECMAScript subsets into compact bytecode instructions.
- **Stack-Based Execution:** Fast evaluation loop with local variables, arithmetic, closures, and branching.
- **Mark-Sweep Garbage Collection:** Fully functional cycle-detecting mark-sweep GC managing object allocations.
- **DOM Bindings:** Direct bidirectional bindings allowing JavaScript to manipulate the DOM tree dynamically.

### 6. Native Networking & TLS 1.3
- **Pure Rust Cryptography:** HTTP/1.1 client with modern TLS 1.3 encryption using `rustls` and `webpki-roots`.
- **Resource Discovery:** Discovers and fetches external stylesheets (`<link rel="stylesheet">`) and remote assets automatically.

### 7. Security Architecture & Sandboxing
- **Multi-Process Model:** Clean separation between the privileged UI browser process and sandboxed renderer instances communicating over IPC channels.
- **Policy Enforcement:** Strict Same-Origin Policy (SOP) and Content-Security-Policy (CSP) headers verification.
- **Transactional Storage:** Origin-isolated IndexedDB transaction manager and session cookie jar.

### 8. Native Browser Shell UI
- **Multi-Tab Architecture:** Dynamic tab creation, switching, and closing.
- **Omnibox Navigation:** URL normalization and search engine routing with interactive suggestion dropdowns.
- **Right-Click Context Menu:** Native contextual operations: Back, Forward, Reload, Copy Page URL, and Inspect Element.

### 9. Built-in DevTools & DOM Inspector
- **Dock Panel [F12]:** Hardware-rendered diagnostic overlay supporting live tabbed panels: `Elements`, `Console`, `Network`, and `Performance`.
- **Live DOM Inspector:** Recursive tree inspection with tag/attribute syntax highlighting, node IDs, and computed CSS box model dimensions.
- **Diagnostic Logging:** Direct integration with JavaScript runtime execution logs and network waterfall requests.

---

## Keyboard Shortcuts & Controls

| Shortcut / Action | Description |
|-------------------|-------------|
| `F12` | Toggle built-in DevTools Dock Panel (DOM Inspector, Console, Network) |
| `Ctrl + T` | Open a new browser tab |
| `Ctrl + W` | Close the active browser tab |
| `Ctrl + R` / `F5` | Reload the current page |
| `Ctrl + L` | Focus the Omnibox address bar |
| `Ctrl + +` | Zoom in page content |
| `Ctrl + -` | Zoom out page content |
| `Right-Click` | Open Context Menu (Back, Forward, Reload, Copy URL, Inspect Element) |
| `Scroll Wheel` | Smooth vertical page scrolling |

---

## Getting Started

### Prerequisites

Install the standard Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Linux systems, install graphics and windowing libraries:

```bash
# Ubuntu / Debian
sudo apt-get install -y libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxkbcommon-dev libwayland-dev
```

### Build & Run

```bash
# Clone the repository
git clone https://github.com/rahadbhuiya/browser_engine.git
cd browser_engine

# Run the test suite (45 unit tests covering DOM, CSSOM, Layout, JS VM, and Security)
cargo test

# Build and run the browser in release mode
cargo run --release
```

---

## Authors & Acknowledgments

- **Rahad Bhuiya** ([@rahadbhuiya](https://github.com/rahadbhuiya))
  - Core Systems & Linux Kernel Engineer
  - Author of `xdp-guard` and `Exploidus OS`
  - Upstream contributor to `aya-rs/aya` and `sched-ext/scx`

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
