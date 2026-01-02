use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, info};

pub struct NntpClient {
    stream: Framed<TcpStream, LinesCodec>,
}

#[derive(Debug)]
pub struct GroupInfo {
    pub number: u64,
    pub low: u64,
    pub high: u64,
    pub name: String,
}

impl NntpClient {
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        info!("Connecting to NNTP server at {}", addr);
        let stream = TcpStream::connect(addr).await?;
        let mut framed = Framed::new(stream, LinesCodec::new());

        let response = framed
            .next()
            .await
            .ok_or_else(|| anyhow!("Connection closed during handshake"))??;

        if !response.starts_with("200") && !response.starts_with("201") {
            return Err(anyhow!("Unexpected welcome message: {}", response));
        }

        debug!("Connected: {}", response);
        Ok(Self { stream: framed })
    }

    pub async fn group(&mut self, group_name: &str) -> Result<GroupInfo> {
        let command = format!("GROUP {}", group_name);
        self.stream.send(command).await?;

        let response = self.stream
            .next()
            .await
            .ok_or_else(|| anyhow!("Connection closed"))??;

        if !response.starts_with("211") {
            return Err(anyhow!("Failed to select group {}: {}", group_name, response));
        }

        let parts: Vec<&str> = response.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(anyhow!("Invalid GROUP response format: {}", response));
        }

        Ok(GroupInfo {
            number: parts[1].parse().unwrap_or(0),
            low: parts[2].parse().unwrap_or(0),
            high: parts[3].parse().unwrap_or(0),
            name: parts[4].to_string(),
        })
    }

    pub async fn article(&mut self, id: &str) -> Result<Vec<String>> {
        // id can be Message-ID (with <>) or article number
        let command = format!("ARTICLE {}", id);
        self.stream.send(command).await?;

        let response = self.stream
            .next()
            .await
            .ok_or_else(|| anyhow!("Connection closed"))??;

        if !response.starts_with("220") {
             return Err(anyhow!("Failed to retrieve article {}: {}", id, response));
        }

        let mut lines = Vec::new();
        while let Some(line) = self.stream.next().await {
            let line = line?;
            if line == "." {
                break;
            }
            // Dot-unstuffing: if line starts with "..", remove one dot
            let line = if line.starts_with("..") {
                line[1..].to_string()
            } else {
                line
            };
            lines.push(line);
        }

        Ok(lines)
    }

    pub async fn quit(&mut self) -> Result<()> {
        self.stream.send("QUIT").await?;
        let response = self.stream
            .next()
            .await
            .ok_or_else(|| anyhow!("Connection closed"))??;
        
        if !response.starts_with("205") {
            debug!("QUIT response was not 205: {}", response);
        }
        Ok(())
    }
}
