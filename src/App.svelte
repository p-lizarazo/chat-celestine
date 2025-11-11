<script lang="ts">
  import Chat from './lib/Chat.svelte';
  import ProviderSelector from './lib/ProviderSelector.svelte';
  import Button from '@smui/button';
  import IconButton from '@smui/icon-button';
  import TopAppBar, { Row, Section, Title } from '@smui/top-app-bar';
  import { theme } from './lib/theme';

  let currentProvider = '';
  let currentModel = '';
  let showSettings = true;
  let currentTheme: 'light' | 'dark' = 'dark';
  
  theme.subscribe(value => {
    currentTheme = value;
  });

  function handleProviderSelected(provider: string, model: string) {
    currentProvider = provider;
    currentModel = model;
    showSettings = false;
  }

  function openSettings() {
    showSettings = true;
  }
  
  function toggleTheme() {
    theme.toggle();
  }
</script>

<main>
  <TopAppBar variant="fixed" style="background-color: var(--bg-secondary); color: var(--text-primary); border-bottom: 1px solid var(--border-color);">
    <Row>
      <Section>
        <Title style="color: var(--text-primary);">Celestine Chat</Title>
      </Section>
      <Section align="end" toolbar>
        <IconButton 
          class="material-icons" 
          on:click={toggleTheme}
          title={currentTheme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
          style="color: var(--text-primary);"
        >
          {currentTheme === 'dark' ? 'light_mode' : 'dark_mode'}
        </IconButton>
        {#if currentProvider}
          <Button on:click={openSettings} variant="raised">Settings</Button>
        {/if}
      </Section>
    </Row>
  </TopAppBar>

  <div class="app-content">
    <div class="container">
      {#if showSettings || !currentProvider}
        <div class="sidebar">
          <ProviderSelector onProviderSelected={handleProviderSelected} />
        </div>
      {/if}

      {#if currentProvider && currentModel}
        <div class="chat-area">
          <Chat providerName={currentProvider} selectedModel={currentModel} />
        </div>
      {:else}
        <div class="welcome">
          <h2>Welcome to Celestine Chat</h2>
          <p>Connect to a provider to start chatting</p>
        </div>
      {/if}
    </div>
  </div>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: var(--bg-primary);
  }

  .app-content {
    margin-top: 64px; /* Height of TopAppBar */
    height: calc(100vh - 64px);
    overflow: hidden;
  }

  .container {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .sidebar {
    width: 400px;
    padding: 2rem;
    border-right: 1px solid var(--border-color);
    background-color: var(--bg-secondary);
    overflow-y: auto;
  }

  .chat-area {
    flex: 1;
    overflow: hidden;
    background-color: var(--bg-primary);
  }

  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    background-color: var(--bg-primary);
  }

  .welcome h2 {
    font-size: 2rem;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .welcome p {
    font-size: 1.1rem;
    color: var(--text-secondary);
  }
</style>
