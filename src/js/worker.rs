use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub struct WebWorker {
    pub script_url: String,
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl WebWorker {
    pub fn new(script_url: &str) -> Self {
        let (tx_main, rx_worker) = channel::<String>();
        let (tx_worker, rx_main) = channel::<String>();
        let script = script_url.to_string();

        thread::spawn(move || {
            println!("WebWorker background thread started for: {}", script);
            while let Ok(msg) = rx_worker.recv() {
                println!("WebWorker received msg: {}", msg);
                let _ = tx_worker.send(format!("processed: {}", msg));
            }
        });

        WebWorker {
            script_url: script_url.to_string(),
            sender: tx_main,
            receiver: rx_main,
        }
    }

    pub fn post_message(&self, msg: &str) -> Result<(), String> {
        self.sender.send(msg.to_string()).map_err(|e| e.to_string())
    }

    pub fn try_recv(&self) -> Option<String> {
        self.receiver.try_recv().ok()
    }
}
