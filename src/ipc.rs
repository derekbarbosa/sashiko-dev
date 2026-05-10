use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcMessage {
    pub patch_id: String,
    pub event: IpcEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum IpcEvent {
    Request {
        symbol: String,
    },
    Response {
        symbol: String,
        implementation_details: Option<String>,
    },
    Broadcast {
        symbol: String,
    },
}

pub struct IpcBroker {
    listener: UnixListener,
    tx: broadcast::Sender<String>,
    state: Arc<RwLock<HashMap<String, String>>>,
}

impl IpcBroker {
    pub async fn start(sock_path: &Path) -> Result<Self> {
        if sock_path.exists() {
            tokio::fs::remove_file(sock_path).await?;
        }
        let listener = UnixListener::bind(sock_path)?;
        let (tx, _) = broadcast::channel(1000);
        let state = Arc::new(RwLock::new(HashMap::new()));

        info!("IPC Broker started at {:?}", sock_path);
        Ok(Self {
            listener,
            tx,
            state,
        })
    }

    pub async fn run(self) {
        let tx = self.tx;
        let state = self.state;

        loop {
            if let Ok((mut stream, _)) = self.listener.accept().await {
                let mut rx = tx.subscribe();
                let tx_clone = tx.clone();
                let state_clone = state.clone();

                tokio::spawn(async move {
                    let (reader, mut writer) = stream.split();
                    let mut reader = BufReader::new(reader);
                    let mut line = String::new();

                    loop {
                        tokio::select! {
                            result = reader.read_line(&mut line) => {
                                match result {
                                    Ok(0) => break, // EOF
                                    Ok(_) => {
                                        // 1. Update internal state based on broadcast/response
                                        if let Ok(msg) = serde_json::from_str::<IpcMessage>(&line) {
                                            match &msg.event {
                                                IpcEvent::Broadcast { symbol } | IpcEvent::Response { symbol, .. } => {
                                                    let mut s = state_clone.write().await;
                                                    s.insert(symbol.clone(), msg.patch_id.clone());
                                                }
                                                IpcEvent::Request { symbol } => {
                                                    // 2. If Broker knows the answer, send it back directly to this client
                                                    let s = state_clone.read().await;
                                                    if let Some(pid) = s.get(symbol) {
                                                        let response = IpcMessage {
                                                            patch_id: pid.clone(),
                                                            event: IpcEvent::Response {
                                                                symbol: symbol.clone(),
                                                                implementation_details: Some(format!("Broker cache: Found in Patch {}", pid)),
                                                            }
                                                        };
                                                        if let Ok(resp_line) = serde_json::to_string(&response) {
                                                            let _ = writer.write_all(format!("{}\n", resp_line).as_bytes()).await;
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // 3. Fan-out to all other subscribers
                                        let _ = tx_clone.send(line.clone());
                                        line.clear();
                                    }
                                    Err(e) => {
                                        error!("IPC read error: {}", e);
                                        break;
                                    }
                                }
                            }
                            msg_result = rx.recv() => {
                                match msg_result {
                                    Ok(msg) => {
                                        if writer.write_all(msg.as_bytes()).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }
                });
            }
        }
    }
}
