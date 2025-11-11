import { writable } from 'svelte/store';

export type ThemeMode = 'light' | 'dark';

// Check for system preference or saved preference
function getInitialTheme(): ThemeMode {
  if (typeof window === 'undefined') return 'dark';
  
  const saved = localStorage.getItem('theme');
  if (saved === 'light' || saved === 'dark') {
    return saved;
  }
  
  // Check system preference
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
    return 'light';
  }
  
  return 'dark';
}

function createThemeStore() {
  const { subscribe, set, update } = writable<ThemeMode>(getInitialTheme());
  
  return {
    subscribe,
    toggle: () => update(mode => {
      const newMode = mode === 'light' ? 'dark' : 'light';
      if (typeof window !== 'undefined') {
        localStorage.setItem('theme', newMode);
        document.documentElement.setAttribute('data-theme', newMode);
      }
      return newMode;
    }),
    set: (mode: ThemeMode) => {
      if (typeof window !== 'undefined') {
        localStorage.setItem('theme', mode);
        document.documentElement.setAttribute('data-theme', mode);
      }
      set(mode);
    }
  };
}

export const theme = createThemeStore();
