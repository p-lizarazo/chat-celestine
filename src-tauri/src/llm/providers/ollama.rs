use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{FramedRead, LinesCodec};

use crate::llm::{
    ChatChunk, ChatRequest, ChatResponse, LlmError, LlmProvider, LlmResult, Message, MessageRole,
    ModelInfo,
};

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaMessage,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamResponse {
    model: String,
    #[serde(default)]
    message: Option<OllamaMessage>,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaListResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    model: String,
    size: u64,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "Ollama"
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }

    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ChatResponse> {
        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: m.role.clone().into(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };

        let response = self
            .client
            .post(&format!("{}/api/chat", self.base_url))
            .json(&ollama_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LlmError::Provider(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaChatResponse = response.json().await?;

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: ollama_response.model,
            message: Message {
                role: MessageRole::Assistant,
                content: ollama_response.message.content,
            },
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> LlmResult<BoxStream<'static, LlmResult<ChatChunk>>> {
        let ollama_request = OllamaChatRequest {
            model: request.model.clone(),
            messages: request
                .messages
                .iter()
                .map(|m| OllamaMessage {
                    role: m.role.clone().into(),
                    content: m.content.clone(),
                })
                .collect(),
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };

        let response = self
            .client
            .post(&format!("{}/api/chat", self.base_url))
            .json(&ollama_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LlmError::Provider(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let id = uuid::Uuid::new_v4().to_string();
        
        // Process the stream by reading bytes and splitting into lines
        let stream = response.bytes_stream();
        let stream = stream.map(|result| {
            result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
        
        let reader = tokio_util::io::StreamReader::new(stream);
        let lines = FramedRead::new(reader, LinesCodec::new());

        let chunk_stream = lines.filter_map(move |line_result| {
            let id = id.clone();
            async move {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            return None;
                        }
                        match serde_json::from_str::<OllamaStreamResponse>(&line) {
                            Ok(stream_resp) => {
                                let delta = stream_resp
                                    .message
                                    .as_ref()
                                    .map(|m| m.content.clone())
                                    .unwrap_or_default();
                                
                                Some(Ok(ChatChunk {
                                    id: id.clone(),
                                    model: stream_resp.model,
                                    delta,
                                    finish_reason: if stream_resp.done {
                                        Some("stop".to_string())
                                    } else {
                                        None
                                    },
                                }))
                            }
                            Err(e) => Some(Err(LlmError::Serialization(e))),
                        }
                    }
                    Err(e) => Some(Err(LlmError::Provider(format!("Stream error: {}", e)))),
                }
            }
        });

        Ok(chunk_stream.boxed())
    }

    async fn list_models(&self) -> LlmResult<Vec<ModelInfo>> {
        let response = self
            .client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(LlmError::Provider(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let list_response: OllamaListResponse = response.json().await?;

        Ok(list_response
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.name,
                provider: "Ollama".to_string(),
                context_length: None,
            })
            .collect())
    }
}
