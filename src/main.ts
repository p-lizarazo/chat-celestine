import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';
import { theme } from './lib/theme';

// Initialize theme on load
theme.subscribe(mode => {
  document.documentElement.setAttribute('data-theme', mode);
});

mount(App, {
  target: document.getElementById('app')!,
});
