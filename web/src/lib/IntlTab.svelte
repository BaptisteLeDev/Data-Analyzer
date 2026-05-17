<script lang="ts">
  import { fetchIntl, type IntlResponse } from "./api";

  let letter = $state("L");
  let sex = $state<0 | 1 | 2>(0);
  let search = $state("");
  let exclude = $state("LL");
  let era_start = $state(1985);
  let era_end = $state(2005);
  let absent_fr = $state("any");
  let double_variant = $state(true);
  let limit = $state(30);

  let loading = $state(false);
  let error = $state<string | null>(null);
  let data = $state<IntlResponse | null>(null);

  async function load(useLimit: number) {
    loading = true;
    error = null;
    try {
      data = await fetchIntl({
        letter,
        sex,
        search,
        exclude,
        era_start,
        era_end,
        absent_fr,
        double_variant,
        limit: useLimit
      });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      data = null;
    } finally {
      loading = false;
    }
  }

  function run() {
    limit = 30;
    load(limit);
  }

  function loadMore() {
    limit += 30;
    load(limit);
  }

  function sexLabel(s: 1 | 2): string {
    return s === 1 ? "M" : "F";
  }
</script>

<div class="layout">
  <aside class="filters">
    <h2>Recherche internationale</h2>

    <p class="explain">
      Algo « inverse » : on cherche dans le dataset US SSA (1880-2017, ~100k prénoms)
      ceux qui sont <strong>absents</strong> du fichier INSEE FR — donc des imports.
      Filtres adaptés à tes indices (lettre simple, variant double dans le corpus, série années 90).
    </p>

    <div class="field">
      <label for="i-letter">Lettre obligatoire <span class="hint">(vide = aucune)</span></label>
      <input
        id="i-letter"
        type="text"
        maxlength="1"
        bind:value={letter}
        oninput={(e) => (letter = (e.currentTarget as HTMLInputElement).value.toUpperCase())}
      />
    </div>

    <div class="field">
      <label for="i-sex">Sexe</label>
      <select id="i-sex" bind:value={sex}>
        <option value={0}>Tous</option>
        <option value={1}>Masculin</option>
        <option value={2}>Féminin</option>
      </select>
    </div>

    <div class="field">
      <label for="i-exclude">Exclusion <span class="hint">(ex : LL — pas de double L)</span></label>
      <input
        id="i-exclude"
        type="text"
        placeholder="ex : LL"
        bind:value={exclude}
        oninput={(e) => (exclude = (e.currentTarget as HTMLInputElement).value.toUpperCase())}
      />
    </div>

    <div class="field">
      <label for="i-search">Recherche <span class="hint">(sous-chaîne obligatoire)</span></label>
      <input
        id="i-search"
        type="text"
        placeholder="ex : EL"
        bind:value={search}
        oninput={(e) => (search = (e.currentTarget as HTMLInputElement).value.toUpperCase())}
      />
    </div>

    <fieldset class="range">
      <legend>Époque US (popularité)</legend>
      <input type="number" min="1880" max="2017" bind:value={era_start} />
      <span>→</span>
      <input type="number" min="1880" max="2017" bind:value={era_end} />
    </fieldset>

    <div class="field">
      <label for="i-absent">Critère "absent FR"</label>
      <select id="i-absent" bind:value={absent_fr}>
        <option value="any">Jamais en France (toutes années)</option>
        <option value="year:2006">Absent uniquement en 2006</option>
      </select>
    </div>

    <label class="check">
      <input type="checkbox" bind:checked={double_variant} />
      <span>Garder uniquement les prénoms ayant un variant à lettre doublée</span>
    </label>

    <button onclick={run} disabled={loading}>
      {loading ? "Calcul…" : "▶ Lancer"}
    </button>
  </aside>

  <section class="main">
    <h2>Résultats {data ? `(${data.results.length})` : ""}</h2>

    {#if loading && !data}
      <p class="meta">Calcul en cours…</p>
    {:else if error}
      <p class="error">Erreur : {error}</p>
    {:else if !data}
      <p class="meta">Renseigne tes filtres puis lance la recherche.</p>
    {:else if data.results.length === 0}
      <p class="meta">Aucun prénom ne correspond — relâche un critère.</p>
    {:else}
      <p class="meta">
        Tri par rareté US totale croissante. La colonne « Variant » affiche un nom dérivé à lettre
        doublée si le filtre est actif (ex : ELIOT → ELLIOT existe aussi dans le corpus).
      </p>
      <table>
        <thead>
          <tr>
            <th class="rank">#</th>
            <th>Prénom</th>
            <th class="sex-col">Sexe</th>
            <th class="num">Total US</th>
            <th class="num">Période</th>
            <th class="num">Pop. {data.era_start}–{data.era_end}</th>
            <th>Variant ↑↑</th>
          </tr>
        </thead>
        <tbody>
          {#each data.results as r}
            <tr>
              <td class="rank">{r.rank}</td>
              <td class="name">{r.prenom}</td>
              <td class="sex-col">{sexLabel(r.sex)}</td>
              <td class="num">{r.total_us.toLocaleString("en-US")}</td>
              <td class="num period">
                {r.first_year}{r.first_year !== r.last_year ? `–${r.last_year}` : ""}
              </td>
              <td class="num">{r.era_count.toLocaleString("en-US")}</td>
              <td class="variant">
                {#if r.variant_example}
                  → <strong>{r.variant_example}</strong>
                {:else if r.has_double_variant}
                  ✓
                {:else}
                  —
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>

      {#if data.has_more}
        <div class="actions">
          <button class="more" onclick={loadMore} disabled={loading}>
            {loading ? "Chargement…" : "Charger 30 de plus"}
          </button>
          <span class="meta inline">Affichés : {data.results.length}</span>
        </div>
      {/if}

      <aside class="hint-box">
        <strong>Astuce</strong> : si tu as un indice supplémentaire (ex : pays d'origine de la
        série, nom du personnage approximatif, époque), réutilise les champs « Recherche » et
        « Exclusion » pour resserrer. Tu peux aussi régler la fenêtre de popularité US pour caler
        exactement sur ta date de naissance (parents qui regardaient la série juste avant).
      </aside>
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
  .explain {
    font-size: 0.8rem;
    color: var(--text-mute);
    line-height: 1.5;
    margin: 0;
    padding: var(--space);
    background: white;
    border-left: 3px solid var(--bleu-rep);
  }
  .main {
    padding: calc(var(--space) * 3);
  }
  h2 {
    margin: 0 0 var(--space);
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    color: var(--text-mute);
    text-transform: uppercase;
  }
  .field {
    display: flex;
    flex-direction: column;
  }
  .hint {
    text-transform: none;
    font-weight: 400;
    color: var(--text-mute);
    font-size: 0.75rem;
  }
  .meta {
    font-size: 0.9rem;
    color: var(--text-mute);
    margin: 0 0 calc(var(--space) * 2);
  }
  .meta.inline {
    margin: 0;
  }
  .error {
    color: var(--rouge-rep);
  }
  .range {
    border: 1px solid var(--border);
    padding: var(--space);
    display: flex;
    align-items: center;
    gap: var(--space);
    background: white;
  }
  .range legend {
    font-size: 0.85rem;
    color: var(--text-mute);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0 var(--space);
  }
  .range input {
    width: 70px;
    padding: 4px 6px;
    border: 1px solid var(--border);
  }
  .check {
    display: flex;
    align-items: center;
    gap: var(--space);
    font-size: 0.85rem;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
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
  .rank,
  .sex-col,
  .num {
    width: 1%;
    white-space: nowrap;
  }
  .num,
  .rank {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .name {
    font-weight: 600;
    color: var(--bleu-rep);
  }
  .period {
    color: var(--text-mute);
  }
  .variant {
    color: var(--rouge-rep);
    font-size: 0.9rem;
  }
  tbody tr:hover {
    background: var(--bg-panel);
  }
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
  .more:hover:not(:disabled) {
    background: var(--bleu-rep);
    color: white;
  }
  .hint-box {
    margin-top: calc(var(--space) * 3);
    padding: calc(var(--space) * 1.5) calc(var(--space) * 2);
    background: #f0f4ff;
    border-left: 3px solid var(--bleu-rep);
    font-size: 0.85rem;
    line-height: 1.5;
  }
</style>
