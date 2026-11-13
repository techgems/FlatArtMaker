<script lang="ts">
  import { getProjects } from '../context';
  import { RecentList } from '../recent-list.svelte';

  interface Props {
    /** Opens a project; left out where projects can only be managed. */
    onopen?: (id: string) => void;
  }

  let { onopen }: Props = $props();

  const projects = getProjects();
  const list = new RecentList();
  const when = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });

  async function rename(id: string): Promise<void> {
    await projects.rename(id, list.draft);
    list.cancel();
  }

  async function remove(id: string): Promise<void> {
    await projects.remove(id);
    list.cancel();
  }
</script>

{#if projects.recent.length > 0}
  <section aria-labelledby="recent-heading">
    <h2 id="recent-heading">Recent projects</h2>
    <ul>
      {#each projects.recent as project (project.id)}
        <li>
          {#if list.renaming === project.id}
            <form
              class="rename"
              onsubmit={(event) => {
                event.preventDefault();
                void rename(project.id);
              }}
            >
              <input
                aria-label="Project name"
                bind:value={list.draft}
                onkeydown={(event) => event.key === 'Escape' && list.cancel()}
                {@attach (input: HTMLInputElement) => input.select()}
              />
              <button type="submit">Save</button>
              <button type="button" onclick={() => list.cancel()}>Cancel</button>
            </form>
          {:else}
            <div class="info">
              {#if onopen}
                <button type="button" class="open" onclick={() => onopen(project.id)}>
                  {project.name}
                </button>
              {:else}
                <span class="name">{project.name}</span>
              {/if}
              <span class="when">{when.format(project.updatedAt)}</span>
            </div>
            {#if list.confirming === project.id}
              <span class="confirm" role="alert">
                Delete “{project.name}”? This can’t be undone.
                <button type="button" class="danger" onclick={() => void remove(project.id)}>
                  Delete
                </button>
                <button type="button" onclick={() => list.cancel()}>Keep</button>
              </span>
            {:else}
              <span class="actions">
                <button type="button" onclick={() => list.startRename(project)}>Rename</button>
                <button type="button" onclick={() => list.askDelete(project.id)}>Delete</button>
              </span>
            {/if}
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  section {
    display: grid;
    gap: 0.5rem;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
  }

  ul {
    display: grid;
    gap: 0.25rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--color-surface);
    border-radius: var(--radius);
  }

  .info {
    display: grid;
    min-width: 0;
  }

  .open,
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .open {
    padding: 0;
    font: inherit;
    font-weight: 600;
    color: var(--color-accent);
    text-align: left;
    cursor: pointer;
    background: none;
    border: none;
  }

  .when {
    font-size: 0.8rem;
    color: var(--color-muted);
  }

  .actions,
  .confirm,
  .rename {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
  }

  button:not(.open) {
    padding: 0.25rem 0.55rem;
    font: inherit;
    color: inherit;
    cursor: pointer;
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }

  .danger {
    color: var(--color-danger);
    border-color: var(--color-danger);
  }

  input {
    padding: 0.25rem 0.4rem;
    font: inherit;
    color: inherit;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }
</style>
