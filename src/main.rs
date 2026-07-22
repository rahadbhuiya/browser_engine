mod html;

use html::Tokenizer;

fn main() {
    let samples = [
        r#"<!DOCTYPE html><html><body><h1 class="title">Hi Diaz</h1><p>where are you sonali</p><!-- comment --><br/></body></html>"#,
        // malformed input on purpose -- tokenizer must not panic
        r#"<div class=unclosed <p>broken <>< / ></div"#,
    ];

    for (i, sample) in samples.iter().enumerate() {
        println!("--- sample {} ---", i + 1);
        let tokens = Tokenizer::new(sample).tokenize();
        for t in tokens {
            println!("{:?}", t);
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