mod css;
mod dom;
mod html;
mod js;
mod layout;
mod net;
mod paint;
mod render;
mod security;
mod style;
mod ui;

use css::{CssParser, CssTokenizer};
use dom::build_dom;
use html::Tokenizer;
use layout::{build_layout_tree, layout_tree};
use render::RenderState;
use style::compute_styles;

pub fn fetch_and_render_page(url_str: &str) -> (dom::Dom, Vec<paint::PaintCommand>) {
    let resolved_url = net::format_url_or_search_query(url_str);
    let is_live_page = resolved_url.starts_with("http://") || resolved_url.starts_with("https://");

    let html_content = if is_live_page {
        match net::fetch(&resolved_url) {
            Ok(resp) => resp.body_as_string(),
            Err(e) => format!("<html><body><h1>Fetch Error</h1><p>{}</p></body></html>", e),
        }
    } else {
        r#"<div class="page"><h1 class="title">Secure Browser Engine</h1><p>Type a search query or URL and press Enter to search!</p></div>"#.to_string()
    };

    let dom_tree = dom::build_dom(html::Tokenizer::new(&html_content).tokenize());

    // --- Batch A fix ---
    // Previously the CSS the page actually shipped (inline <style> tags and
    // <link rel="stylesheet"> files) was parsed into `_css_content` and then
    // silently discarded; every page was rendered against a single hardcoded
    // rule (`body { color: #333; }`). That meant no page's real styling ever
    // reached the screen no matter what Batches 2-4 implemented downstream.
    // We now collect real CSS from the page before building the stylesheet.
    let mut css_source = String::new();

    // 1. Inline <style> element contents.
    css_source.push_str(&collect_inline_styles(&dom_tree));

    // 2. External stylesheets discovered via <link rel="stylesheet">.
    if is_live_page {
        let discovered = net::discover_resources(&dom_tree, &resolved_url);
        for sheet_url in discovered.stylesheets {
            match net::fetch(&sheet_url) {
                Ok(resp) => {
                    css_source.push('\n');
                    css_source.push_str(&resp.body_as_string());
                }
                Err(e) => {
                    eprintln!("Warning: failed to fetch stylesheet {}: {}", sheet_url, e);
                }
            }
        }
    } else {
        css_source.push_str(r#".page { width: 400px; padding: 10px; } .title { color: #00f; }"#);
    }

    // Baseline so a page that ships zero CSS still isn't completely bare;
    // real page CSS is always concatenated *before* this so it always wins
    // under source-order rules for equal specificity.
    if css_source.trim().is_empty() {
        css_source.push_str("body { color: #333; }");
    }

    let stylesheet = css::CssParser::new(css::CssTokenizer::new(&css_source).tokenize()).parse();
    let styles = style::compute_styles(&dom_tree, &stylesheet);

    let display_list = if let Some(mut layout_root) =
        layout::build_layout_tree(&dom_tree, &styles, dom_tree.root)
    {
        layout::layout_tree(&mut layout_root, 800.0, &styles);
        paint::build_display_list(&layout_root, &styles)
    } else {
        vec![]
    };

    (dom_tree, display_list)
}

/// Walks the DOM collecting the text content of every `<style>` element,
/// in document order, joined with newlines so cascade source-order between
/// multiple `<style>` blocks is preserved.
fn collect_inline_styles(dom: &dom::Dom) -> String {
    let mut out = String::new();
    for node in &dom.nodes {
        if let dom::NodeType::Element(elem) = &node.node_type {
            if elem.tag.eq_ignore_ascii_case("style") {
                for &child_idx in &node.children {
                    if let dom::NodeType::Text(text) = &dom.nodes[child_idx].node_type {
                        out.push_str(text);
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

fn navigate_tab(
    tab_manager: &mut ui::TabManager,
    address_bar: &mut ui::AddressBar,
    window: &std::sync::Arc<winit::window::Window>,
    target_url: &str,
    is_new: bool,
    image_pipeline: &net::AsyncImagePipeline,
) {
    let resolved_url = net::format_url_or_search_query(target_url);
    address_bar.set_text(&resolved_url);

    let is_live_page = resolved_url.starts_with("http://") || resolved_url.starts_with("https://");
    let (title, body) = if is_live_page {
        match net::fetch(&resolved_url) {
            Ok(resp) => {
                let raw_body = resp.body_as_string();
                let mut temp_dom = dom::build_dom(html::Tokenizer::new(&raw_body).tokenize());
                let res = net::discover_resources(&temp_dom, &resolved_url);
                let external_css = net::ResourceLoader::fetch_all_stylesheets(&res.stylesheets);

                // Queue discovered image URLs for asynchronous background download
                for img_url in &res.images {
                    let win = std::sync::Arc::clone(window);
                    image_pipeline.queue_fetch(img_url, move || {
                        win.request_redraw();
                    });
                }

                // Execute embedded scripts with JsRuntime
                let scripts = js::JsRuntime::extract_scripts(&temp_dom);
                if !scripts.is_empty() {
                    let mut runtime = js::JsRuntime::new();
                    for s in &scripts {
                        let _ = runtime.execute(s, &mut temp_dom);
                    }
                }

                // Extract <title> if present
                let title_nodes = temp_dom.get_elements_by_tag_name("title");
                let extracted_title = title_nodes
                    .first()
                    .map(|&idx| temp_dom.text_content(idx).trim().to_string())
                    .filter(|t| !t.is_empty());

                let final_title = extracted_title.unwrap_or_else(|| resolved_url.clone());
                let final_body = if !external_css.is_empty() {
                    format!("<style>{}</style>{}", external_css, raw_body)
                } else {
                    raw_body
                };
                (final_title, final_body)
            }
            Err(err) => (
                "Fetch Error".to_string(),
                format!("<div class='card'><h2 style='color: red;'>Fetch Error</h2><p>{}</p></div>", err),
            ),
        }
    } else {
        (
            "Diaz's Secure Browser Engine".to_string(),
            r#"<div class="page"><h1 class="title">Secure Browser Engine</h1><p>Type a search query or URL and press Enter to search!</p></div>"#.to_string(),
        )
    };

    window.set_title(&format!("{} - Diaz's Secure Browser Engine", title));

    let tab = tab_manager.active_tab_mut();
    if is_new {
        tab.navigate_to(&title, &resolved_url, &body);
    } else {
        tab.title = title;
        tab.url = resolved_url;
        tab.page_body = body;
        tab.scroll_y = 0.0;
        tab.target_scroll_y = 0.0;
        tab.scroll_velocity = 0.0;
    }
    window.request_redraw();
}

#[cfg(not(test))]
fn main() {
    println!("=== Diaz's Secure Browser Engine ===");
    println!("Starting GPU-accelerated window...");

    use std::sync::Arc;
    use winit::{
        event::{Event, MouseScrollDelta, WindowEvent},
        event_loop::EventLoop,
        keyboard::{Key, NamedKey},
        window::WindowBuilder,
    };

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Diaz's Secure Browser Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(1024, 720))
            .build(&event_loop)
            .unwrap(),
    );

    let mut state = pollster::block_on(RenderState::new(window.clone()));
    let mut address_bar = ui::AddressBar::new("https://google.com");
    let mut tab_manager = ui::TabManager::new();
    let bookmark_manager = ui::BookmarkManager::new();
    let download_manager = net::DownloadManager::new();
    let mut context_menu = ui::ContextMenu::new();
    let mut devtools = ui::DevTools::new();
    let navbar = ui::NavBar::new();
    let image_pipeline = net::AsyncImagePipeline::new();
    let mut modifiers_state = winit::keyboard::ModifiersState::default();
    let mut mouse_pos = (0.0f32, 0.0f32);
    let mut current_layout_root: Option<layout::LayoutBox> = None;
    // Total rendered document height in pixels (updated each RedrawRequested).
    let mut document_height: f32 = 0.0;

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                        state.window.request_redraw();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        state.resize(state.window.inner_size());
                        state.window.request_redraw();
                    }
                    WindowEvent::ModifiersChanged(m) => {
                        modifiers_state = m.state();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_pos = (position.x as f32, position.y as f32);
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        // Convert raw wheel delta to pixels.
                        // LineDelta: multiply by a comfortable line-height (40 px/line).
                        // PixelDelta: use the physical y component directly.
                        let delta_y = match delta {
                            MouseScrollDelta::LineDelta(_, rows) => rows * 40.0,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };
                        let viewport_h = state.size.height as f32;
                        let max_scroll = (document_height - viewport_h).max(0.0);
                        tab_manager.active_tab_mut().apply_wheel_delta(delta_y, max_scroll);
                        state.window.request_redraw();
                    }
                    WindowEvent::MouseInput { state: element_state, button, .. } => {
                        if element_state == winit::event::ElementState::Pressed {
                            let (mx, my) = mouse_pos;
                            if button == winit::event::MouseButton::Right {
                                context_menu.show(mx, my);
                                state.window.request_redraw();
                            } else if button == winit::event::MouseButton::Left {
                                if context_menu.is_visible {
                                    if let Some(item) = context_menu.get_item_at_click(mx, my) {
                                        match item {
                                            "Back" => {
                                                if let Some(prev) = tab_manager.active_tab_mut().history.go_back() {
                                                    navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &prev, false, &image_pipeline);
                                                }
                                            }
                                            "Forward" => {
                                                if let Some(next) = tab_manager.active_tab_mut().history.go_forward() {
                                                    navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &next, false, &image_pipeline);
                                                }
                                            }
                                            "Reload" => {
                                                let target = tab_manager.active_tab().url.clone();
                                                println!("Reloading: {}", target);
                                                navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &target, false, &image_pipeline);
                                            }
                                            "Copy URL" => {
                                                println!("URL copied to clipboard: {}", tab_manager.active_tab().url);
                                            }
                                            "Inspect Element" => {
                                                devtools.open();
                                                println!("\n=== DEVTOOLS: Inspect Element Activated ===");
                                            }
                                            _ => {}
                                        }
                                    }
                                    context_menu.hide();
                                    state.window.request_redraw();
                                } else if let Some(close_idx) = tab_manager.get_close_click(mx, my) {
                                    println!("Closing tab: {}", close_idx);
                                    tab_manager.close_tab(close_idx);
                                    address_bar.set_text(&tab_manager.active_tab().url);
                                    state.window.set_title(&format!("{} - Diaz's Secure Browser Engine", tab_manager.active_tab().title));
                                    state.window.request_redraw();
                                } else if tab_manager.is_new_tab_click(mx, my) {
                                    println!("Opening new tab");
                                    tab_manager.new_tab("New Tab", "https://google.com");
                                    address_bar.set_text(&tab_manager.active_tab().url);
                                    state.window.set_title("New Tab - Diaz's Secure Browser Engine");
                                    state.window.request_redraw();
                                } else if let Some(tab_idx) = tab_manager.get_tab_at_click(mx, my) {
                                    tab_manager.switch_tab(tab_idx);
                                    address_bar.set_text(&tab_manager.active_tab().url);
                                    state.window.set_title(&format!("{} - Diaz's Secure Browser Engine", tab_manager.active_tab().title));
                                    state.window.request_redraw();
                                } else if let Some(action) = navbar.get_button_at_click(mx, my) {
                                    match action {
                                        ui::NavAction::Back => {
                                            if let Some(prev) = tab_manager.active_tab_mut().history.go_back() {
                                                println!("Back button clicked: {}", prev);
                                                navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &prev, false, &image_pipeline);
                                            }
                                        }
                                        ui::NavAction::Forward => {
                                            if let Some(next) = tab_manager.active_tab_mut().history.go_forward() {
                                                println!("Forward button clicked: {}", next);
                                                navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &next, false, &image_pipeline);
                                            }
                                        }
                                        ui::NavAction::Reload => {
                                            let current_url = tab_manager.active_tab().url.clone();
                                            println!("Reload button clicked: {}", current_url);
                                            navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &current_url, false, &image_pipeline);
                                        }
                                    }
                                } else if let Some(target_url) = bookmark_manager.get_url_at_click(mx, my) {
                                    println!("Bookmark clicked: {}", target_url);
                                    navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &target_url, true, &image_pipeline);
                                } else if my >= 80.0 && my <= 120.0 {
                                    address_bar.set_text("");
                                    state.window.request_redraw();
                                } else if my > 120.0 {
                                    // 2D Point hit-testing against hyperlink bounding boxes
                                    let content_y = my + tab_manager.active_tab().scroll_y;
                                    if let Some(ref ltree) = current_layout_root {
                                        if let Some(href) = ltree.hit_test_link(mx, content_y) {
                                            let current_base = tab_manager.active_tab().url.clone();
                                            let target_url = net::resolve_url(&current_base, href);
                                            println!("Hyperlink clicked: '{}' -> '{}'", href, target_url);
                                            navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &target_url, true, &image_pipeline);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        if key_event.state == winit::event::ElementState::Pressed {
                            match &key_event.logical_key {
                                Key::Named(NamedKey::ArrowDown) => {
                                    tab_manager.active_tab_mut().scroll_y += 25.0;
                                    state.window.request_redraw();
                                }
                                Key::Named(NamedKey::ArrowUp) => {
                                    let active = tab_manager.active_tab_mut();
                                    active.scroll_y = (active.scroll_y - 25.0).max(0.0);
                                    state.window.request_redraw();
                                }
                                Key::Named(NamedKey::Backspace) => {
                                    address_bar.backspace();
                                    state.window.request_redraw();
                                }
                                Key::Named(NamedKey::ArrowLeft) => {
                                    if let Some(prev_url) = tab_manager.active_tab_mut().history.go_back() {
                                        println!("Going back to: {}", prev_url);
                                        navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &prev_url, false, &image_pipeline);
                                    }
                                }
                                Key::Named(NamedKey::ArrowRight) => {
                                    if let Some(next_url) = tab_manager.active_tab_mut().history.go_forward() {
                                        println!("Going forward to: {}", next_url);
                                        navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &next_url, false, &image_pipeline);
                                    }
                                }
                                Key::Named(NamedKey::F5) => {
                                    let current_url = tab_manager.active_tab().url.clone();
                                    println!("Reloading (F5): {}", current_url);
                                    navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &current_url, false, &image_pipeline);
                                }
                                Key::Named(NamedKey::Enter) => {
                                    let target_url = address_bar.resolve_query();
                                    println!("Fetching: {}", target_url);
                                    navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &target_url, true, &image_pipeline);
                                }
                                Key::Named(NamedKey::Escape) => {
                                    address_bar.set_text("");
                                    state.window.request_redraw();
                                }
                                Key::Named(NamedKey::F12) => {
                                    devtools.toggle();
                                    println!("\n=== DEVTOOLS (F12) ===");
                                    println!("Status: is_open={}", devtools.is_open);
                                    println!("Active URL: {}", tab_manager.active_tab().url);
                                    println!("Active Title: {}", tab_manager.active_tab().title);
                                    state.window.request_redraw();
                                }
                                Key::Named(NamedKey::Space) => {
                                    address_bar.insert_char(' ');
                                    state.window.request_redraw();
                                }
                                Key::Character(ch_str) if modifiers_state.control_key() => {
                                    if ch_str.eq_ignore_ascii_case("r") {
                                        let current_url = tab_manager.active_tab().url.clone();
                                        println!("Reloading (Ctrl+R): {}", current_url);
                                        navigate_tab(&mut tab_manager, &mut address_bar, &state.window, &current_url, false, &image_pipeline);
                                    }
                                }
                                Key::Character(ch_str) if ch_str == "+" || ch_str == "=" => {
                                    tab_manager.active_tab_mut().zoom_in();
                                    println!("Zoom level: {:.0}%", tab_manager.active_tab().zoom_level * 100.0);
                                    state.window.request_redraw();
                                }
                                Key::Character(ch_str) if ch_str == "-" => {
                                    tab_manager.active_tab_mut().zoom_out();
                                    println!("Zoom level: {:.0}%", tab_manager.active_tab().zoom_level * 100.0);
                                    state.window.request_redraw();
                                }
                                Key::Character(ch_str) => {
                                    for c in ch_str.chars() {
                                        address_bar.insert_char(c);
                                    }
                                    state.window.request_redraw();
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        let tab_strip = tab_manager.render_tab_strip_html();
                        let bookmarks_bar = bookmark_manager.render_bookmarks_bar_html();
                        let downloads_bar = download_manager.render_downloads_bar_html();
                        let suggestions_bar = address_bar.render_suggestions_html();
                        let context_menu_html = context_menu.render_html();
                        let active_tab = tab_manager.active_tab();
                        let can_back = active_tab.history.can_go_back();
                        let can_forward = active_tab.history.can_go_forward();
                        let navbar_html = navbar.render_html(&address_bar.url_text, can_back, can_forward, &suggestions_bar);
                        let temp_content_dom = build_dom(Tokenizer::new(&active_tab.page_body).tokenize());
                        let devtools_html = devtools.render_html(&temp_content_dom, &active_tab.url);

                        let html_doc = format!(
                            r#"
                            <div class="page">
                                {}
                                {}
                                {}
                                <div class="content">
                                    {}
                                </div>
                                {}
                                {}
                                {}
                            </div>
                            "#,
                            tab_strip,
                            navbar_html,
                            bookmarks_bar,
                            active_tab.page_body,
                            downloads_bar,
                            context_menu_html,
                            devtools_html
                        );

                        let css_doc = r#"
                            .page { width: 780px; padding: 12px; background-color: #0f172a; }
                            .navbar { background-color: #1e293b; padding: 10px; border-width: 2px; border-color: #6366f1; margin-bottom: 8px; }
                            .nav-controls { margin-bottom: 6px; }
                            .nav-btn { font-size: 12px; font-weight: bold; padding: 4px 8px; margin-right: 6px; border-width: 1px; border-color: #475569; }
                            .nav-title { color: #818cf8; font-weight: bold; margin: 0; font-size: 13px; }
                            .address-input { background-color: #0f172a; color: #38bdf8; font-weight: bold; padding: 8px; margin: 4px 0 0 0; border-width: 2px; border-color: #38bdf8; }
                            .card { padding: 16px; background-color: #1e293b; border-width: 1px; border-color: #334155; margin-top: 8px; }
                            p { color: #cbd5e1; font-size: 15px; margin: 6px 0; }
                            h2 { color: #f8fafc; margin: 4px 0; }
                            .devtools-panel { background-color: #020617; border-width: 2px; border-color: #38bdf8; padding: 10px; margin-top: 14px; }
                            .devtools-title { color: #38bdf8; font-weight: bold; font-size: 13px; margin: 0 0 6px 0; }
                            .devtools-tabs { margin-bottom: 8px; }
                            .devtools-tab { color: #94a3b8; font-size: 12px; margin-right: 8px; font-weight: bold; }
                            .devtools-tab-active { color: #38bdf8; font-weight: bold; }
                            .devtools-subhead { color: #f8fafc; font-size: 12px; font-weight: bold; margin: 4px 0; }
                            .devtools-code-box { background-color: #0f172a; padding: 8px; border-width: 1px; border-color: #1e293b; margin: 4px 0; }
                            .devtools-node { color: #e2e8f0; font-size: 11px; margin: 2px 0; }
                            .devtools-tag { color: #f43f5e; font-weight: bold; }
                            .devtools-attr { color: #facc15; }
                            .devtools-text-node { color: #4ade80; font-size: 11px; margin: 1px 0; }
                            .devtools-comment { color: #64748b; font-size: 11px; margin: 1px 0; }
                            .devtools-metric-label { color: #a855f7; font-size: 11px; margin-top: 6px; }
                            .devtools-console-box { background-color: #0f172a; padding: 8px; border-width: 1px; border-color: #1e293b; }
                            .devtools-console-entry { color: #38bdf8; font-size: 11px; margin: 2px 0; }
                            .devtools-network { margin-top: 4px; }
                            .devtools-net-box { background-color: #0f172a; padding: 8px; border-width: 1px; border-color: #1e293b; }
                            .devtools-net-entry { color: #22c55e; font-size: 11px; margin: 2px 0; }
                            .devtools-perf { margin-top: 4px; }
                            .devtools-perf-item { color: #cbd5e1; font-size: 12px; margin: 3px 0; }
                        "#;

                        let dom_tree = build_dom(Tokenizer::new(&html_doc).tokenize());
                        let mut stylesheet = CssParser::new(CssTokenizer::new(css_doc).tokenize()).parse();

                        // Merge embedded <style> blocks from the loaded page into stylesheet cascade
                        for node in &dom_tree.nodes {
                            if let dom::NodeType::Element(elem) = &node.node_type {
                                if elem.tag.to_ascii_lowercase() == "style" {
                                    for child_idx in &node.children {
                                        if let dom::NodeType::Text(css_text) = &dom_tree.nodes[*child_idx].node_type {
                                            let parsed_sheet = CssParser::new(CssTokenizer::new(css_text).tokenize()).parse();
                                            stylesheet.rules.extend(parsed_sheet.rules);
                                        }
                                    }
                                }
                            }
                        }

                        let styles = compute_styles(&dom_tree, &stylesheet);

                        if let Some(mut layout_root) = build_layout_tree(&dom_tree, &styles, dom_tree.root) {
                            let viewport_width = state.size.width as f32;
                            layout_tree(&mut layout_root, viewport_width, &styles);

                            // Update document height, clamp scroll, and advance the
                            // smooth-scroll animation.
                            document_height = layout_root.total_document_height();
                            let viewport_h = state.size.height as f32;
                            let max_scroll = (document_height - viewport_h).max(0.0);
                            {
                                let tab = tab_manager.active_tab_mut();
                                tab.clamp_scroll(max_scroll);
                                if tab.tick_scroll() {
                                    state.window.request_redraw();
                                }
                            }
                            let paint_commands = paint::build_display_list(&layout_root, &styles);

                            // Queue any un-cached images for asynchronous fetch
                            for cmd in &paint_commands {
                                if let paint::PaintCommand::DrawImage { src, .. } = cmd {
                                    if !image_pipeline.is_cached(src) {
                                        let win = state.window.clone();
                                        image_pipeline.queue_fetch(src, move || {
                                            win.request_redraw();
                                        });
                                    }
                                }
                            }

                            current_layout_root = Some(layout_root);
                            let scroll_y = tab_manager.active_tab().scroll_y;
                            let img_cache = image_pipeline.cache.lock().unwrap();
                            if let Err(e) = state.render_with_cache(&paint_commands, scroll_y, Some(&img_cache)) {
                                eprintln!("Render error: {:?}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
}

fn print_layout_box(lb: &layout::LayoutBox, depth: usize) {
    let indent = "  ".repeat(depth);
    let d = lb.dimensions.content;
    match lb.box_type {
        layout::BoxType::Block(node_idx) => {
            println!(
                "{indent}Block(node {node_idx}) x={:.0} y={:.0} w={:.0} h={:.0}",
                d.x, d.y, d.width, d.height
            );
        }
        layout::BoxType::Anonymous => {
            println!(
                "{indent}Text(\"{}\") x={:.0} y={:.0} w={:.0} h={:.0}",
                lb.text_content.as_deref().unwrap_or(""),
                d.x,
                d.y,
                d.width,
                d.height
            );
        }
    }
    for child in &lb.children {
        print_layout_box(child, depth + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::html::{Token, Tokenizer};

    #[test]
    fn tokenizes_simple_tag_with_text() {
        let tokens = Tokenizer::new("<p>hello</p>").tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::StartTag {
                    name: "p".into(),
                    attributes: vec![],
                    self_closing: false
                },
                Token::Text("hello".into()),
                Token::EndTag { name: "p".into() },
                Token::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_attributes_quoted_and_unquoted() {
        let tokens = Tokenizer::new(r#"<a href="x.html" target=_blank>go</a>"#).tokenize();
        match &tokens[0] {
            Token::StartTag {
                name, attributes, ..
            } => {
                assert_eq!(name, "a");
                assert_eq!(
                    attributes,
                    &vec![
                        ("href".to_string(), "x.html".to_string()),
                        ("target".to_string(), "_blank".to_string()),
                    ]
                );
            }
            other => panic!("expected StartTag, got {:?}", other),
        }
    }

    #[test]
    fn tokenizes_self_closing_tag() {
        let tokens = Tokenizer::new("<br/>").tokenize();
        assert_eq!(
            tokens[0],
            Token::StartTag {
                name: "br".into(),
                attributes: vec![],
                self_closing: true
            }
        );
    }

    #[test]
    fn tokenizes_comment() {
        let tokens = Tokenizer::new("<!-- hi -->").tokenize();
        assert_eq!(tokens[0], Token::Comment(" hi ".into()));
    }

    #[test]
    fn tokenizes_doctype() {
        let tokens = Tokenizer::new("<!DOCTYPE html>").tokenize();
        assert_eq!(
            tokens[0],
            Token::Doctype {
                name: Some("html".into())
            }
        );
    }

    #[test]
    fn preserves_bengali_text() {
        let tokens = Tokenizer::new("<p>School</p>").tokenize();
        assert_eq!(tokens[1], Token::Text("School".into()));
    }

    #[test]
    fn never_panics_on_malformed_input() {
        // this must not panic -- that's the whole point of the security requirement
        let inputs = [
            "<div",
            "< >",
            "</>",
            "<a href=",
            "<a href=\"unterminated",
            "<!--unterminated comment",
            "<!weird",
        ];
        for input in inputs {
            let _ = Tokenizer::new(input).tokenize();
        }
    }
}

#[cfg(test)]
mod css_tests {
    use super::css::{CssParser, CssTokenizer};

    fn parse(src: &str) -> super::css::Stylesheet {
        let tokens = CssTokenizer::new(src).tokenize();
        CssParser::new(tokens).parse()
    }

    #[test]
    fn parses_simple_rule() {
        let sheet = parse("p { color: red; }");
        assert_eq!(sheet.rules.len(), 1);
        let rule = &sheet.rules[0];
        assert_eq!(rule.selectors[0].steps[0].1.tag.as_deref(), Some("p"));
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, "red");
    }

    #[test]
    fn parses_class_and_id_selectors() {
        let sheet = parse("div.card#main { width: 100%; }");
        let simple = &sheet.rules[0].selectors[0].steps[0].1;
        assert_eq!(simple.tag.as_deref(), Some("div"));
        assert_eq!(simple.classes, vec!["card".to_string()]);
        assert_eq!(simple.id.as_deref(), Some("main"));
    }

    #[test]
    fn parses_descendant_and_child_combinators() {
        let sheet = parse("div p { color: blue; } div > span { color: green; }");
        let descendant_selector = &sheet.rules[0].selectors[0];
        assert_eq!(descendant_selector.steps.len(), 2);
        assert_eq!(
            descendant_selector.steps[1].0,
            Some(super::css::Combinator::Descendant)
        );

        let child_selector = &sheet.rules[1].selectors[0];
        assert_eq!(
            child_selector.steps[1].0,
            Some(super::css::Combinator::Child)
        );
    }

    #[test]
    fn parses_multiple_selectors_in_list() {
        let sheet = parse("h1, h2, .title { margin: 0; }");
        assert_eq!(sheet.rules[0].selectors.len(), 3);
    }

    #[test]
    fn detects_important() {
        let sheet = parse("p { color: red !important; }");
        assert!(sheet.rules[0].declarations[0].important);
    }

    #[test]
    fn computes_specificity() {
        // #id (1,0,0) beats .class (0,1,0) beats type (0,0,1)
        let sheet = parse("#a { } .b { } div { }");
        assert_eq!(sheet.rules[0].selectors[0].specificity(), (1, 0, 0));
        assert_eq!(sheet.rules[1].selectors[0].specificity(), (0, 1, 0));
        assert_eq!(sheet.rules[2].selectors[0].specificity(), (0, 0, 1));
    }

    #[test]
    fn skips_at_rules_without_crashing() {
        let sheet = parse("@media (max-width: 600px) { .card { width: 100%; } } p { color: red; }");
        // the @media block is skipped wholesale in this batch; only the
        // trailing rule outside it should show up
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
    }

    #[test]
    fn parses_pseudo_classes() {
        let sheet = parse("#header, .nav a:hover { color: blue; } p:first-child { margin: 0; }");
        assert_eq!(sheet.rules.len(), 2);
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
        let hover_sel = &sheet.rules[0].selectors[1];
        assert_eq!(hover_sel.steps[1].1.tag.as_deref(), Some("a"));
        assert_eq!(
            hover_sel.steps[1].1.pseudo_classes,
            vec![super::css::PseudoClass::Hover]
        );
        // Specificity of `.nav a:hover`: 1 class (.nav) + 1 pseudo-class (:hover) + 1 tag (a) = (0, 2, 1)
        assert_eq!(hover_sel.specificity(), (0, 2, 1));
    }

    #[test]
    fn malformed_selector_drops_whole_rule_cleanly() {
        let sheet = parse("div [unsupported] { color: blue; } p { margin: 0; }");
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "margin");
    }

    #[test]
    fn never_panics_on_malformed_css() {
        let inputs = [
            "div { color",
            "div { : red; }",
            "div { color: ; }",
            "} } } div { color: red; }",
            ". { color: red; }",
            "div >",
            "@weird",
        ];
        for input in inputs {
            let _ = parse(input);
        }
    }
}

#[cfg(test)]
mod batch3_tests {
    use super::css::{CssParser, CssTokenizer};
    use super::dom::build_dom;
    use super::html::Tokenizer;
    use super::layout::{build_layout_tree, layout_tree, BoxType};
    use super::style::compute_styles;

    fn build(html: &str, css: &str) -> (super::dom::Dom, super::css::Stylesheet) {
        let dom = build_dom(Tokenizer::new(html).tokenize());
        let sheet = CssParser::new(CssTokenizer::new(css).tokenize()).parse();
        (dom, sheet)
    }

    #[test]
    fn dom_builder_creates_correct_tree_shape() {
        let dom = build_dom(Tokenizer::new("<div><p>hi</p><p>bye</p></div>").tokenize());
        let div_idx = dom.nodes[dom.root].children[0];
        let div = dom.element_at(div_idx).unwrap();
        assert_eq!(div.tag, "div");
        assert_eq!(dom.nodes[div_idx].children.len(), 2);
    }

    #[test]
    fn dom_builder_recovers_from_mismatched_tags() {
        // </span> with no matching open <span> must not panic or corrupt the tree
        let dom = build_dom(Tokenizer::new("<div>text</span><p>ok</p></div>").tokenize());
        let div_idx = dom.nodes[dom.root].children[0];
        // both the stray text and the <p> should still be children of <div>
        assert_eq!(dom.nodes[div_idx].children.len(), 2);
    }

    #[test]
    fn dom_builder_handles_void_elements_without_consuming_stack() {
        let dom = build_dom(Tokenizer::new("<div><br><p>after br</p></div>").tokenize());
        let div_idx = dom.nodes[dom.root].children[0];
        // <br> should not have swallowed <p> as its child
        assert_eq!(dom.nodes[div_idx].children.len(), 2);
        let br_idx = dom.nodes[div_idx].children[0];
        assert_eq!(dom.nodes[br_idx].children.len(), 0);
    }

    #[test]
    fn style_matches_tag_class_and_id_selectors() {
        let (dom, sheet) = build(
            r#"<div id="main" class="card"><p>hi</p></div>"#,
            "#main { color: red; } .card { padding: 5px; } p { margin: 1px; }",
        );
        let styles = compute_styles(&dom, &sheet);
        let div_idx = dom.nodes[dom.root].children[0];
        let div_style = &styles[&div_idx];
        assert_eq!(div_style.get("color"), Some(&"red".to_string()));
        assert_eq!(div_style.get("padding"), Some(&"5px".to_string()));
    }

    #[test]
    fn style_cascade_respects_specificity_over_source_order() {
        // .a comes later in source but #id has higher specificity and must win
        let (dom, sheet) = build(
            r#"<div id="x" class="a"></div>"#,
            "#x { color: blue; } .a { color: red; }",
        );
        let styles = compute_styles(&dom, &sheet);
        let div_idx = dom.nodes[dom.root].children[0];
        assert_eq!(styles[&div_idx].get("color"), Some(&"blue".to_string()));
    }

    #[test]
    fn style_important_beats_higher_specificity() {
        let (dom, sheet) = build(
            r#"<div id="x" class="a"></div>"#,
            "#x { color: blue; } .a { color: red !important; }",
        );
        let styles = compute_styles(&dom, &sheet);
        let div_idx = dom.nodes[dom.root].children[0];
        assert_eq!(styles[&div_idx].get("color"), Some(&"red".to_string()));
    }

    #[test]
    fn style_inherits_color_from_parent() {
        let (dom, sheet) = build(r#"<div><p>hi</p></div>"#, "div { color: green; }");
        let styles = compute_styles(&dom, &sheet);
        let div_idx = dom.nodes[dom.root].children[0];
        let p_idx = dom.nodes[div_idx].children[0];
        assert_eq!(styles[&p_idx].get("color"), Some(&"green".to_string()));
    }

    #[test]
    fn style_child_combinator_matches_direct_child_only() {
        let (dom, sheet) = build(
            r#"<div><span><em>deep</em></span><em>direct</em></div>"#,
            "div > em { color: red; }",
        );
        let styles = compute_styles(&dom, &sheet);
        let div_idx = dom.nodes[dom.root].children[0];
        let span_idx = dom.nodes[div_idx].children[0];
        let deep_em_idx = dom.nodes[span_idx].children[0];
        let direct_em_idx = dom.nodes[div_idx].children[1];
        assert!(styles
            .get(&deep_em_idx)
            .and_then(|s| s.get("color"))
            .is_none());
        assert_eq!(
            styles.get(&direct_em_idx).and_then(|s| s.get("color")),
            Some(&"red".to_string())
        );
    }

    #[test]
    fn layout_block_boxes_stack_vertically_and_fill_width() {
        let (dom, sheet) = build(
            r#"<div class="page"><p>one</p><p>two</p></div>"#,
            ".page { width: 300px; } p { height: 20px; margin: 5px 0; }",
        );
        let styles = compute_styles(&dom, &sheet);
        let mut root = build_layout_tree(&dom, &styles, dom.root).unwrap();
        layout_tree(&mut root, 800.0, &styles);

        let page_box = &root.children[0];
        assert_eq!(page_box.dimensions.content.width, 300.0);

        let p1 = &page_box.children[0];
        let p2 = &page_box.children[1];
        // p2 must start below p1's margin box (5px margin + 20px height + 5px margin)
        assert!(p2.dimensions.content.y > p1.dimensions.content.y);
        assert_eq!(p1.dimensions.content.height, 20.0);
    }

    #[test]
    fn layout_display_none_excludes_node_and_children() {
        let (dom, sheet) = build(
            r#"<div><p class="hidden">gone</p><p>visible</p></div>"#,
            ".hidden { display: none; }",
        );
        let styles = compute_styles(&dom, &sheet);
        let root = build_layout_tree(&dom, &styles, dom.root).unwrap();
        let div_box = &root.children[0];
        // only the visible <p> should have produced a layout box
        assert_eq!(div_box.children.len(), 1);
        assert!(matches!(div_box.children[0].box_type, BoxType::Block(_)));
    }

    #[test]
    fn never_panics_on_malformed_html_css_combo() {
        let combos = [
            ("<div", "div { color: red"),
            ("<div><p></div>", ". { }"),
            ("<div class=x>text<span>", "div > { color: red; }"),
        ];
        for (html, css) in combos {
            let (dom, sheet) = build(html, css);
            let styles = compute_styles(&dom, &sheet);
            if let Some(mut root) = build_layout_tree(&dom, &styles, dom.root) {
                layout_tree(&mut root, 800.0, &styles);
            }
        }
    }
}
