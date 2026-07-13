/// <reference types="svelte" />

// TypeScript declarations for Svelte 5 + Vite
declare module '*.svelte' {
  import type { Component } from 'svelte';
  const component: Component;
  export default component;
}
