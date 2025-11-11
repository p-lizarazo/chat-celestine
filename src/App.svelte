<script lang="ts">
  import Chat from './lib/Chat.svelte';
  import ProviderSelector from './lib/ProviderSelector.svelte';

  let currentProvider = '';
  let currentModel = '';
  let showSettings = true;

  function handleProviderSelected(provider: string, model: string) {
    currentProvider = provider;
    currentModel = model;
    showSettings = false;
  }

  function openSettings() {
    showSettings = true;
  }
</script>

<main>
  <header>
    <h1>Celestine Chat</h1>
    {#if currentProvider}
      <button class="settings-btn" on:click={openSettings}>Settings</button>
    {/if}
  </header>

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
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 2rem;
    border-bottom: 1px solid #333;
    background: #1f2937;
  }

  h1 {
    font-size: 1.5rem;
    margin: 0;
  }

  .settings-btn {
    padding: 0.5rem 1rem;
    background: #374151;
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
    transition: background 0.2s;
  }

  .settings-btn:hover {
    background: #4b5563;
  }

  .container {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 400px;
    padding: 2rem;
    border-right: 1px solid #333;
    background: #111827;
    overflow-y: auto;
  }

  .chat-area {
    flex: 1;
    overflow: hidden;
  }

  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #6b7280;
  }

  .welcome h2 {
    font-size: 2rem;
    margin-bottom: 0.5rem;
  }

  .welcome p {
    font-size: 1.1rem;
  }
</style>
