// HTML5 Standard User-Agent Default Stylesheet (Phase 4.1)
//
// Provides standardized browser default styles for HTML semantic elements,
// typography, headings, tables, blockquotes, lists, and forms.

use super::cssom::Stylesheet;
use super::parser::CssParser;
use super::tokenizer::CssTokenizer;

pub const DEFAULT_USER_AGENT_CSS: &str = r#"
html, body {
    margin: 0;
    padding: 0;
    font-family: sans-serif;
    font-size: 16px;
    color: #1e293b;
    background-color: #ffffff;
}

h1 { display: block; font-size: 28px; margin-top: 18px; margin-bottom: 14px; font-weight: bold; }
h2 { display: block; font-size: 22px; margin-top: 16px; margin-bottom: 12px; font-weight: bold; }
h3 { display: block; font-size: 18px; margin-top: 14px; margin-bottom: 10px; font-weight: bold; }
h4 { display: block; font-size: 16px; margin-top: 12px; margin-bottom: 8px; font-weight: bold; }
h5 { display: block; font-size: 14px; margin-top: 10px; margin-bottom: 6px; font-weight: bold; }
h6 { display: block; font-size: 12px; margin-top: 8px; margin-bottom: 4px; font-weight: bold; }

p { display: block; margin-top: 12px; margin-bottom: 12px; line-height: 22px; }
blockquote { display: block; margin: 12px 24px; padding: 8px 16px; border-color: #cbd5e1; border-width: 0; }

ul, ol { display: block; margin-top: 12px; margin-bottom: 12px; padding-left: 28px; }
li { display: block; margin-bottom: 4px; }

nav, header, footer, section, article, aside, main, figure {
    display: block;
}

hr {
    display: block;
    height: 1px;
    border-width: 0;
    background-color: #e2e8f0;
    margin: 16px 0;
}

table { display: block; margin: 12px 0; }
thead, tbody, tfoot, tr { display: block; }
th, td { display: block; padding: 6px 12px; }
th { font-weight: bold; }

a {
    color: #2563eb;
    text-decoration: underline;
}

code, pre {
    font-family: monospace;
    font-size: 14px;
    background-color: #f1f5f9;
}

pre {
    display: block;
    padding: 12px;
    margin: 12px 0;
    border-radius: 4px;
}

button {
    display: inline-block;
    padding: 6px 14px;
    background-color: #f1f5f9;
    border-width: 1px;
    border-color: #cbd5e1;
    border-radius: 4px;
    color: #0f172a;
    font-size: 14px;
}
"#;

pub fn default_user_agent_stylesheet() -> Stylesheet {
    CssParser::new(CssTokenizer::new(DEFAULT_USER_AGENT_CSS).tokenize()).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_stylesheet_parses_cleanly() {
        let sheet = default_user_agent_stylesheet();
        assert!(!sheet.rules.is_empty());
        // Verify headings, body, paragraphs are present
        let has_h1 = sheet.rules.iter().any(|r| {
            r.selectors.iter().any(|s| {
                s.steps
                    .iter()
                    .any(|(_, simple)| simple.tag.as_deref() == Some("h1"))
            })
        });
        assert!(has_h1, "User-agent stylesheet should contain h1 rules");
    }
}
