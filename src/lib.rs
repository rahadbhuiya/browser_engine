//! # browser_engine
//!
//! A modular, GPU-accelerated, security-first web browser engine built from scratch in Rust.
//! Features custom HTML tokenizer/parser, arena-based DOM tree, CSS parser & CSSOM,
//! layout engine with box-model, custom JavaScript bytecode VM, TLS 1.3 networking client,
//! multi-process IPC security sandbox, and hardware-accelerated `wgpu` compositor.

pub mod css;
pub mod dom;
pub mod html;
pub mod js;
pub mod layout;
pub mod net;
pub mod paint;
pub mod render;
pub mod security;
pub mod style;
pub mod ui;

pub use css::{CssParser, CssTokenizer};
pub use dom::build_dom;
pub use html::Tokenizer;
pub use layout::{build_layout_tree, layout_tree};
pub use style::compute_styles;
