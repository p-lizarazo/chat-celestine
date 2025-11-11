<script lang="ts">
  import { sendChatMessage, type ChatRequest, type Message, MessageRole, type ChatResponse } from './api';

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
    <h2>Chat</h2>
    {#if messages.length > 0}
      <button class="clear-btn" on:click={clearChat}>Clear</button>
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
        <div class="message {message.role}">
          <div class="message-role">{message.role}</div>
          <div class="message-content">{message.content}</div>
        </div>
      {/each}
    {/if}

    {#if isLoading}
      <div class="message assistant loading">
        <div class="message-role">assistant</div>
        <div class="message-content">Thinking...</div>
      </div>
    {/if}

    {#if error}
      <div class="error-message">
        <strong>Error:</strong> {error}
      </div>
    {/if}
  </div>

  <div class="input-container">
    <textarea
      bind:value={userInput}
      on:keydown={handleKeyDown}
      placeholder="Type your message... (Shift+Enter for new line)"
      rows="3"
      disabled={isLoading || !providerName || !selectedModel}
    ></textarea>
    <button
      on:click={handleSend}
      disabled={!userInput.trim() || isLoading || !providerName || !selectedModel}
    >
      Send
    </button>
  </div>
</div>

<style>
  .chat-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-width: 900px;
    margin: 0 auto;
  }

  .chat-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #333;
  }

  .chat-header h2 {
    margin: 0;
    font-size: 1.5rem;
  }

  .clear-btn {
    padding: 0.5rem 1rem;
    background: #444;
    border: none;
    border-radius: 4px;
    color: white;
    cursor: pointer;
  }

  .clear-btn:hover {
    background: #555;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty-state {
    text-align: center;
    color: #666;
    margin-top: 4rem;
  }

  .empty-state .hint {
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }

  .message {
    padding: 1rem;
    border-radius: 8px;
    max-width: 80%;
  }

  .message.user {
    align-self: flex-end;
    background: #2563eb;
    margin-left: auto;
  }

  .message.assistant {
    align-self: flex-start;
    background: #374151;
  }

  .message.system {
    align-self: center;
    background: #1f2937;
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

  .loading {
    opacity: 0.7;
  }

  .error-message {
    padding: 1rem;
    background: #dc2626;
    border-radius: 8px;
    color: white;
  }

  .input-container {
    padding: 1rem;
    border-top: 1px solid #333;
    display: flex;
    gap: 0.5rem;
  }

  textarea {
    flex: 1;
    padding: 0.75rem;
    background: #1f2937;
    border: 1px solid #374151;
    border-radius: 4px;
    color: white;
    font-family: inherit;
    font-size: 1rem;
    resize: vertical;
  }

  textarea:focus {
    outline: none;
    border-color: #2563eb;
  }

  textarea:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button {
    padding: 0.75rem 1.5rem;
    background: #2563eb;
    border: none;
    border-radius: 4px;
    color: white;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  button:hover:not(:disabled) {
    background: #1d4ed8;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
