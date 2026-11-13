<script lang="ts">
  import { onDestroy } from 'svelte';
  import { ColourSummary } from './modules/colours';
  import { ControlsPanel } from './modules/controls';
  import { DownloadBar } from './modules/downloads';
  import {
    EngineBadge,
    EngineStatus,
    createWorkerEngineClient,
    setEngineClient,
    setEngineStatus,
  } from './modules/engine';
  import { PreviewPanel } from './modules/preview-2d';
  import { Session, setSession } from './modules/session';
  import { UploadScreen } from './modules/upload';

  const client = setEngineClient(createWorkerEngineClient());
  const status = setEngineStatus(new EngineStatus(client));
  const session = setSession(new Session(client));
  void status.load();

  onDestroy(() => {
    session.close();
    client.dispose();
  });
</script>

<div class="app">
  <header>
    <h1>FlatArtMaker</h1>
    {#if session.phase === 'ready'}
      <span class="file">{session.name}</span>
      <button type="button" class="new" onclick={() => session.close()}>New image</button>
      <div class="downloads"><DownloadBar /></div>
    {/if}
    <EngineBadge />
  </header>

  {#if session.phase === 'ready'}
    <main class="workspace">
      <aside class="left" aria-label="Settings"><ControlsPanel /></aside>
      <section class="centre" aria-label="Previews"><PreviewPanel /></section>
      <aside class="right" aria-label="Output colours"><ColourSummary /></aside>
    </main>
  {:else if session.phase === 'loading'}
    <p class="loading" aria-live="polite">Reading {session.name}…</p>
  {:else}
    <UploadScreen onfile={(file: File) => void session.open(file)} error={session.error} />
  {/if}
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr;
    height: 100vh;
  }

  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.75rem 1rem;
    padding: 0.75rem 1.5rem;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
  }

  .file {
    color: var(--color-muted);
  }

  .new {
    padding: 0.3rem 0.6rem;
    font: inherit;
    font-size: 0.85rem;
    color: inherit;
    cursor: pointer;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  .downloads {
    margin-left: auto;
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(16rem, 20rem) 1fr minmax(12rem, 16rem);
    min-height: 0;
  }

  .left,
  .right {
    padding: 1rem;
    overflow-y: auto;
    background: var(--color-surface);
  }

  .left {
    border-right: 1px solid var(--color-border);
  }

  .right {
    border-left: 1px solid var(--color-border);
  }

  .centre {
    min-width: 0;
    min-height: 0;
    padding: 1rem;
  }

  .loading {
    margin: 3rem auto;
    color: var(--color-muted);
  }

  @media (max-width: 60rem) {
    .app {
      height: auto;
      min-height: 100vh;
    }

    .workspace {
      grid-template-columns: 1fr;
    }

    .centre {
      height: 70vh;
    }

    .left,
    .right {
      border: none;
    }
  }
</style>
