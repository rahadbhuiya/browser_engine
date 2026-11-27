use crate::dom::{Dom, NodeType};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Checkbox,
    Radio,
    Submit,
    Button,
    Reset,
    Hidden,
}

impl InputType {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "password" => InputType::Password,
            "email" => InputType::Email,
            "number" => InputType::Number,
            "checkbox" => InputType::Checkbox,
            "radio" => InputType::Radio,
            "submit" => InputType::Submit,
            "button" => InputType::Button,
            "reset" => InputType::Reset,
            "hidden" => InputType::Hidden,
            _ => InputType::Text,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Text => "text",
            InputType::Password => "password",
            InputType::Email => "email",
            InputType::Number => "number",
            InputType::Checkbox => "checkbox",
            InputType::Radio => "radio",
            InputType::Submit => "submit",
            InputType::Button => "button",
            InputType::Reset => "reset",
            InputType::Hidden => "hidden",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMethod {
    Get,
    Post,
}

impl FormMethod {
    pub fn from_str(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("post") {
            FormMethod::Post
        } else {
            FormMethod::Get
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FormMethod::Get => "GET",
            FormMethod::Post => "POST",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormEncoding {
    UrlEncoded, // application/x-www-form-urlencoded
    Multipart,  // multipart/form-data
    TextPlain,  // text/plain
}

impl FormEncoding {
    pub fn from_str(s: &str) -> Self {
        let val = s.trim().to_ascii_lowercase();
        if val.contains("multipart/form-data") {
            FormEncoding::Multipart
        } else if val.contains("text/plain") {
            FormEncoding::TextPlain
        } else {
            FormEncoding::UrlEncoded
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FormEncoding::UrlEncoded => "application/x-www-form-urlencoded",
            FormEncoding::Multipart => "multipart/form-data",
            FormEncoding::TextPlain => "text/plain",
        }
    }
}

/// Dynamic interactive state of an input control (e.g., text buffer, cursor, focus).
#[derive(Debug, Clone)]
pub struct FormControlState {
    pub node_idx: usize,
    pub input_type: InputType,
    pub name: String,
    pub value: String,
    pub checked: bool,
    pub disabled: bool,
    pub is_focused: bool,
    pub cursor_position: usize,
}

impl FormControlState {
    pub fn new(
        node_idx: usize,
        input_type: InputType,
        name: String,
        initial_value: String,
    ) -> Self {
        let cursor = initial_value.chars().count();
        FormControlState {
            node_idx,
            input_type,
            name,
            value: initial_value,
            checked: false,
            disabled: false,
            is_focused: false,
            cursor_position: cursor,
        }
    }

    /// Appends a typed character at the current cursor position.
    pub fn insert_char(&mut self, ch: char) {
        if self.disabled {
            return;
        }
        let char_idx = self.cursor_position.min(self.value.chars().count());
        let byte_offset = self
            .value
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len());

        self.value.insert(byte_offset, ch);
        self.cursor_position += 1;
    }

    /// Handles Backspace key press (deletes character before cursor).
    pub fn delete_backwards(&mut self) -> bool {
        if self.disabled || self.cursor_position == 0 {
            return false;
        }
        let target_idx = self.cursor_position - 1;
        if let Some((byte_offset, ch)) = self.value.char_indices().nth(target_idx) {
            self.value.drain(byte_offset..byte_offset + ch.len_utf8());
            self.cursor_position -= 1;
            true
        } else {
            false
        }
    }

    /// Handles Delete key press (deletes character at cursor).
    pub fn delete_forwards(&mut self) -> bool {
        if self.disabled || self.cursor_position >= self.value.chars().count() {
            return false;
        }
        if let Some((byte_offset, ch)) = self.value.char_indices().nth(self.cursor_position) {
            self.value.drain(byte_offset..byte_offset + ch.len_utf8());
            true
        } else {
            false
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let total = self.value.chars().count();
        if self.cursor_position < total {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.value.chars().count();
    }
}

/// Prepared HTTP payload ready for network submission.
#[derive(Debug, Clone, PartialEq)]
pub struct FormSubmissionPayload {
    pub target_url: String,
    pub method: FormMethod,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

/// Manages interactive form controls across the loaded DOM.
#[derive(Default)]
pub struct FormManager {
    controls: HashMap<usize, FormControlState>, // node_idx -> control state
    focused_control: Option<usize>,
}

impl FormManager {
    pub fn new() -> Self {
        FormManager {
            controls: HashMap::new(),
            focused_control: None,
        }
    }

    /// Synchronizes interactive form controls from DOM elements (e.g. input, textarea, button).
    pub fn scan_dom(&mut self, dom: &Dom) {
        for (idx, node) in dom.nodes.iter().enumerate() {
            if let NodeType::Element(ref elem) = node.node_type {
                let tag = elem.tag.to_ascii_lowercase();
                if tag == "input" {
                    let itype = InputType::from_str(elem.get_attribute("type").unwrap_or("text"));
                    let name = elem.get_attribute("name").unwrap_or("").to_string();
                    let val = elem.get_attribute("value").unwrap_or("").to_string();
                    let checked = elem.has_attribute("checked");
                    let disabled = elem.has_attribute("disabled");

                    let state = self
                        .controls
                        .entry(idx)
                        .or_insert_with(|| FormControlState::new(idx, itype, name, val));
                    state.disabled = disabled;
                    if itype == InputType::Checkbox || itype == InputType::Radio {
                        state.checked = checked;
                    }
                } else if tag == "textarea" {
                    let name = elem.get_attribute("name").unwrap_or("").to_string();
                    let val = dom.text_content(idx);
                    let disabled = elem.has_attribute("disabled");

                    let state = self
                        .controls
                        .entry(idx)
                        .or_insert_with(|| FormControlState::new(idx, InputType::Text, name, val));
                    state.disabled = disabled;
                } else if tag == "button" {
                    let itype = InputType::from_str(elem.get_attribute("type").unwrap_or("submit"));
                    let name = elem.get_attribute("name").unwrap_or("").to_string();
                    let val = elem.get_attribute("value").unwrap_or("").to_string();
                    let disabled = elem.has_attribute("disabled");

                    let state = self
                        .controls
                        .entry(idx)
                        .or_insert_with(|| FormControlState::new(idx, itype, name, val));
                    state.disabled = disabled;
                }
            }
        }
    }

    pub fn get_control(&self, node_idx: usize) -> Option<&FormControlState> {
        self.controls.get(&node_idx)
    }

    pub fn get_control_mut(&mut self, node_idx: usize) -> Option<&mut FormControlState> {
        self.controls.get_mut(&node_idx)
    }

    pub fn set_focus(&mut self, node_idx: Option<usize>) {
        if let Some(old) = self.focused_control {
            if let Some(c) = self.controls.get_mut(&old) {
                c.is_focused = false;
            }
        }
        self.focused_control = node_idx;
        if let Some(new_idx) = node_idx {
            if let Some(c) = self.controls.get_mut(&new_idx) {
                c.is_focused = true;
            }
        }
    }

    pub fn focused_control(&self) -> Option<usize> {
        self.focused_control
    }

    /// Handles keyboard input into the currently focused form control.
    pub fn handle_char_input(&mut self, ch: char) -> bool {
        if let Some(focused) = self.focused_control {
            if let Some(c) = self.controls.get_mut(&focused) {
                if !c.disabled && !ch.is_control() {
                    c.insert_char(ch);
                    return true;
                }
            }
        }
        false
    }

    /// Handles key actions like Backspace or Delete on the focused control.
    pub fn handle_backspace(&mut self) -> bool {
        if let Some(focused) = self.focused_control {
            if let Some(c) = self.controls.get_mut(&focused) {
                return c.delete_backwards();
            }
        }
        false
    }
}

/// Collects all successful form submittable key-value pairs from a `<form>` element.
pub fn collect_form_fields(
    dom: &Dom,
    form_idx: usize,
    form_manager: &FormManager,
) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let descendants = dom.descendants(form_idx);

    for idx in descendants {
        if let Some(control) = form_manager.get_control(idx) {
            if control.disabled || control.name.is_empty() {
                continue;
            }

            match control.input_type {
                InputType::Text
                | InputType::Password
                | InputType::Email
                | InputType::Number
                | InputType::Hidden => {
                    fields.push((control.name.clone(), control.value.clone()));
                }
                InputType::Checkbox | InputType::Radio => {
                    if control.checked {
                        let val = if control.value.is_empty() {
                            "on".to_string()
                        } else {
                            control.value.clone()
                        };
                        fields.push((control.name.clone(), val));
                    }
                }
                InputType::Submit => {
                    if !control.value.is_empty() {
                        fields.push((control.name.clone(), control.value.clone()));
                    }
                }
                InputType::Button | InputType::Reset => {}
            }
        } else if let Some(elem) = dom.element_at(idx) {
            // Fallback for elements not yet registered in form_manager
            let tag = elem.tag.to_ascii_lowercase();
            if tag == "input" {
                let name = elem.get_attribute("name").unwrap_or("");
                let itype = InputType::from_str(elem.get_attribute("type").unwrap_or("text"));
                let val = elem.get_attribute("value").unwrap_or("");
                let disabled = elem.has_attribute("disabled");

                if !name.is_empty() && !disabled {
                    match itype {
                        InputType::Text
                        | InputType::Password
                        | InputType::Email
                        | InputType::Number
                        | InputType::Hidden => {
                            fields.push((name.to_string(), val.to_string()));
                        }
                        InputType::Checkbox | InputType::Radio => {
                            if elem.has_attribute("checked") {
                                let v = if val.is_empty() { "on" } else { val };
                                fields.push((name.to_string(), v.to_string()));
                            }
                        }
                        _ => {}
                    }
                }
            } else if tag == "textarea" {
                let name = elem.get_attribute("name").unwrap_or("");
                let disabled = elem.has_attribute("disabled");
                if !name.is_empty() && !disabled {
                    let val = dom.text_content(idx);
                    fields.push((name.to_string(), val));
                }
            }
        }
    }

    fields
}

/// Encodes form fields as `application/x-www-form-urlencoded`.
pub fn encode_url_encoded(fields: &[(String, String)]) -> String {
    let mut parts = Vec::new();
    for (key, val) in fields {
        let enc_k = percent_encode_component(key);
        let enc_v = percent_encode_component(val);
        parts.push(format!("{}={}", enc_k, enc_v));
    }
    parts.join("&")
}

/// RFC 3986 / HTML5 application/x-www-form-urlencoded percent encoder.
pub fn percent_encode_component(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(b as char);
            }
            b' ' => output.push('+'),
            _ => output.push_str(&format!("%{:02X}", b)),
        }
    }
    output
}

/// Encodes form fields as `multipart/form-data` with a specified boundary.
pub fn encode_multipart(fields: &[(String, String)], boundary: &str) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, val) in fields {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
        );
        body.extend_from_slice(val.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    body
}

/// Builds the final `FormSubmissionPayload` from a `<form>` element.
pub fn build_form_submission(
    dom: &Dom,
    form_idx: usize,
    form_manager: &FormManager,
    base_url: &str,
) -> Result<FormSubmissionPayload, String> {
    let form_elem = dom
        .element_at(form_idx)
        .ok_or_else(|| "Target is not a valid DOM element".to_string())?;

    if !form_elem.tag.eq_ignore_ascii_case("form") {
        return Err("Target element is not a <form>".to_string());
    }

    let action_attr = form_elem.get_attribute("action").unwrap_or("");
    let method_attr = form_elem.get_attribute("method").unwrap_or("GET");
    let enctype_attr = form_elem
        .get_attribute("enctype")
        .unwrap_or("application/x-www-form-urlencoded");

    let method = FormMethod::from_str(method_attr);
    let enctype = FormEncoding::from_str(enctype_attr);

    // Resolve target URL
    let resolved_url = crate::net::resolve_url(base_url, action_attr);
    let fields = collect_form_fields(dom, form_idx, form_manager);

    match method {
        FormMethod::Get => {
            let query = encode_url_encoded(&fields);
            let final_url = if query.is_empty() {
                resolved_url
            } else if resolved_url.contains('?') {
                format!("{}&{}", resolved_url, query)
            } else {
                format!("{}?{}", resolved_url, query)
            };

            Ok(FormSubmissionPayload {
                target_url: final_url,
                method: FormMethod::Get,
                content_type: None,
                body: Vec::new(),
            })
        }
        FormMethod::Post => match enctype {
            FormEncoding::UrlEncoded => {
                let encoded = encode_url_encoded(&fields);
                Ok(FormSubmissionPayload {
                    target_url: resolved_url,
                    method: FormMethod::Post,
                    content_type: Some("application/x-www-form-urlencoded".to_string()),
                    body: encoded.into_bytes(),
                })
            }
            FormEncoding::Multipart => {
                let boundary = "----ExploidusFormBoundary7MA4YWxkTrZu0gW";
                let body = encode_multipart(&fields, boundary);
                Ok(FormSubmissionPayload {
                    target_url: resolved_url,
                    method: FormMethod::Post,
                    content_type: Some(format!("multipart/form-data; boundary={}", boundary)),
                    body,
                })
            }
            FormEncoding::TextPlain => {
                let mut text_body = String::new();
                for (k, v) in fields {
                    text_body.push_str(&format!("{}={}\r\n", k, v));
                }
                Ok(FormSubmissionPayload {
                    target_url: resolved_url,
                    method: FormMethod::Post,
                    content_type: Some("text/plain".to_string()),
                    body: text_body.into_bytes(),
                })
            }
        },
    }
}
