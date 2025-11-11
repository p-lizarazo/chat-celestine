<script lang="ts">
  import { onMount } from 'svelte';
  import {
    checkOllamaAvailable,
    initializeProvider,
    listModels,
    CredentialStore,
    type ModelInfo,
    type ProviderConfig,
    ProviderType,
  } from './api';

  export let onProviderSelected: (provider: string, model: string) => void;

  let credStore = new CredentialStore();
  let selectedProvider = ProviderType.Ollama;
  let ollamaAvailable = false;
  let models: ModelInfo[] = [];
  let selectedModel = '';
  let isLoading = false;
  let error = '';

  // OpenAI credentials
  let openaiApiKey = '';
  let openaiBaseUrl = '';

  onMount(async () => {
    await checkProviders();
    await loadSavedCredentials();
  });

  async function checkProviders() {
    ollamaAvailable = await checkOllamaAvailable();
  }

  async function loadSavedCredentials() {
    const openaiKey = await credStore.getApiKey('openai');
    if (openaiKey) {
      openaiApiKey = openaiKey;
    }

    const openaiConfig = await credStore.getProviderConfig('openai');
    if (openaiConfig?.base_url) {
      openaiBaseUrl = openaiConfig.base_url;
    }
  }

  async function handleConnect() {
    isLoading = true;
    error = '';
    models = [];
    selectedModel = '';

    try {
      const config: ProviderConfig = {
        provider_type: selectedProvider,
      };

      if (selectedProvider === ProviderType.OpenAI) {
        if (!openaiApiKey.trim()) {
          throw new Error('OpenAI API key is required');
        }
        config.api_key = openaiApiKey.trim();
        config.base_url = openaiBaseUrl.trim() || undefined;

        // Save credentials
        await credStore.saveApiKey('openai', openaiApiKey.trim());
        if (openaiBaseUrl.trim()) {
          await credStore.saveProviderConfig('openai', config);
        }
      } else if (selectedProvider === ProviderType.Ollama) {
        config.base_url = 'http://localhost:11434';
      }

      const providerName = await initializeProvider(config);
      const availableModels = await listModels(providerName);
      models = availableModels;

      if (models.length > 0) {
        selectedModel = models[0].id;
        onProviderSelected(providerName, selectedModel);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      console.error('Provider connection error:', e);
    } finally {
      isLoading = false;
    }
  }

  function handleModelChange() {
    if (selectedModel) {
      const providerName = selectedProvider === ProviderType.Ollama ? 'Ollama' : 'OpenAI';
      onProviderSelected(providerName, selectedModel);
    }
  }
</script>

<div class="provider-selector">
  <h2>Select Provider</h2>

  <div class="provider-type">
    <label>
      <input
        type="radio"
        bind:group={selectedProvider}
        value={ProviderType.Ollama}
        disabled={isLoading}
      />
      Ollama (Local)
      {#if ollamaAvailable}
        <span class="status available">●</span>
      {:else}
        <span class="status unavailable">●</span>
      {/if}
    </label>

    <label>
      <input
        type="radio"
        bind:group={selectedProvider}
        value={ProviderType.OpenAI}
        disabled={isLoading}
      />
      OpenAI (Cloud)
    </label>
  </div>

  {#if selectedProvider === ProviderType.OpenAI}
    <div class="provider-config">
      <div class="form-group">
        <label for="openai-key">API Key</label>
        <input
          id="openai-key"
          type="password"
          bind:value={openaiApiKey}
          placeholder="sk-..."
          disabled={isLoading}
        />
      </div>

      <div class="form-group">
        <label for="openai-url">Base URL (optional)</label>
        <input
          id="openai-url"
          type="text"
          bind:value={openaiBaseUrl}
          placeholder="https://api.openai.com/v1"
          disabled={isLoading}
        />
      </div>
    </div>
  {/if}

  <button class="connect-btn" on:click={handleConnect} disabled={isLoading}>
    {isLoading ? 'Connecting...' : 'Connect'}
  </button>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  {#if models.length > 0}
    <div class="models-section">
      <label for="model-select">Select Model</label>
      <select id="model-select" bind:value={selectedModel} on:change={handleModelChange}>
        {#each models as model}
          <option value={model.id}>{model.name}</option>
        {/each}
      </select>
    </div>
  {/if}
</div>

<style>
  .provider-selector {
    padding: 1.5rem;
    background: #1f2937;
    border-radius: 8px;
    max-width: 500px;
  }

  h2 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
  }

  .provider-type {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }

  .provider-type label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 4px;
    transition: background 0.2s;
  }

  .provider-type label:hover {
    background: #374151;
  }

  .status {
    margin-left: auto;
    font-size: 0.75rem;
  }

  .status.available {
    color: #10b981;
  }

  .status.unavailable {
    color: #6b7280;
  }

  .provider-config {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-bottom: 1rem;
    padding: 1rem;
    background: #111827;
    border-radius: 4px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .form-group label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  input[type='text'],
  input[type='password'] {
    padding: 0.5rem;
    background: #1f2937;
    border: 1px solid #374151;
    border-radius: 4px;
    color: white;
    font-family: inherit;
  }

  input[type='text']:focus,
  input[type='password']:focus {
    outline: none;
    border-color: #2563eb;
  }

  .connect-btn {
    width: 100%;
    padding: 0.75rem;
    background: #2563eb;
    border: none;
    border-radius: 4px;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .connect-btn:hover:not(:disabled) {
    background: #1d4ed8;
  }

  .connect-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    margin-top: 1rem;
    padding: 0.75rem;
    background: #dc2626;
    border-radius: 4px;
    color: white;
    font-size: 0.875rem;
  }

  .models-section {
    margin-top: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .models-section label {
    font-size: 0.875rem;
    font-weight: 500;
  }

  select {
    padding: 0.5rem;
    background: #111827;
    border: 1px solid #374151;
    border-radius: 4px;
    color: white;
    font-family: inherit;
    cursor: pointer;
  }

  select:focus {
    outline: none;
    border-color: #2563eb;
  }
</style>
