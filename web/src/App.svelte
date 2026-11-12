<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    DebugExport,
    EngineBadge,
    EngineStatus,
    createWorkerEngineClient,
    setEngineClient,
    setEngineStatus,
  } from './modules/engine';

  const client = setEngineClient(createWorkerEngineClient());
  const status = setEngineStatus(new EngineStatus(client));
  void status.load();

  onDestroy(() => client.dispose());
</script>

<header>
  <h1>FlatArtMaker</h1>
  <EngineBadge />
</header>

{#if import.meta.env.DEV}
  <DebugExport />
{/if}

<style>
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    background: var(--color-surface);
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
  }
</style>
