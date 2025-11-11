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
  import Card, { Content } from '@smui/card';
  import Button from '@smui/button';
  import Textfield from '@smui/textfield';
  import Select, { Option } from '@smui/select';
  import Radio from '@smui/radio';
  import FormField from '@smui/form-field';
  import Paper, { Title as PaperTitle } from '@smui/paper';

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
  <Card style="background-color: var(--bg-tertiary); padding: 1rem;">
    <Content>
      <h2 style="color: var(--text-primary); margin-bottom: 1.5rem;">Select Provider</h2>

      <div class="provider-type">
        <FormField>
          <Radio bind:group={selectedProvider} value={ProviderType.Ollama} disabled={isLoading} />
          <span slot="label" style="color: var(--text-primary);">
            Ollama (Local)
            {#if ollamaAvailable}
              <span class="status available">●</span>
            {:else}
              <span class="status unavailable">●</span>
            {/if}
          </span>
        </FormField>

        <FormField>
          <Radio bind:group={selectedProvider} value={ProviderType.OpenAI} disabled={isLoading} />
          <span slot="label" style="color: var(--text-primary);">OpenAI (Cloud)</span>
        </FormField>
      </div>

      {#if selectedProvider === ProviderType.OpenAI}
        <Paper variant="outlined" style="padding: 1rem; margin-top: 1rem; background-color: var(--bg-secondary);">
          <div class="form-group">
            <Textfield
              bind:value={openaiApiKey}
              label="API Key"
              type="password"
              disabled={isLoading}
              style="width: 100%; margin-bottom: 1rem;"
              input$placeholder="sk-..."
            />
          </div>

          <div class="form-group">
            <Textfield
              bind:value={openaiBaseUrl}
              label="Base URL (optional)"
              type="text"
              disabled={isLoading}
              style="width: 100%;"
              input$placeholder="https://api.openai.com/v1"
            />
          </div>
        </Paper>
      {/if}

      <div style="margin-top: 1rem;">
        <Button variant="raised" on:click={handleConnect} disabled={isLoading} style="width: 100%;">
          <span>{isLoading ? 'Connecting...' : 'Connect'}</span>
        </Button>
      </div>

      {#if error}
        <Paper variant="outlined" style="padding: 0.75rem; margin-top: 1rem; background-color: var(--mdc-theme-error); color: var(--mdc-theme-on-error);">
          <div class="error">{error}</div>
        </Paper>
      {/if}

      {#if models.length > 0}
        <div class="models-section" style="margin-top: 1rem;">
          <Select bind:value={selectedModel} label="Select Model" style="width: 100%;" on:SMUISelect:change={handleModelChange}>
            {#each models as model}
              <Option value={model.id}>{model.name}</Option>
            {/each}
          </Select>
        </div>
      {/if}
    </Content>
  </Card>
</div>

<style>
  .provider-selector {
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

  .status {
    margin-left: 0.5rem;
    font-size: 0.75rem;
  }

  .status.available {
    color: var(--status-available);
  }

  .status.unavailable {
    color: var(--status-unavailable);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .error {
    font-size: 0.875rem;
  }

  .models-section {
    margin-top: 1rem;
  }
</style>
