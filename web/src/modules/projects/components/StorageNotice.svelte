<script lang="ts">
  import { getProjects } from '../context';
  import RecentProjects from './RecentProjects.svelte';

  const projects = getProjects();

  function show(dialog: HTMLDialogElement): void {
    dialog.showModal();
  }
</script>

{#if projects.notice === 'unavailable'}
  <p class="notice" role="status">
    This browser isn’t letting the app store anything (a private window, or blocked site data), so
    projects won’t be kept. Everything else works.
  </p>
{:else if projects.notice === 'full'}
  <p class="notice warning" role="alert">
    Browser storage is full, so your latest changes aren’t saved.
    <button type="button" onclick={() => (projects.managing = true)}>Delete old projects</button>
  </p>
{/if}
{#if projects.error}
  <p class="notice warning" role="alert">Saving projects failed: {projects.error}</p>
{/if}

{#if projects.managing}
  <dialog {@attach show} onclose={() => (projects.managing = false)} aria-labelledby="free-heading">
    <h2 id="free-heading">Free up space</h2>
    <p>Deleting a project frees its space, and your current work is then saved.</p>
    <RecentProjects />
    <form method="dialog"><button>Done</button></form>
  </dialog>
{/if}

<style>
  .notice {
    margin: 0;
    padding: 0.5rem 1.5rem;
    font-size: 0.9rem;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }

  .warning {
    color: var(--color-warning);
  }

  button {
    margin-left: 0.5rem;
    padding: 0.2rem 0.6rem;
    font: inherit;
    color: inherit;
    cursor: pointer;
    background: none;
    border: 1px solid currentColor;
    border-radius: 4px;
  }

  dialog {
    display: grid;
    gap: 0.75rem;
    width: min(32rem, calc(100vw - 2rem));
    color: var(--color-text);
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }

  dialog h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  dialog p {
    margin: 0;
  }

  dialog form {
    justify-self: end;
  }

  dialog form button {
    margin: 0;
    padding: 0.4rem 1rem;
  }
</style>
