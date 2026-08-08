#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

#[derive(Debug, Clone)]
pub struct WebSocketFrame {
    pub fin: bool,
    pub opcode: Opcode,
    pub payload: Vec<u8>,
}

impl WebSocketFrame {
    pub fn new_text(text: &str) -> Self {
        WebSocketFrame {
            fin: true,
            opcode: Opcode::Text,
            payload: text.as_bytes().to_vec(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let first_byte = (if self.fin { 0x80 } else { 0x00 }) | (self.opcode.clone() as u8);
        buf.push(first_byte);

        let len = self.payload.len();
        if len <= 125 {
            buf.push(len as u8);
        } else if len <= 65535 {
            buf.push(126);
            buf.push(((len >> 8) & 0xFF) as u8);
            buf.push((len & 0xFF) as u8);
        }
        buf.extend_from_slice(&self.payload);
        buf
    }
}

pub struct WebSocketClient {
    pub url: String,
    pub connected: bool,
}

impl WebSocketClient {
    pub fn connect(url: &str) -> Result<Self, String> {
        println!("WebSocket Engine: Connected to {}", url);
        Ok(WebSocketClient {
            url: url.to_string(),
            connected: true,
        })
    }

    pub fn send_text(&mut self, text: &str) -> Result<(), String> {
        let frame = WebSocketFrame::new_text(text);
        println!("WebSocket Sent Frame: {:?}", frame);
        Ok(())
    }
}
