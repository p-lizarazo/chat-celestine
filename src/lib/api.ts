import { invoke } from '@tauri-apps/api/core';
import { Store } from '@tauri-apps/plugin-store';

// Types matching Rust backend
export enum ProviderType {
  Ollama = 'ollama',
  OpenAI = 'openai',
  Anthropic = 'anthropic',
  Custom = 'custom',
}

export enum MessageRole {
  System = 'system',
  User = 'user',
  Assistant = 'assistant',
}

export interface Message {
  role: MessageRole;
  content: string;
}

export interface ChatRequest {
  model: string;
  messages: Message[];
  temperature?: number;
  max_tokens?: number;
  stream: boolean;
}

export interface ChatResponse {
  id: string;
  model: string;
  message: Message;
  finish_reason?: string;
}

export interface ChatChunk {
  id: string;
  model: string;
  delta: string;
  finish_reason?: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  context_length?: number;
}

export interface ProviderConfig {
  provider_type: ProviderType;
  base_url?: string;
  api_key?: string;
  default_model?: string;
}

// Credential storage using Tauri Store
export class CredentialStore {
  private store: Store | null = null;

  private async getStore(): Promise<Store> {
    if (!this.store) {
      this.store = await Store.load('credentials.json');
    }
    return this.store;
  }

  async saveApiKey(provider: string, apiKey: string): Promise<void> {
    const store = await this.getStore();
    await store.set(`${provider}_api_key`, apiKey);
    await store.save();
  }

  async getApiKey(provider: string): Promise<string | null> {
    const store = await this.getStore();
    const value = await store.get<string>(`${provider}_api_key`);
    return value ?? null;
  }

  async deleteApiKey(provider: string): Promise<void> {
    const store = await this.getStore();
    await store.delete(`${provider}_api_key`);
    await store.save();
  }

  async saveProviderConfig(provider: string, config: ProviderConfig): Promise<void> {
    const store = await this.getStore();
    await store.set(`${provider}_config`, config);
    await store.save();
  }

  async getProviderConfig(provider: string): Promise<ProviderConfig | null> {
    const store = await this.getStore();
    const value = await store.get<ProviderConfig>(`${provider}_config`);
    return value ?? null;
  }
}

// API functions
export async function initializeProvider(config: ProviderConfig): Promise<string> {
  return await invoke<string>('initialize_provider', { config });
}

export async function listModels(providerName: string): Promise<ModelInfo[]> {
  return await invoke<ModelInfo[]>('list_models', { providerName });
}

export async function sendChatMessage(
  providerName: string,
  request: ChatRequest
): Promise<ChatResponse> {
  return await invoke<ChatResponse>('send_chat_message', { providerName, request });
}

export async function checkOllamaAvailable(): Promise<boolean> {
  return await invoke<boolean>('check_ollama_available');
}

export async function sendChatMessageStream(
  providerName: string,
  request: ChatRequest
): Promise<void> {
  return await invoke<void>('send_chat_message_stream', { providerName, request });
}
