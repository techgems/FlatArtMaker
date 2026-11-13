<script lang="ts">
  import { getSession } from '../../session';
  import { textOn } from '../utils/contrast';
  import { coverage } from '../utils/coverage';

  const session = getSession();

  const rows = $derived(
    session.previews
      ? coverage(session.previews.mapped, session.previews.vector.borders.faces)
      : [],
  );
  const base = $derived(
    session.settings?.baseHeight === undefined ? undefined : session.previews?.mapped.palette.base,
  );

  const css = ({ r, g, b }: { r: number; g: number; b: number }): string => `rgb(${r} ${g} ${b})`;
</script>

<section class="summary" aria-labelledby="colours-heading">
  <h2 id="colours-heading">Output colours</h2>
  <ul>
    {#each rows as row (row.colour.id)}
      <li>
        <span
          class="chip"
          style:background={css(row.colour.display)}
          style:color={textOn(row.colour.display)}
          title={row.slot === null ? 'Empty: not in the model, so it takes no slot' : undefined}
        >
          {row.slot ?? '!'}
        </span>
        <span class="name">
          {row.colour.name}
          {#if row.colour.id === base}<span class="tag">base</span>{/if}
        </span>
        <span class="share">{(row.share * 100).toFixed(1)}%</span>
      </li>
    {/each}
  </ul>
  <button type="button" disabled title="The colour editor is coming soon">Edit colours</button>
</section>

<style>
  .summary {
    display: grid;
    gap: 0.75rem;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
  }

  ul {
    display: grid;
    gap: 0.4rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 0.5rem;
  }

  .chip {
    display: grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
    font-size: 0.85rem;
    font-weight: 600;
    border: 1px solid var(--color-border);
    border-radius: 50%;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tag {
    margin-left: 0.3rem;
    padding: 0 0.3rem;
    font-size: 0.75rem;
    color: var(--color-muted);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }

  .share {
    font-size: 0.85rem;
    font-variant-numeric: tabular-nums;
    color: var(--color-muted);
  }

  button {
    padding: 0.5rem;
    font: inherit;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }
</style>
