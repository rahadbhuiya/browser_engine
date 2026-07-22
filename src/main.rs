mod css;
mod html;

use css::{CssParser, CssTokenizer};
use html::Tokenizer;

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