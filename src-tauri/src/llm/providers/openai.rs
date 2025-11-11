use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{FramedRead, LinesCodec};

use crate::llm::{
    ChatChunk, ChatRequest, ChatResponse, LlmError, LlmProvider, LlmResult, Message, MessageRole,
    ModelInfo,
};

pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIDelta {
    #[serde(default)]
    content: Option<String>,
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn is_available(&self) -> bool {
        // Try to list models to verify API key is valid
        self.list_models().await.is_ok()
    }

    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ChatResponse> {
        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OpenAIMessage {
                    role: m.role.clone().into(),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!("OpenAI API error: {}", error_text)));
        }

        let openai_response: OpenAIChatResponse = response.json().await?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| LlmError::Provider("No choices in response".to_string()))?;

        Ok(ChatResponse {
            id: openai_response.id,
            model: openai_response.model,
            message: Message {
                role: MessageRole::Assistant,
                content: choice.message.content.clone(),
            },
            finish_reason: choice.finish_reason.clone(),
        })
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> LlmResult<BoxStream<'static, LlmResult<ChatChunk>>> {
        let openai_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OpenAIMessage {
                    role: m.role.clone().into(),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!("OpenAI API error: {}", error_text)));
        }

        // Process the SSE stream
        let stream = response.bytes_stream();
        let stream = stream.map(|result| {
            result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
        
        let reader = tokio_util::io::StreamReader::new(stream);
        let lines = FramedRead::new(reader, LinesCodec::new());

        let chunk_stream = lines.filter_map(|line_result| async move {
            match line_result {
                Ok(line) => {
                    // OpenAI uses SSE format: "data: {json}"
                    if line.trim().is_empty() || line == "data: [DONE]" {
                        return None;
                    }
                    
                    let json_str = line.strip_prefix("data: ").unwrap_or(&line);
                    
                    match serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                        Ok(stream_resp) => {
                            let choice = stream_resp.choices.first()?;
                            let delta = choice.delta.content.clone().unwrap_or_default();
                            
                            Some(Ok(ChatChunk {
                                id: stream_resp.id,
                                model: stream_resp.model,
                                delta,
                                finish_reason: choice.finish_reason.clone(),
                            }))
                        }
                        Err(_) => None, // Skip invalid JSON
                    }
                }
                Err(e) => Some(Err(LlmError::Provider(format!("Stream error: {}", e)))),
            }
        });

        Ok(chunk_stream.boxed())
    }

    async fn list_models(&self) -> LlmResult<Vec<ModelInfo>> {
        // Return a static list of common OpenAI models
        // In a real implementation, you'd call the /models endpoint
        Ok(vec![
            ModelInfo {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                provider: "OpenAI".to_string(),
                context_length: Some(8192),
            },
            ModelInfo {
                id: "gpt-4-turbo-preview".to_string(),
                name: "GPT-4 Turbo".to_string(),
                provider: "OpenAI".to_string(),
                context_length: Some(128000),
            },
            ModelInfo {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
                provider: "OpenAI".to_string(),
                context_length: Some(16385),
            },
        ])
    }
}
