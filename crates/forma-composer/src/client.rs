/// Thin wrapper around the Anthropic Messages API v1.
/// Reads ANTHROPIC_API_KEY from the environment.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("ANTHROPIC_API_KEY is not set. Export it before running compose commands.")]
    MissingApiKey,
    #[error("Invalid header value: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Failed to parse API response: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("API returned error status {status}: {body}")]
    ApiError { status: u16, body: String },
    #[error("Unexpected content block type: {0}")]
    UnexpectedBlock(String),
}

pub struct FormaClient {
    http: reqwest::blocking::Client,
    model: String,
    max_tokens: u32,
}

impl FormaClient {
    pub fn new(model: &str, max_tokens: u32) -> Result<Self, ClientError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| ClientError::MissingApiKey)?;

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {api_key}"))?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(Self {
            http: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()?,
            model: model.to_string(),
            max_tokens,
        })
    }

    pub fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String, ClientError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": user_prompt }
            ]
        });

        let resp = self.http.post("https://api.anthropic.com/v1/messages")
            .json(&body)
            .send()?;

        let status = resp.status();
        let body_text = resp.text()?;

        if !status.is_success() {
            return Err(ClientError::ApiError {
                status: status.as_u16(),
                body: body_text.clone(),
            });
        }

        #[derive(serde::Deserialize)]
        struct ApiResponse {
            content: Vec<ContentBlock>,
        }

        #[derive(serde::Deserialize)]
        struct ContentBlock {
            #[serde(rename = "type")]
            block_type: String,
            #[serde(default)]
            text: String,
        }

        let api_resp: ApiResponse = serde_json::from_str(&body_text)?;

        let text_block = api_resp.content.into_iter()
            .find(|b| b.block_type == "text")
            .ok_or_else(|| ClientError::UnexpectedBlock(
                "no text content block in response".into()
            ))?;

        Ok(text_block.text)
    }
}
