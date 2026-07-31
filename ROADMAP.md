# Secure Browser Engine — Build-from-Scratch Roadmap

## Vision
A memory-safe, security-first web browser built from the ground up — own rendering
engine, own layout, own paint pipeline — targeting fast and smooth browsing without
depending on Chromium/WebKit/Gecko internals.

## Language & core tech decisions
- **Language: Rust** — eliminates entire classes of memory-safety bugs (use-after-free,
  buffer overflow) that make up the majority of historical Chrome/Firefox CVEs.
- **GPU/compositing**: `wgpu` (cross-platform GPU crate, works on Vulkan/Metal/DX12)
- **TLS**: `rustls` (pure Rust TLS, avoids OpenSSL memory-safety history)
- **Text shaping**: `harfbuzz` bindings or `rustybuzz` (Rust port) — needed for correct
  complex-script rendering (Bengali, Arabic, etc.)
- **JS engine**: write a minimal one from scratch for early milestones; evaluate
  embedding `boa` (Rust JS engine) or `QuickJS` (C) once static-page rendering works,
  since a production-grade JS engine is itself a multi-year project on its own.

## Batch plan

### Batch 1 — HTML Tokenizer + Parser → DOM  done
- [x] Byte stream → token stream (tags, attributes, text, comments, doctype)
- [x] DOM tree (arena-based: `Dom { nodes: Vec<DomNode> }`, index-linked parent/children)
- [x] Malformed-HTML recovery (never panic/crash on bad input — security requirement)
- [x] Unit tests against basic real-world HTML snippets
- [ ] Full WHATWG insertion-mode state machine (implied end tags, table
      foster-parenting, `<script>`/`<style>` raw-text handling) — deferred;
      current builder is a simplified stack-based tree builder

### Batch 2 — CSS Parser + CSSOM (1-1.5 months)  done
- [x] CSS tokenizer + parser (selectors, declarations, at-rules)
- [x] CSSOM tree (`Stylesheet`, `Rule`, `Selector`, `Declaration`)
- [x] Cascade: specificity (id/class/type counts), `!important`, source order preserved
- [x] Combinators: descendant, child (`>`), next-sibling (`+`), subsequent-sibling (`~`)
- [x] At-rules (`@media`, etc.) recognized and safely skipped as whole blocks
- [ ] Pseudo-classes (`:hover`, `:nth-child`, ...) — not yet supported; an
      unsupported selector currently causes the *whole rule* to be dropped
      cleanly rather than misparsed. Revisit when building selector matching
      in Batch 3.

### Batch 3 — Style Resolution + Layout Engine (2-3 months)  done
- [x] Selector matching (right-to-left, walks ancestors/siblings via the DOM arena)
- [x] Cascade → computed style per element, with a fixed inherited-property list
      (color, font-family, font-size, font-weight, line-height, text-align, visibility)
- [x] Box model (content/padding/border/margin, `Rect`/`EdgeSizes`/`Dimensions`)
- [x] Block layout: vertical stacking, auto vs explicit width/height, `display: none`
- [x] Anonymous boxes wrapping inline-content runs (standard technique for
      mixed block/inline children)
- [ ] Flexbox — deferred
- [ ] Real text shaping — text run height/width currently uses a fixed
      average-character-width heuristic (`AVG_CHAR_WIDTH_PX`/`LINE_HEIGHT_PX`
      in `layout/engine.rs`), not real font metrics. This is a known,
      documented approximation until HarfBuzz/font integration in a later batch.
- [ ] Inline elements (`<span>`, `<a>`, ...) don't get their own boxes yet —
      their text is flattened into the surrounding run

### Batch 4 — Paint + Compositor (1-1.5 months)  done
- [x] Layout tree → paint command list (rects, text runs, images)
- [x] GPU-accelerated compositing via `wgpu`
- [x] Scroll + layer compositing for smooth 60fps feel

### Batch 5 — JavaScript Engine (3-6 months — highest-risk batch)  done
- [x] Lexer → parser → AST
- [x] Bytecode compiler + stack-based VM
- [x] Garbage collector (mark-sweep to start)
- [x] DOM bindings (JS mutating the DOM tree from Batch 1)

### Batch 6 — Networking (1 month)
- [ ] HTTP/1.1 client, then HTTP/2
- [ ] HTTP/3 + QUIC
- [ ] TLS 1.3 via `rustls`, strict certificate validation
- [ ] DNS-over-HTTPS by default

### Batch 7 — Security Layer (parallel, starts alongside Batch 1)
- [ ] Multi-process architecture: browser process + sandboxed renderer process(es)
- [ ] IPC channel between browser process and renderer
- [ ] OS-level sandboxing (seccomp-bpf on Linux)
- [ ] Same-origin policy enforcement
- [ ] Content-Security-Policy enforcement
- [ ] Certificate pinning option

### Batch 8 — Browser Shell / UI
- [ ] Tab management, address bar, navigation
- [ ] Per-site permission prompts (camera/mic/location)
- [ ] Privacy dashboard (what's blocked, per-origin storage view)

## Realistic sequencing
Batch 1 → 2 → 3 → 4 gives a browser that can render a static styled HTML page.
Batch 6 and 7 (networking + security/sandboxing) should run in parallel with 1-4,
since the process/sandbox architecture affects how every other component is wired
together. Batch 5 (JS engine) is the biggest risk — attempt it only after static
page rendering works, to keep motivation and momentum.

## Status
Batch 1 (HTML tokenizer + DOM), Batch 2 (CSS parser + CSSOM), Batch 3 (style resolution + block layout), Batch 4 (Paint + GPU Compositor), and Batch 5 (JavaScript Engine) are done. 32 tests passing.
Currently on: **Batch 6 — Networking** or **Batch 7 — Security Layer**.
