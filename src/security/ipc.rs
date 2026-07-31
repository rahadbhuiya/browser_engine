use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, PartialEq)]
pub enum IpcMessage {
    FetchUrl { url: String },
    RenderFrame { commands_count: usize },
    MutateDom { node_id: usize, text: String },
    SecurityViolation { policy: String, detail: String },
}

pub struct IpcChannel {
    pub tx: Sender<IpcMessage>,
    pub rx: Receiver<IpcMessage>,
}

impl IpcChannel {
    pub fn new() -> (Self, Self) {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();

        let endpoint1 = IpcChannel { tx: tx1, rx: rx2 };
        let endpoint2 = IpcChannel { tx: tx2, rx: rx1 };

        (endpoint1, endpoint2)
    }

    pub fn send(&self, msg: IpcMessage) -> Result<(), String> {
        self.tx.send(msg).map_err(|e| e.to_string())
    }

    pub fn try_recv(&self) -> Option<IpcMessage> {
        self.rx.try_recv().ok()
    }
}
