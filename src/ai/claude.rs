// Copyright 2026 The Sashiko Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::ai::{
    AiProvider, AiRequest, AiResponse, AiRole, AiUsage, ProviderCapabilities, ToolCall,
};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{error, warn};

// --- Claude API Request/Response Types ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeMessage {
    pub role: String, // "user" or "assistant"
    pub content: Vec<ClaudeContent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeContent {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub cache_type: String, // "ephemeral"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemBlock {
    #[serde(rename = "type")]
    pub block_type: String, // "text"
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub messages: Vec<ClaudeMessage>,
    pub max_tokens: u32, // Required by Claude API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<SystemBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ClaudeTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ClaudeContent>,
    pub stop_reason: Option<String>,
    pub usage: ClaudeUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeErrorResponse {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: ClaudeErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeErrorDetails {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

// --- Error Types ---

#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("Rate limit exceeded, retry after {0:?}")]
    RateLimitExceeded(Duration),
    #[error("API overloaded, retry after {0:?}")]
    OverloadedError(Duration),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("API error {0}: {1}")]
    ApiError(reqwest::StatusCode, String),
}

// --- ClaudeClient ---

pub struct ClaudeClient {
    api_key: String,
    model: String,
    client: Client,
    enable_caching: bool,
}

impl ClaudeClient {
    pub fn new(model: String, enable_caching: bool) -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .or_else(|_| std::env::var("LLM_API_KEY"))
            .unwrap_or_default();

        Self {
            api_key,
            model,
            client: Client::new(),
            enable_caching,
        }
    }

    async fn post_request(&self, body: &ClaudeRequest) -> Result<ClaudeResponse> {
        let url = "https://api.anthropic.com/v1/messages";

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            self.api_key.parse().context("Invalid API key format")?,
        );
        headers.insert(
            "anthropic-version",
            "2023-06-01"
                .parse()
                .context("Invalid anthropic-version header")?,
        );
        headers.insert(
            "content-type",
            "application/json"
                .parse()
                .context("Invalid content-type header")?,
        );

        let res = self
            .client
            .post(url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        let status = res.status();

        if status.is_success() {
            let response: ClaudeResponse = res
                .json()
                .await
                .context("Failed to parse Claude API response")?;
            Ok(response)
        } else {
            let error_body = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            match status.as_u16() {
                429 => {
                    // Rate limit - extract retry-after if present
                    let duration = Duration::from_secs(60); // Default to 60s
                    Err(ClaudeError::RateLimitExceeded(duration))?
                }
                529 => {
                    // Overloaded - use exponential backoff
                    let duration = Duration::from_secs(5); // Start with 5s
                    Err(ClaudeError::OverloadedError(duration))?
                }
                400 => Err(ClaudeError::InvalidRequest(error_body))?,
                401 | 403 => Err(ClaudeError::AuthenticationError(error_body))?,
                _ => Err(ClaudeError::ApiError(status, error_body))?,
            }
        }
    }
}
