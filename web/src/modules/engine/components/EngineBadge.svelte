<script lang="ts">
  import { getEngineStatus } from '../context';

  const status = getEngineStatus();
</script>

<span class={['badge', status.phase]}>
  {#if status.phase === 'loading'}
    Engine loading…
  {:else if status.phase === 'ready'}
    Engine v{status.version}
  {:else}
    Engine failed: {status.error}
  {/if}
</span>

<style>
  .badge {
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  .failed {
    color: var(--color-danger);
  }
</style>
