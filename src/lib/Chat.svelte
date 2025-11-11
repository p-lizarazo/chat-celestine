<script lang="ts">
  import { sendChatMessage, type ChatRequest, type Message, MessageRole, type ChatResponse } from './api';
  import Button from '@smui/button';
  import Paper from '@smui/paper';
  import Card, { Content } from '@smui/card';

  export let providerName: string = '';
  export let selectedModel: string = '';

  let messages: Message[] = [];
  let userInput = '';
  let isLoading = false;
  let error = '';

  async function handleSend() {
    if (!userInput.trim() || !providerName || !selectedModel || isLoading) return;

    const userMessage: Message = {
      role: MessageRole.User,
      content: userInput.trim(),
    };

    messages = [...messages, userMessage];
    userInput = '';
    isLoading = true;
    error = '';

    try {
      const request: ChatRequest = {
        model: selectedModel,
        messages: messages,
        temperature: 0.7,
        stream: false,
      };

      const response: ChatResponse = await sendChatMessage(providerName, request);
      messages = [...messages, response.message];
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      console.error('Chat error:', e);
    } finally {
      isLoading = false;
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  }

  function clearChat() {
    messages = [];
    error = '';
  }
</script>

<div class="chat-container">
  <div class="chat-header">
    <h2 style="color: var(--text-primary);">Chat</h2>
    {#if messages.length > 0}
      <Button variant="outlined" on:click={clearChat}>Clear</Button>
    {/if}
  </div>

  <div class="messages-container">
    {#if messages.length === 0}
      <div class="empty-state">
        <p>Start a conversation</p>
        <p class="hint">Using {providerName} - {selectedModel}</p>
      </div>
    {:else}
      {#each messages as message}
        <Paper 
          elevation={2} 
          class="message {message.role}"
          style="
            background-color: {message.role === 'user' ? 'var(--user-message-bg)' : 'var(--assistant-message-bg)'};
            color: {message.role === 'user' ? 'var(--user-message-text)' : 'var(--assistant-message-text)'};
          "
        >
          <div class="message-role">{message.role}</div>
          <div class="message-content">{message.content}</div>
        </Paper>
      {/each}
    {/if}

    {#if isLoading}
      <Paper elevation={2} class="message assistant loading" style="background-color: var(--assistant-message-bg); color: var(--assistant-message-text);">
        <div class="message-role">assistant</div>
        <div class="message-content">Thinking...</div>
      </Paper>
    {/if}

    {#if error}
      <Paper elevation={3} style="padding: 1rem; background-color: var(--mdc-theme-error); color: var(--mdc-theme-on-error); margin: 1rem 0;">
        <strong>Error:</strong> {error}
      </Paper>
    {/if}
  </div>

  <Paper elevation={1} class="input-container" style="background-color: var(--bg-secondary);">
    <textarea
      bind:value={userInput}
      on:keydown={handleKeyDown}
      placeholder="Type your message... (Shift+Enter for new line)"
      rows="3"
      disabled={isLoading || !providerName || !selectedModel}
      style="background-color: var(--bg-primary); color: var(--text-primary); border-color: var(--border-color);"
    ></textarea>
    <Button 
      variant="raised" 
      on:click={handleSend}
      disabled={!userInput.trim() || isLoading || !providerName || !selectedModel}
    >
      <span>Send</span>
    </Button>
  </Paper>
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-width: 900px;
    margin: 0 auto;
    background-color: var(--bg-primary);
  }

  .chat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
    background-color: var(--bg-secondary);
  }

  .chat-header h2 {
    margin: 0;
    font-size: 1.5rem;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    background-color: var(--bg-primary);
  }

  .empty-state {
    text-align: center;
    color: var(--text-secondary);
    margin-top: 4rem;
  }

  .empty-state .hint {
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }

  :global(.message) {
    padding: 1rem;
    border-radius: 12px;
    max-width: 80%;
  }

  :global(.message.user) {
    align-self: flex-end;
    margin-left: auto;
  }

  :global(.message.assistant) {
    align-self: flex-start;
  }

  :global(.message.system) {
    align-self: center;
    font-style: italic;
  }

  .message-role {
    font-size: 0.75rem;
    text-transform: uppercase;
    font-weight: 600;
    margin-bottom: 0.5rem;
    opacity: 0.7;
  }

  .message-content {
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  :global(.loading) {
    opacity: 0.7;
  }

  :global(.input-container) {
    padding: 1rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    gap: 0.5rem;
  }

  textarea {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-family: inherit;
    font-size: 1rem;
    resize: vertical;
    transition: border-color 0.2s;
  }

  textarea:focus {
    outline: none;
    border-color: var(--mdc-theme-primary);
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
