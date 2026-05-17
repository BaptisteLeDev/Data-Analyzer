<script lang="ts">
  import type { RarestResponse } from "./api";

  type Props = { data: RarestResponse | null; loading: boolean; error: string | null };
  const { data, loading, error }: Props = $props();

  function sexLabel(s: 1 | 2): string {
    return s === 1 ? "M" : "F";
  }
</script>

<section class="results">
  <h2>Top 20 — Prénoms rares</h2>

  {#if loading}
    <p class="meta">Calcul en cours…</p>
  {:else if error}
    <p class="error">Erreur : {error}</p>
  {:else if !data}
    <p class="meta">Sélectionnez vos filtres puis lancez le calcul.</p>
  {:else if data.results.length === 0}
    <p class="meta">
      Aucun prénom contenant « {data.letter} » trouvé pour {data.dept} en {data.year}.
    </p>
  {:else}
    <p class="meta">
      Année {data.year} · Département {data.dept} · Lettre obligatoire « {data.letter} »
    </p>
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
</style>
