pub mod providers;
pub mod types;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use types::*;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),
}

pub type LlmResult<T> = Result<T, LlmError>;

/// Core trait that all LLM providers must implement
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the name of this provider
    fn name(&self) -> &str;

    /// Check if this provider is available (e.g., local server running, API key valid)
    async fn is_available(&self) -> bool;

    /// Send a chat completion request
    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ChatResponse>;

    /// Stream a chat completion response
    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> LlmResult<futures::stream::BoxStream<'static, LlmResult<ChatChunk>>>;

    /// List available models for this provider
    async fn list_models(&self) -> LlmResult<Vec<ModelInfo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_length: Option<usize>,
}
