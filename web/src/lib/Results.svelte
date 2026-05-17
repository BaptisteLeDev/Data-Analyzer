<script lang="ts">
  import type { RarestResponse } from "./api";

  type Props = {
    data: RarestResponse | null;
    loading: boolean;
    error: string | null;
    onLoadMore: () => void;
  };
  const { data, loading, error, onLoadMore }: Props = $props();

  function sexLabel(s: 1 | 2): string {
    return s === 1 ? "M" : "F";
  }

  function describeFilters(d: RarestResponse): string {
    const parts: string[] = [`Année ${d.year}`];
    parts.push(d.dept ? `Département ${d.dept}` : "France entière");
    if (d.sex === 1) parts.push("hommes");
    else if (d.sex === 2) parts.push("femmes");
    if (d.letter) parts.push(`contient « ${d.letter} »`);
    if (d.search) parts.push(`avec « ${d.search} »`);
    if (d.exclude) parts.push(`sans « ${d.exclude} »`);
    return parts.join(" · ");
  }
</script>

<section class="results">
  <h2>Prénoms rares {data ? `(${data.results.length})` : ""}</h2>

  {#if loading && !data}
    <p class="meta">Calcul en cours…</p>
  {:else if error}
    <p class="error">Erreur : {error}</p>
  {:else if !data}
    <p class="meta">Sélectionnez vos filtres puis lancez le calcul.</p>
  {:else if data.results.length === 0}
    <p class="meta">Aucun prénom ne correspond à ces filtres.</p>
  {:else}
    <p class="meta">{describeFilters(data)}</p>
    <table>
      <thead>
        <tr>
          <th class="rank">#</th>
          <th>Prénom</th>
          <th class="sex">Sexe</th>
          <th class="num">Naissances</th>
        </tr>
      </thead>
      <tbody>
        {#each data.results as r}
          <tr>
            <td class="rank">{r.rank}</td>
            <td class="name">{r.prenom}</td>
            <td class="sex">{sexLabel(r.sexe)}</td>
            <td class="num">{r.n}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if data.has_more}
      <div class="actions">
        <button class="more" onclick={onLoadMore} disabled={loading}>
          {loading ? "Chargement…" : "Charger 20 de plus"}
        </button>
        <span class="meta inline">Affichés : {data.results.length}</span>
      </div>
    {:else if data.results.length >= 20}
      <p class="meta done">— Fin de la liste —</p>
    {/if}
  {/if}
</section>

<style>
  .results { padding: calc(var(--space) * 3); }
  h2 {
    margin: 0 0 var(--space);
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    color: var(--text-mute);
    text-transform: uppercase;
  }
  .meta {
    font-size: 0.9rem;
    color: var(--text-mute);
    margin: 0 0 calc(var(--space) * 2);
  }
  .meta.inline { margin: 0; }
  .meta.done { text-align: center; margin-top: calc(var(--space) * 2); }
  .error { color: var(--rouge-rep); }
  table { width: 100%; border-collapse: collapse; }
  th, td {
    text-align: left;
    padding: var(--space) calc(var(--space) * 1.5);
    border-bottom: 1px solid var(--border);
    font-size: 0.95rem;
  }
  th {
    font-weight: 600;
    color: var(--text-mute);
    background: var(--bg-panel);
    border-bottom: 2px solid var(--border);
  }
  .rank, .sex, .num { width: 1%; white-space: nowrap; }
  .num, .rank { text-align: right; font-variant-numeric: tabular-nums; }
  .name { font-weight: 600; color: var(--bleu-rep); }
  tbody tr:hover { background: var(--bg-panel); }
  .actions {
    margin-top: calc(var(--space) * 2);
    display: flex;
    align-items: center;
    gap: calc(var(--space) * 2);
  }
  .more {
    background: white;
    color: var(--bleu-rep);
    border: 1px solid var(--bleu-rep);
    padding: var(--space) calc(var(--space) * 2);
    font-weight: 600;
    cursor: pointer;
  }
  .more:hover:not(:disabled) { background: var(--bleu-rep); color: white; }
</style>
