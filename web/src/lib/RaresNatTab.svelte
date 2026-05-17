<script lang="ts">
  import { fetchRarestNat, type RarestNatResponse } from "./api";

  let year = $state(2006);
  let letter = $state("L");
  let sex = $state<0 | 1 | 2>(0);
  let search = $state("");
  let exclude = $state("");
  let limit = $state(20);

  let loading = $state(false);
  let error = $state<string | null>(null);
  let data = $state<RarestNatResponse | null>(null);

  const YEARS = Array.from({ length: 22 }, (_, i) => 2000 + i);

  async function load(useLimit: number) {
    loading = true;
    error = null;
    try {
      data = await fetchRarestNat({ year, letter, sex, search, exclude, limit: useLimit });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      data = null;
    } finally {
      loading = false;
    }
  }

  function run() {
    limit = 20;
    load(limit);
  }

  function loadMore() {
    limit += 20;
    load(limit);
  }

  function sexLabel(s: 1 | 2): string {
    return s === 1 ? "M" : "F";
  }

  function distribLabel(n: number): string {
    if (n === 0) return "diffus (0 dept)";
    if (n === 1) return "1 dept";
    return `${n} dépts`;
  }
</script>

<div class="layout">
  <aside class="filters">
    <h2>Filtres nationaux</h2>

    <div class="field">
      <label for="n-year">Année</label>
      <select id="n-year" bind:value={year}>
        {#each YEARS as y}<option value={y}>{y}</option>{/each}
      </select>
    </div>

    <div class="field">
      <label for="n-sex">Sexe</label>
      <select id="n-sex" bind:value={sex}>
        <option value={0}>Tous</option>
        <option value={1}>Masculin</option>
        <option value={2}>Féminin</option>
      </select>
    </div>

    <div class="field">
      <label for="n-letter">Lettre obligatoire <span class="hint">(vide = aucune)</span></label>
      <input id="n-letter" type="text" maxlength="1" bind:value={letter}
        oninput={(e) => letter = (e.currentTarget as HTMLInputElement).value.toUpperCase()} />
    </div>

    <div class="field">
      <label for="n-search">Recherche <span class="hint">(doit contenir)</span></label>
      <input id="n-search" type="text" placeholder="ex : LOU" bind:value={search}
        oninput={(e) => search = (e.currentTarget as HTMLInputElement).value.toUpperCase()} />
    </div>

    <div class="field">
      <label for="n-exclude">Exclusion <span class="hint">(ne doit pas contenir)</span></label>
      <input id="n-exclude" type="text" placeholder="ex : LL → exclut Elliot" bind:value={exclude}
        oninput={(e) => exclude = (e.currentTarget as HTMLInputElement).value.toUpperCase()} />
    </div>

    <button onclick={run} disabled={loading}>
      {loading ? "Calcul…" : "▶ Lancer"}
    </button>
  </aside>

  <section class="main">
    <h2>Prénoms rares — niveau national {data ? `(${data.results.length})` : ""}</h2>

    {#if loading && !data}
      <p class="meta">Calcul en cours…</p>
    {:else if error}
      <p class="error">Erreur : {error}</p>
    {:else if !data}
      <p class="meta">Sélectionne tes filtres puis lance.</p>
    {:else if data.results.length === 0}
      <p class="meta">Aucun prénom ne correspond.</p>
    {:else}
      <p class="meta">
        Source : fichier national INSEE — agrégation France entière.
        La colonne « Diffusion » indique dans combien de départements le prénom apparaît dans le fichier départemental
        (0 = totalement diffus, jamais ≥ 3 dans aucun département individuel).
      </p>
      <table>
        <thead>
          <tr>
            <th class="rank">#</th>
            <th>Prénom</th>
            <th class="sex">Sexe</th>
            <th class="num">N national</th>
            <th class="num">Diffusion</th>
          </tr>
        </thead>
        <tbody>
          {#each data.results as r}
            <tr>
              <td class="rank">{r.rank}</td>
              <td class="name">{r.prenom}</td>
              <td class="sex">{sexLabel(r.sexe)}</td>
              <td class="num">{r.n}</td>
              <td class="num diffusion">{distribLabel(r.dept_count)}</td>
            </tr>
          {/each}
        </tbody>
      </table>

      {#if data.has_more}
        <div class="actions">
          <button class="more" onclick={loadMore} disabled={loading}>
            {loading ? "Chargement…" : "Charger 20 de plus"}
          </button>
          <span class="meta inline">Affichés : {data.results.length}</span>
        </div>
      {:else if data.results.length >= 20}
        <p class="meta done">— Fin de la liste —</p>
      {/if}

      {#if data.censored_count > 0}
        <aside class="censored">
          <span class="badge">_PRENOMS_RARES</span>
          <strong>{data.censored_count.toLocaleString("fr-FR")}</strong> naissance{data.censored_count > 1 ? "s" : ""}
          dont le prénom apparaît 1 ou 2 fois au niveau national — censurées par l'INSEE même à cette granularité.
        </aside>
      {/if}
    {/if}
  </section>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 320px 1fr;
    min-height: calc(100vh - 130px);
  }
  .filters {
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    padding: calc(var(--space) * 3);
    display: flex;
    flex-direction: column;
    gap: calc(var(--space) * 2);
  }
  .main { padding: calc(var(--space) * 3); }
  h2 {
    margin: 0 0 var(--space);
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    color: var(--text-mute);
    text-transform: uppercase;
  }
  .field { display: flex; flex-direction: column; }
  .hint {
    text-transform: none;
    font-weight: 400;
    color: var(--text-mute);
    font-size: 0.75rem;
  }
  .meta { font-size: 0.9rem; color: var(--text-mute); margin: 0 0 calc(var(--space) * 2); }
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
  .diffusion { color: var(--text-mute); font-style: italic; }
  tbody tr:hover { background: var(--bg-panel); }
  .actions {
    margin-top: calc(var(--space) * 2);
    display: flex; align-items: center; gap: calc(var(--space) * 2);
  }
  .more {
    background: white; color: var(--bleu-rep);
    border: 1px solid var(--bleu-rep);
    padding: var(--space) calc(var(--space) * 2);
    font-weight: 600; cursor: pointer;
  }
  .more:hover:not(:disabled) { background: var(--bleu-rep); color: white; }
  .censored {
    margin-top: calc(var(--space) * 3);
    padding: calc(var(--space) * 1.5) calc(var(--space) * 2);
    background: #FFF6F6;
    border-left: 3px solid var(--rouge-rep);
    font-size: 0.85rem;
    line-height: 1.6;
  }
  .badge {
    display: inline-block;
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    background: var(--rouge-rep);
    color: white;
    padding: 2px 6px;
    margin-right: 6px;
    letter-spacing: 0.04em;
  }
</style>
