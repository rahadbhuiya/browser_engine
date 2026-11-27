//! CSS stylesheet and rule parser.

use super::cssom::{
    AttributeMatcher, AttributeSelector, Combinator, Declaration, MediaQuery, PseudoClass, Rule,
    Selector, SimpleSelector, Stylesheet,
};
use super::token::CssToken;

pub struct CssParser {
    tokens: Vec<CssToken>,
    pos: usize,
}

impl CssParser {
    pub fn new(tokens: Vec<CssToken>) -> Self {
        CssParser { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Stylesheet {
        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some(CssToken::Eof) => break,
                Some(CssToken::AtKeyword(name)) if name.eq_ignore_ascii_case("media") => {
                    let (_mq, media_rules) = self.parse_media_query();
                    rules.extend(media_rules);
                }
                Some(CssToken::AtKeyword(_)) => self.skip_at_rule(),
                Some(CssToken::RightBrace) => {
                    self.pos += 1;
                }
                _ => {
                    let pos_before = self.pos;
                    if let Some(rule) = self.parse_rule() {
                        rules.push(rule);
                    }
                    if self.pos == pos_before {
                        self.pos += 1;
                    }
                }
            }
        }
        Stylesheet { rules }
    }

    fn peek(&self) -> Option<&CssToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<CssToken> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(CssToken::Whitespace)) {
            self.pos += 1;
        }
    }

    fn skip_at_rule(&mut self) {
        self.pos += 1;
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) => return,
                Some(CssToken::Semicolon) => {
                    self.pos += 1;
                    return;
                }
                Some(CssToken::LeftBrace) => {
                    self.skip_balanced_block();
                    return;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_media_query(&mut self) -> (MediaQuery, Vec<Rule>) {
        self.pos += 1;
        self.skip_whitespace();

        let mut mq = MediaQuery::default();

        while let Some(tok) = self.peek() {
            if matches!(tok, CssToken::LeftBrace | CssToken::Eof) {
                break;
            }
            if matches!(tok, CssToken::LeftParen) {
                self.pos += 1;
                self.skip_whitespace();

                let feature = match self.peek() {
                    Some(CssToken::Ident(s)) => {
                        let name = s.to_ascii_lowercase();
                        self.pos += 1;
                        name
                    }
                    _ => String::new(),
                };
                self.skip_whitespace();

                if matches!(self.peek(), Some(CssToken::Colon)) {
                    self.pos += 1;
                    self.skip_whitespace();

                    let mut num_val = None;
                    match self.peek() {
                        Some(CssToken::Number(n)) => {
                            num_val = Some(*n as f32);
                            self.pos += 1;
                        }
                        Some(CssToken::Ident(id)) => {
                            let id_lower = id.to_ascii_lowercase();
                            self.pos += 1;
                            if feature == "prefers-color-scheme" {
                                if id_lower == "dark" {
                                    mq.prefers_dark = Some(true);
                                } else if id_lower == "light" {
                                    mq.prefers_dark = Some(false);
                                }
                            }
                        }
                        _ => {}
                    }
                    self.skip_whitespace();

                    if matches!(self.peek(), Some(CssToken::Ident(_))) {
                        self.pos += 1;
                    }

                    if let Some(val) = num_val {
                        if feature == "min-width" {
                            mq.min_width = Some(val);
                        } else if feature == "max-width" {
                            mq.max_width = Some(val);
                        }
                    }
                }

                while let Some(inner) = self.peek() {
                    if matches!(
                        inner,
                        CssToken::RightParen | CssToken::LeftBrace | CssToken::Eof
                    ) {
                        if matches!(inner, CssToken::RightParen) {
                            self.pos += 1;
                        }
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                self.pos += 1;
            }
            self.skip_whitespace();
        }

        let mut rules = Vec::new();
        if matches!(self.peek(), Some(CssToken::LeftBrace)) {
            self.pos += 1;
            loop {
                self.skip_whitespace();
                match self.peek() {
                    None | Some(CssToken::Eof) | Some(CssToken::RightBrace) => break,
                    _ => {
                        let pos_before = self.pos;
                        if let Some(mut rule) = self.parse_rule() {
                            rule.media_query = Some(mq.clone());
                            rules.push(rule);
                        }
                        if self.pos == pos_before {
                            self.pos += 1;
                        }
                    }
                }
            }
            if matches!(self.peek(), Some(CssToken::RightBrace)) {
                self.pos += 1;
            }
        }

        (mq, rules)
    }

    /// Consumes a `{ ... }` block, correctly handling nested braces (needed for
    /// @media blocks which contain full rules with their own braces).
    fn skip_balanced_block(&mut self) {
        let mut depth = 0;
        loop {
            match self.advance() {
                Some(CssToken::LeftBrace) => depth += 1,
                Some(CssToken::RightBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                None => return, // unterminated block at EOF
                _ => {}
            }
        }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selector_list()?;
        self.skip_whitespace();
        if !matches!(self.peek(), Some(CssToken::LeftBrace)) {
            // Either malformed input, or selector syntax we don't support yet
            // (e.g. pseudo-classes like `:hover` — planned for a later batch).
            // Recover by discarding everything up to and including the next
            self.recover_malformed_rule();
            return None;
        }
        self.pos += 1; // consume '{'
        let declarations = self.parse_declarations();
        Some(Rule {
            selectors,
            declarations,
            media_query: None,
        })
    }

    fn recover_malformed_rule(&mut self) {
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) => return,
                Some(CssToken::LeftBrace) => {
                    self.skip_balanced_block();
                    return;
                }
                Some(CssToken::Semicolon) | Some(CssToken::RightBrace) => {
                    self.pos += 1;
                    return;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    pub fn parse_selector_list(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();
        loop {
            self.skip_whitespace();
            let selector = self.parse_single_selector()?;
            selectors.push(selector);
            self.skip_whitespace();
            match self.peek() {
                Some(CssToken::Comma) => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Some(selectors)
    }

    pub fn parse_single_selector(&mut self) -> Option<Selector> {
        let mut steps = Vec::new();
        let first = self.parse_simple_selector()?;
        steps.push((None, first));

        loop {
            let had_whitespace = matches!(self.peek(), Some(CssToken::Whitespace));
            if had_whitespace {
                self.pos += 1;
                self.skip_whitespace();
            }

            let explicit_combinator = match self.peek() {
                Some(CssToken::Delim('>')) => Some(Combinator::Child),
                Some(CssToken::Delim('+')) => Some(Combinator::NextSibling),
                Some(CssToken::Delim('~')) => Some(Combinator::SubsequentSibling),
                _ => None,
            };

            if let Some(comb) = explicit_combinator {
                self.pos += 1;
                self.skip_whitespace();
                match self.parse_simple_selector() {
                    Some(simple) => steps.push((Some(comb), simple)),
                    None => break, // trailing combinator with nothing after — stop
                }
            } else if had_whitespace && Self::starts_simple_selector(self.peek()) {
                match self.parse_simple_selector() {
                    Some(simple) => steps.push((Some(Combinator::Descendant), simple)),
                    None => break,
                }
            } else {
                break;
            }
        }

        Some(Selector { steps })
    }

    fn starts_simple_selector(tok: Option<&CssToken>) -> bool {
        matches!(
            tok,
            Some(CssToken::Ident(_))
                | Some(CssToken::Hash(_))
                | Some(CssToken::Delim('.'))
                | Some(CssToken::Delim('*'))
                | Some(CssToken::Colon)
                | Some(CssToken::LeftBracket)
        )
    }

    fn parse_attr_value(&mut self) -> String {
        match self.peek() {
            Some(CssToken::Str(s)) | Some(CssToken::Ident(s)) => {
                let res = s.clone();
                self.pos += 1;
                res
            }
            _ => String::new(),
        }
    }

    fn parse_simple_selector(&mut self) -> Option<SimpleSelector> {
        let mut simple = SimpleSelector::default();
        let mut matched_anything = false;

        if let Some(CssToken::Delim('*')) = self.peek() {
            simple.universal = true;
            self.pos += 1;
            matched_anything = true;
        } else if let Some(CssToken::Ident(name)) = self.peek() {
            simple.tag = Some(name.to_ascii_lowercase());
            self.pos += 1;
            matched_anything = true;
        }

        loop {
            match self.peek() {
                Some(CssToken::Hash(id)) => {
                    simple.id = Some(id.clone());
                    self.pos += 1;
                    matched_anything = true;
                }
                Some(CssToken::Delim('.')) => {
                    self.pos += 1;
                    if let Some(CssToken::Ident(class)) = self.peek() {
                        simple.classes.push(class.clone());
                        self.pos += 1;
                        matched_anything = true;
                    } else {
                        // malformed "." with nothing after — recover by stopping
                        break;
                    }
                }
                Some(CssToken::Colon) => {
                    self.pos += 1;
                    // Optional double colon `::` for pseudo-elements (tolerant parsing)
                    if matches!(self.peek(), Some(CssToken::Colon)) {
                        self.pos += 1;
                    }
                    if let Some(CssToken::Ident(pseudo)) = self.peek() {
                        let pc = PseudoClass::from_name(pseudo);
                        simple.pseudo_classes.push(pc);
                        self.pos += 1;
                        matched_anything = true;
                    } else {
                        // malformed ":" with nothing after — recover by stopping
                        break;
                    }
                }
                Some(CssToken::LeftBracket) => {
                    self.pos += 1;
                    self.skip_whitespace();
                    if let Some(CssToken::Ident(attr_name)) = self.peek() {
                        let attr_name = attr_name.clone();
                        self.pos += 1;
                        self.skip_whitespace();
                        let matcher = match self.peek() {
                            Some(CssToken::RightBracket) => AttributeMatcher::Exists,
                            Some(CssToken::Delim('=')) => {
                                self.pos += 1;
                                self.skip_whitespace();
                                let val = self.parse_attr_value();
                                AttributeMatcher::Exact(val)
                            }
                            Some(CssToken::Delim('~')) => {
                                self.pos += 1;
                                if matches!(self.peek(), Some(CssToken::Delim('='))) {
                                    self.pos += 1;
                                }
                                self.skip_whitespace();
                                let val = self.parse_attr_value();
                                AttributeMatcher::Includes(val)
                            }
                            Some(CssToken::Delim('|')) => {
                                self.pos += 1;
                                if matches!(self.peek(), Some(CssToken::Delim('='))) {
                                    self.pos += 1;
                                }
                                self.skip_whitespace();
                                let val = self.parse_attr_value();
                                AttributeMatcher::DashMatch(val)
                            }
                            Some(CssToken::Delim('^')) => {
                                self.pos += 1;
                                if matches!(self.peek(), Some(CssToken::Delim('='))) {
                                    self.pos += 1;
                                }
                                self.skip_whitespace();
                                let val = self.parse_attr_value();
                                AttributeMatcher::Prefix(val)
                            }
                            Some(CssToken::Delim('$')) => {
                                self.pos += 1;
                                if matches!(self.peek(), Some(CssToken::Delim('='))) {
                                    self.pos += 1;
                                }
                                self.skip_whitespace();
                                let val = self.parse_attr_value();
                                AttributeMatcher::Suffix(val)
                            }
                            Some(CssToken::Delim('*')) => {
                                self.pos += 1;
                                if matches!(self.peek(), Some(CssToken::Delim('='))) {
                                    self.pos += 1;
                                }
                                self.skip_whitespace();
                                let val = self.parse_attr_value();
                                AttributeMatcher::Substring(val)
                            }
                            _ => AttributeMatcher::Exists,
                        };
                        self.skip_whitespace();
                        if matches!(self.peek(), Some(CssToken::RightBracket)) {
                            self.pos += 1;
                        }
                        simple.attributes.push(AttributeSelector {
                            name: attr_name,
                            matcher,
                        });
                        matched_anything = true;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        if matched_anything {
            Some(simple)
        } else {
            None
        }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut decls = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None | Some(CssToken::Eof) | Some(CssToken::RightBrace) => {
                    if matches!(self.peek(), Some(CssToken::RightBrace)) {
                        self.pos += 1;
                    }
                    break;
                }
                Some(CssToken::Semicolon) => {
                    self.pos += 1; // stray semicolon, e.g. empty declaration
                }
                Some(CssToken::Ident(_)) => {
                    if let Some(decl) = self.parse_one_declaration() {
                        decls.push(decl);
                    }
                }
                _ => {
                    // malformed token where a property name was expected —
                    // skip it and keep going rather than aborting the block
                    self.pos += 1;
                }
            }
        }
        decls
    }

    fn parse_one_declaration(&mut self) -> Option<Declaration> {
        let property = match self.advance() {
            Some(CssToken::Ident(name)) => name.to_ascii_lowercase(),
            _ => return None,
        };
        self.skip_whitespace();
        if !matches!(self.peek(), Some(CssToken::Colon)) {
            // malformed: "property" with no colon — skip to next ';' or '}'
            self.recover_to_declaration_end();
            return None;
        }
        self.pos += 1; // consume ':'
        self.skip_whitespace();

        let mut value_tokens = Vec::new();
        loop {
            match self.peek() {
                None
                | Some(CssToken::Eof)
                | Some(CssToken::Semicolon)
                | Some(CssToken::RightBrace) => break,
                Some(tok) => {
                    value_tokens.push(tok.clone());
                    self.pos += 1;
                }
            }
        }
        if matches!(self.peek(), Some(CssToken::Semicolon)) {
            self.pos += 1;
        }

        let important = Self::strip_important(&mut value_tokens);
        let value = Self::render_value(&value_tokens);

        Some(Declaration {
            property,
            value,
            important,
        })
    }

    fn recover_to_declaration_end(&mut self) {
        loop {
            match self.peek() {
                None | Some(CssToken::Eof) | Some(CssToken::RightBrace) => return,
                Some(CssToken::Semicolon) => {
                    self.pos += 1;
                    return;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }
    }

    /// Detects a trailing "!important" in a declaration's value tokens and
    /// removes it, returning whether it was present.
    fn strip_important(tokens: &mut Vec<CssToken>) -> bool {
        // trim trailing whitespace first
        while matches!(tokens.last(), Some(CssToken::Whitespace)) {
            tokens.pop();
        }
        if let Some(CssToken::Ident(word)) = tokens.last() {
            if word.eq_ignore_ascii_case("important") {
                let ident_pos = tokens.len() - 1;
                // look back through optional whitespace for the '!' delim
                let mut i = ident_pos;
                while i > 0 && matches!(tokens[i - 1], CssToken::Whitespace) {
                    i -= 1;
                }
                if i > 0 && matches!(tokens[i - 1], CssToken::Delim('!')) {
                    tokens.truncate(i - 1);
                    while matches!(tokens.last(), Some(CssToken::Whitespace)) {
                        tokens.pop();
                    }
                    return true;
                }
            }
        }
        false
    }

    fn render_value(tokens: &[CssToken]) -> String {
        let mut out = String::new();
        for tok in tokens {
            match tok {
                CssToken::Whitespace => out.push(' '),
                CssToken::Ident(s) => out.push_str(s),
                CssToken::Str(s) => {
                    out.push('"');
                    out.push_str(s);
                    out.push('"');
                }
                CssToken::Number(n) => out.push_str(&n.to_string()),
                CssToken::Percentage(n) => {
                    out.push_str(&n.to_string());
                    out.push('%');
                }
                CssToken::Hash(h) => {
                    out.push('#');
                    out.push_str(h);
                }
                CssToken::Delim(c) => out.push(*c),
                CssToken::Comma => out.push(','),
                CssToken::LeftParen => out.push('('),
                CssToken::RightParen => out.push(')'),
                _ => {}
            }
        }
        out.trim().to_string()
    }
}

/// Convenience helper to parse a single CSS selector string.
pub fn parse_selector(input: &str) -> Option<Selector> {
    let tokens = crate::css::tokenizer::CssTokenizer::new(input).tokenize();
    let mut parser = CssParser::new(tokens);
    parser.parse_single_selector()
}

/// Convenience helper to parse a comma-separated CSS selector list.
pub fn parse_selector_list(input: &str) -> Option<Vec<Selector>> {
    let tokens = crate::css::tokenizer::CssTokenizer::new(input).tokenize();
    let mut parser = CssParser::new(tokens);
    parser.parse_selector_list()
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_single_selector() {
        let sel = parse_selector("div.content > p#intro").unwrap();
        assert_eq!(sel.steps.len(), 2);
        assert_eq!(sel.steps[0].1.tag.as_deref(), Some("div"));
        assert_eq!(sel.steps[0].1.classes, vec!["content".to_string()]);
        assert_eq!(sel.steps[1].0, Some(Combinator::Child));
        assert_eq!(sel.steps[1].1.tag.as_deref(), Some("p"));
        assert_eq!(sel.steps[1].1.id.as_deref(), Some("intro"));
    }

    #[test]
    fn test_parse_selector_list() {
        let list = parse_selector_list("h1, h2.title, #banner").unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].steps[0].1.tag.as_deref(), Some("h1"));
        assert_eq!(list[1].steps[0].1.tag.as_deref(), Some("h2"));
        assert_eq!(list[1].steps[0].1.classes, vec!["title".to_string()]);
        assert_eq!(list[2].steps[0].1.id.as_deref(), Some("banner"));
    }

    #[test]
    fn test_parse_pseudo_classes() {
        let sel = parse_selector("a:hover, button:first-child:focus").unwrap();
        assert_eq!(sel.steps[0].1.tag.as_deref(), Some("a"));
        assert_eq!(sel.steps[0].1.pseudo_classes, vec![PseudoClass::Hover]);

        let sel2 = parse_selector("button:first-child:focus").unwrap();
        assert_eq!(sel2.steps[0].1.tag.as_deref(), Some("button"));
        assert_eq!(
            sel2.steps[0].1.pseudo_classes,
            vec![PseudoClass::FirstChild, PseudoClass::Focus]
        );
        // Specificity: tag (1) + 2 pseudo-classes (2) = (0, 2, 1)
        assert_eq!(sel2.specificity(), (0, 2, 1));
    }

    #[test]
    fn test_parse_attribute_selectors() {
        let sel = parse_selector(r#"input[type="text"][disabled]"#).unwrap();
        assert_eq!(sel.steps[0].1.tag.as_deref(), Some("input"));
        assert_eq!(sel.steps[0].1.attributes.len(), 2);
        assert_eq!(sel.steps[0].1.attributes[0].name, "type");
        assert_eq!(
            sel.steps[0].1.attributes[0].matcher,
            AttributeMatcher::Exact("text".to_string())
        );
        assert_eq!(sel.steps[0].1.attributes[1].name, "disabled");
        assert_eq!(
            sel.steps[0].1.attributes[1].matcher,
            AttributeMatcher::Exists
        );
        // Specificity: tag (1) + 2 attributes (2) = (0, 2, 1)
        assert_eq!(sel.specificity(), (0, 2, 1));
    }
}
