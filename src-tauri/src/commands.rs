use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use futures::StreamExt;

use crate::llm::{
    providers::{OllamaProvider, OpenAIProvider},
    ChatRequest, ChatResponse, LlmProvider, ModelInfo, ProviderConfig, ProviderType, ChatChunk,
};

pub struct AppState {
    pub providers: Arc<RwLock<Vec<Arc<dyn LlmProvider>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[tauri::command]
pub async fn initialize_provider(
    config: ProviderConfig,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let provider: Arc<dyn LlmProvider> = match config.provider_type {
        ProviderType::Ollama => {
            Arc::new(OllamaProvider::new(config.base_url))
        }
        ProviderType::OpenAI => {
            let api_key = config.api_key.ok_or("OpenAI requires an API key")?;
            Arc::new(OpenAIProvider::new(api_key, config.base_url))
        }
        _ => return Err("Provider type not yet supported".to_string()),
    };

    let provider_name = provider.name().to_string();

    // Check if provider is available
    if !provider.is_available().await {
        return Err(format!("Provider {} is not available", provider_name));
    }

    // Add to state
    state.providers.write().await.push(provider);

    Ok(provider_name)
}

#[tauri::command]
pub async fn list_models(
    provider_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<ModelInfo>, String> {
    let providers = state.providers.read().await;

    let provider = providers
        .iter()
        .find(|p| p.name() == provider_name)
        .ok_or("Provider not found")?;

    provider
        .list_models()
        .await
        .map_err(|e| format!("Failed to list models: {}", e))
}

#[tauri::command]
pub async fn send_chat_message(
    provider_name: String,
    request: ChatRequest,
    state: State<'_, AppState>,
) -> Result<ChatResponse, String> {
    let providers = state.providers.read().await;

    let provider = providers
        .iter()
        .find(|p| p.name() == provider_name)
        .ok_or("Provider not found")?;

    provider
        .chat_completion(request)
        .await
        .map_err(|e| format!("Chat completion failed: {}", e))
}

#[tauri::command]
pub async fn check_ollama_available() -> Result<bool, String> {
    let provider = OllamaProvider::new(None);
    Ok(provider.is_available().await)
}

#[tauri::command]
pub async fn send_chat_message_stream(
    provider_name: String,
    request: ChatRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let providers = state.providers.read().await;

    let provider = providers
        .iter()
        .find(|p| p.name() == provider_name)
        .ok_or("Provider not found")?;

    let mut stream = provider
        .chat_completion_stream(request)
        .await
        .map_err(|e| format!("Stream initialization failed: {}", e))?;

    // Emit chunks as they arrive
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                app.emit("chat-chunk", &chunk)
                    .map_err(|e| format!("Failed to emit chunk: {}", e))?;
            }
            Err(e) => {
                app.emit("chat-error", &format!("{}", e))
                    .map_err(|e| format!("Failed to emit error: {}", e))?;
                return Err(format!("Stream error: {}", e));
            }
        }
    }

    Ok(())
}
