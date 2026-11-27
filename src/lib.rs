//! # browser_engine
//!
//! A modular, GPU-accelerated, security-first web browser engine built from scratch in Rust.
//! Features custom HTML tokenizer/parser, arena-based DOM tree, CSS parser & CSSOM,
//! layout engine with box-model, custom JavaScript bytecode VM, TLS 1.3 networking client,
//! multi-process IPC security sandbox, hardware-accelerated `wgpu` compositor,
//! interactive form submission controls, W3C DOM event bubbling, and audio/video streaming pipeline.

pub mod css;
pub mod dom;
pub mod html;
pub mod js;
pub mod layout;
pub mod media;
pub mod net;
pub mod paint;
pub mod render;
pub mod security;
pub mod style;
pub mod ui;

pub use css::{CssParser, CssTokenizer};
pub use dom::{build_dom, Dom, DomEvent, EventRegistry, FormManager};
pub use html::Tokenizer;
pub use layout::{build_layout_tree, layout_tree};
pub use media::{MediaControls, MediaElementState, MediaManager, MediaType, PlaybackState};
pub use style::{compute_styles, compute_styles_with_viewport};
