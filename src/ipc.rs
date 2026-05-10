use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::broadcast;
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
}

impl IpcBroker {
    pub async fn start(sock_path: &Path) -> Result<Self> {
        if sock_path.exists() {
            tokio::fs::remove_file(sock_path).await?;
        }
        let listener = UnixListener::bind(sock_path)?;
        let (tx, _) = broadcast::channel(100);

        info!("IPC Broker started at {:?}", sock_path);
        Ok(Self { listener, tx })
    }

    pub async fn run(self) {
        let tx = self.tx;
        loop {
            if let Ok((mut stream, _)) = self.listener.accept().await {
                let mut rx = tx.subscribe();
                let tx_clone = tx.clone();
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
                                        // Broadcast the line to all subscribers
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
