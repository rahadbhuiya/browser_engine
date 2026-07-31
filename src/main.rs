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
use style::compute_styles;
use render::RenderState;



pub fn fetch_and_render_page(url_str: &str) -> (dom::Dom, Vec<paint::PaintCommand>) {
    let (html_content, _css_content) = if url_str.starts_with("http://") || url_str.starts_with("https://") {
        match net::fetch(url_str) {
            Ok(resp) => (resp.body_as_string(), "".to_string()),
            Err(e) => (format!("<html><body><h1>Fetch Error</h1><p>{}</p></body></html>", e), "".to_string()),
        }
    } else {
        (
            r#"<div class="page"><h1 class="title">Diaz's Secure Browser</h1><p>Type a URL and press Enter to load live pages!</p></div>"#.to_string(),
            r#".page { width: 400px; padding: 10px; } .title { color: #00f; }"#.to_string(),
        )
    };

    let dom_tree = dom::build_dom(html::Tokenizer::new(&html_content).tokenize());
    let stylesheet = css::CssParser::new(css::CssTokenizer::new("body { color: #333; }").tokenize()).parse();
    let styles = style::compute_styles(&dom_tree, &stylesheet);

    let display_list = if let Some(mut layout_root) = layout::build_layout_tree(&dom_tree, &styles, dom_tree.root) {
        layout::layout_tree(&mut layout_root, 800.0, &styles);
        paint::build_display_list(&layout_root, &styles)
    } else {
        vec![]
    };

    (dom_tree, display_list)
}

#[cfg(not(test))]
fn main() {


    let samples = [
        r#"<!DOCTYPE html><html><body><h1 class="title">Hi Diaz</h1><p>where are you sonali</p><!-- comment --><br/></body></html>"#,
        // malformed input on purpose -- tokenizer must not panic
        r#"<div class=unclosed <p>broken <>< / ></div"#,
    ];

    for (i, sample) in samples.iter().enumerate() {
        println!("--- HTML sample {} ---", i + 1);
        let tokens = Tokenizer::new(sample).tokenize();
        for t in tokens {
            println!("{:?}", t);
        }
        println!();
    }

    let css_samples = [
        r#"
        body { font-family: sans-serif; color: #333; }
        div.card > p.title { font-weight: bold; margin: 0 !important; }
        #header, .nav a:hover { color: blue; }
        @media (max-width: 600px) { .card { width: 100%; } }
        "#,
        // malformed CSS on purpose -- parser must not panic
        r#"div { color ; background: red missing-brace"#,
    ];

    for (i, sample) in css_samples.iter().enumerate() {
        println!("--- CSS sample {} ---", i + 1);
        let tokens = CssTokenizer::new(sample).tokenize();
        let stylesheet = CssParser::new(tokens).parse();
        for rule in &stylesheet.rules {
            for selector in &rule.selectors {
                println!("selector {:?} specificity {:?}", selector, selector.specificity());
            }
            for decl in &rule.declarations {
                println!("  {}: {}{}", decl.property, decl.value, if decl.important { " !important" } else { "" });
            }
        }
        println!();
    }

    println!("--- Batch 3: DOM + style + layout end-to-end ---");
    let html_doc = r#"<div class="page"><h1 class="title">Diaz's Browser</h1><p>Building from scratch.</p></div>"#;
    let css_doc = r#"
        .page { width: 400px; padding: 10px; }
        .title { color: #222; margin: 0; }
        p { margin: 8px 0; }
    "#;

    let dom_tree = build_dom(Tokenizer::new(html_doc).tokenize());
    let stylesheet = CssParser::new(CssTokenizer::new(css_doc).tokenize()).parse();
    let styles = compute_styles(&dom_tree, &stylesheet);

    if let Some(mut layout_root) = build_layout_tree(&dom_tree, &styles, dom_tree.root) {
        layout_tree(&mut layout_root, 800.0, &styles);
        print_layout_box(&layout_root, 0);
    }

    use winit::{
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
        keyboard::{Key, NamedKey},
    };
    use std::sync::Arc;

    println!("Starting GUI window for Batch 4...");

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Diaz's Secure Browser Engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
            .build(&event_loop)
            .unwrap()
    );

    let mut state = pollster::block_on(RenderState::new(window.clone()));
    let mut scroll_y: f32 = 0.0;

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
                    WindowEvent::KeyboardInput { event: key_event, .. } => {
                        if key_event.state == winit::event::ElementState::Pressed {
                            match key_event.logical_key {
                                Key::Named(NamedKey::ArrowDown) => {
                                    scroll_y += 20.0;
                                    state.window.request_redraw();
                                }
                                Key::Named(NamedKey::ArrowUp) => {
                                    scroll_y = (scroll_y - 20.0).max(0.0);
                                    state.window.request_redraw();
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // Re-run the full pipeline: DOM -> Style -> Layout -> Paint -> Render
                        let html_doc = r#"
                        <div class="page">
                            <h1 class="title">Diaz's Secure Browser</h1>
                            <p>Building a secure, custom browser rendering engine from scratch in Rust.</p>
                            <p>This page is laid out using a custom Block Formatting Context engine and rendered on the GPU using WGPU!</p>
                            <p style="color: blue;">Standard features like text color, font sizes, margins, padding, and borders are fully supported.</p>
                            <br/>
                            <div class="card" style="background-color: #e0f0ff; border-width: 2px; border-color: #004080; padding: 10px;">
                                <p style="font-weight: bold; color: #004080;">GPU-Accelerated Compositing</p>
                                <p>This colored card has custom borders, padding, background color, and margin. Use the ArrowUp and ArrowDown keys to scroll the page smoothly at 60 FPS.</p>
                            </div>
                            <br/>
                            <p style="color: #666; font-size: 14px;">Batch 4 (Compositing + Rasterization) complete.</p>
                        </div>
                        "#;

                        let css_doc = r#"
                            .page { width: 600px; padding: 20px; background-color: #ffffff; }
                            .title { color: #111; font-size: 32px; margin-bottom: 12px; }
                            p { color: #333; font-size: 16px; margin: 8px 0; }
                            .card { margin: 15px 0; }
                        "#;

                        let dom_tree = build_dom(Tokenizer::new(html_doc).tokenize());
                        let stylesheet = CssParser::new(CssTokenizer::new(css_doc).tokenize()).parse();
                        let styles = compute_styles(&dom_tree, &stylesheet);

                        if let Some(mut layout_root) = build_layout_tree(&dom_tree, &styles, dom_tree.root) {
                            let viewport_width = state.size.width as f32;
                            layout_tree(&mut layout_root, viewport_width, &styles);
                            
                            let paint_commands = paint::build_display_list(&layout_root, &styles);
                            if let Err(e) = state.render(&paint_commands, scroll_y) {
                                eprintln!("WGPU Render error: {:?}", e);
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
                d.x, d.y, d.width, d.height
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
        assert_eq!(descendant_selector.steps[1].0, Some(super::css::Combinator::Descendant));

        let child_selector = &sheet.rules[1].selectors[0];
        assert_eq!(child_selector.steps[1].0, Some(super::css::Combinator::Child));
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
    fn unsupported_pseudo_class_drops_whole_rule_cleanly() {
        // Regression test: `:hover` isn't supported yet. Earlier the parser
        // leaked the stray "hover" identifier into a bogus new rule and
        // silently swallowed the *next* rule's declaration block. The fix
        // drops the whole unsupported rule as one unit so nothing else on
        // the stylesheet gets corrupted.
        let sheet = parse("#header, .nav a:hover { color: blue; } p { margin: 0; }");
        // only the trailing supported rule should survive
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].declarations[0].property, "margin");
        // critically, no bogus rule with tag "hover" should appear
        for rule in &sheet.rules {
            for selector in &rule.selectors {
                for (_, simple) in &selector.steps {
                    assert_ne!(simple.tag.as_deref(), Some("hover"));
                }
            }
        }
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
        assert!(styles.get(&deep_em_idx).and_then(|s| s.get("color")).is_none());
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