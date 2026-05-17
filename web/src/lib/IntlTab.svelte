<script lang="ts">
  import {
    fetchIntl,
    fetchIntlMatch,
    type IntlResponse,
    type IntlMatchResponse,
    type IntlMatchAlgo
  } from "./api";

  // Two modes: legacy "inverse" search (US-only, no algo merging) and the
  // new "match" (3 algos cross-referenced with INSEE).
  let mode = $state<"match" | "inverse">("match");

  // ---- shared filters ----
  let letter = $state("L");
  let sex = $state<0 | 1 | 2>(0);
  let search = $state("");
  let exclude = $state("");
  let era_start = $state(1985);
  let era_end = $state(2005);

  // ---- inverse-mode-only ----
  let absent_fr = $state("any");
  let double_variant = $state(true);

  // ---- match-mode-only ----
  let n_min = $state(5);
  let n_max = $state(100);
  let one_l = $state(true);
  let lev_max = $state(2);
  let intl_seed_limit = $state(800);
  let use_phonetic = $state(true);
  let use_lev2 = $state(true);
  let use_anglicisation = $state(true);

  let limit = $state(50);

  let loading = $state(false);
  let error = $state<string | null>(null);
  let dataInv = $state<IntlResponse | null>(null);
  let dataMatch = $state<IntlMatchResponse | null>(null);

  async function load(useLimit: number) {
    loading = true;
    error = null;
    try {
      if (mode === "inverse") {
        dataInv = await fetchIntl({
          letter,
          sex,
          search,
          exclude: exclude || "LL",
          era_start,
          era_end,
          absent_fr,
          double_variant,
          limit: useLimit
        });
        dataMatch = null;
      } else {
        const algos: IntlMatchAlgo[] = [];
        if (use_phonetic) algos.push("phonetic");
        if (use_lev2) algos.push("lev2");
        if (use_anglicisation) algos.push("anglicisation");
        dataMatch = await fetchIntlMatch({
          letter,
          sex,
          search,
          exclude,
          era_start,
          era_end,
          n_min,
          n_max,
          one_l,
          lev_max,
          intl_seed_limit,
          algos,
          limit: useLimit
        });
        dataInv = null;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      dataInv = null;
      dataMatch = null;
    } finally {
      loading = false;
    }
  }

  function run() {
    limit = mode === "inverse" ? 30 : 50;
    load(limit);
  }

  function loadMore() {
    limit += mode === "inverse" ? 30 : 50;
    load(limit);
  }

  function sexLabel(s: 1 | 2): string {
    return s === 1 ? "M" : "F";
  }

  function algoBadge(a: IntlMatchAlgo): string {
    return a === "phonetic" ? "PHON" : a === "lev2" ? "LEV" : "ANGLO";
  }
</script>

<div class="layout">
  <aside class="filters">
    <h2>Recherche internationale</h2>

    <div class="modeswitch">
      <button class:active={mode === "match"} onclick={() => (mode = "match")}>
        Multi-algo (FR ↔ anglo)
      </button>
      <button class:active={mode === "inverse"} onclick={() => (mode = "inverse")}>
        Inverse (US absents FR)
      </button>
    </div>

    {#if mode === "match"}
      <p class="explain">
        Trois algos parallèles : <strong>phonétique</strong> (DoubleMetaphone),
        <strong>Levenshtein ≤ 2</strong>, et <strong>règles d'anglicisation</strong>
        (PH↔F, K↔C, Y↔I, -EY↔-IE, …). Les noms touchés par plusieurs algos
        remontent en haut. Cible : <em>prénom français rare</em> dérivé d'un
        anglo qui n'est pas dans INSEE.
      </p>
    {:else}
      <p class="explain">
        Algo « inverse » : on cherche dans le dataset US SSA (1880-2017)
        ceux qui sont <strong>absents</strong> du fichier INSEE FR — donc des imports.
      </p>
    {/if}

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
      <label for="i-search">Recherche <span class="hint">(sous-chaîne obligatoire)</span></label>
      <input
        id="i-search"
        type="text"
        placeholder="ex : EL"
        bind:value={search}
        oninput={(e) => (search = (e.currentTarget as HTMLInputElement).value.toUpperCase())}
      />
    </div>

    <div class="field">
      <label for="i-exclude">Exclusion <span class="hint">(sous-chaîne interdite)</span></label>
      <input
        id="i-exclude"
        type="text"
        placeholder="ex : LL"
        bind:value={exclude}
        oninput={(e) => (exclude = (e.currentTarget as HTMLInputElement).value.toUpperCase())}
      />
    </div>

    <fieldset class="range">
      <legend>Époque US (popularité)</legend>
      <input type="number" min="1880" max="2017" bind:value={era_start} />
      <span>→</span>
      <input type="number" min="1880" max="2017" bind:value={era_end} />
    </fieldset>

    {#if mode === "inverse"}
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
    {:else}
      <fieldset class="range">
        <legend>Total INSEE (toutes années)</legend>
        <input type="number" min="0" max="100000" bind:value={n_min} />
        <span>→</span>
        <input type="number" min="0" max="100000" bind:value={n_max} />
      </fieldset>

      <label class="check">
        <input type="checkbox" bind:checked={one_l} />
        <span>Exactement un seul 'L'</span>
      </label>

      <fieldset class="algos">
        <legend>Algos actifs</legend>
        <label><input type="checkbox" bind:checked={use_phonetic} /> Phonétique (DoubleMetaphone)</label>
        <label><input type="checkbox" bind:checked={use_lev2} /> Levenshtein ≤ {lev_max}</label>
        <label><input type="checkbox" bind:checked={use_anglicisation} /> Anglicisation (règles)</label>
      </fieldset>

      <div class="field">
        <label for="i-lev">Lev max <span class="hint">(1–3)</span></label>
        <input id="i-lev" type="number" min="1" max="3" bind:value={lev_max} />
      </div>

      <div class="field">
        <label for="i-seed">Seeds intl <span class="hint">(plus = + lent)</span></label>
        <input id="i-seed" type="number" min="50" max="5000" step="100" bind:value={intl_seed_limit} />
      </div>
    {/if}

    <button onclick={run} disabled={loading}>
      {loading ? "Calcul…" : "▶ Lancer"}
    </button>
  </aside>

  <section class="main">
    <h2>
      Résultats
      {#if mode === "inverse" && dataInv}({dataInv.results.length}){/if}
      {#if mode === "match" && dataMatch}({dataMatch.results.length}){/if}
    </h2>

    {#if loading && !dataInv && !dataMatch}
      <p class="meta">Calcul en cours…</p>
    {:else if error}
      <p class="error">Erreur : {error}</p>
    {:else if mode === "inverse" && dataInv}
      {#if dataInv.results.length === 0}
        <p class="meta">Aucun prénom ne correspond — relâche un critère.</p>
      {:else}
        <p class="meta">
          Tri par rareté US totale croissante. La colonne « Variant » affiche un nom dérivé à lettre
          doublée si le filtre est actif.
        </p>
        <table>
          <thead>
            <tr>
              <th class="rank">#</th>
              <th>Prénom</th>
              <th class="sex-col">Sexe</th>
              <th class="num">US</th>
              <th class="num">UK</th>
              <th class="sources-col">Source</th>
              <th class="num">Période</th>
              <th class="num">Pop. {dataInv.era_start}–{dataInv.era_end}</th>
              <th>Variant ↑↑</th>
            </tr>
          </thead>
          <tbody>
            {#each dataInv.results as r}
              <tr>
                <td class="rank">{r.rank}</td>
                <td class="name">{r.prenom}</td>
                <td class="sex-col">{sexLabel(r.sex)}</td>
                <td class="num">{r.total_us > 0 ? r.total_us.toLocaleString("en-US") : "—"}</td>
                <td class="num">{r.total_uk > 0 ? r.total_uk.toLocaleString("en-US") : "—"}</td>
                <td class="sources-col">
                  {#each r.sources as src}
                    <span class="badge badge-{src.toLowerCase()}">{src}</span>
                  {/each}
                </td>
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

        {#if dataInv.has_more}
          <div class="actions">
            <button class="more" onclick={loadMore} disabled={loading}>
              {loading ? "Chargement…" : "Charger 30 de plus"}
            </button>
            <span class="meta inline">Affichés : {dataInv.results.length}</span>
          </div>
        {/if}
      {/if}
    {:else if mode === "match" && dataMatch}
      {#if dataMatch.results.length === 0}
        <p class="meta">
          Aucun prénom français rare ne correspond aux algos sélectionnés. Élargis
          la fenêtre (Lev max ↑, n_min/n_max plus larges, seeds ↑).
        </p>
      {:else}
        <p class="meta">
          Tri par <strong>score</strong> décroissant (consensus inter-algos), puis rareté INSEE.
          Algos actifs : {dataMatch.algos.join(", ")}. Bande n: [{dataMatch.n_min}–{dataMatch.n_max}].
        </p>
        <table>
          <thead>
            <tr>
              <th class="rank">#</th>
              <th>Prénom FR</th>
              <th class="num">Score</th>
              <th>Détectée par</th>
              <th class="num">Total INSEE</th>
              <th class="num">Années</th>
              <th class="num">Lev</th>
              <th>Anglo source(s)</th>
              <th>Règles</th>
            </tr>
          </thead>
          <tbody>
            {#each dataMatch.results as r}
              <tr>
                <td class="rank">{r.rank}</td>
                <td class="name">{r.prenom}</td>
                <td class="num score">{r.score.toFixed(2)}</td>
                <td class="badges">
                  {#each r.matched_by as a}
                    <span class="badge badge-{a}">{algoBadge(a)}</span>
                  {/each}
                </td>
                <td class="num">{r.insee_total.toLocaleString("fr-FR")}</td>
                <td class="num">{r.insee_years}</td>
                <td class="num">{r.lev_distance ?? "—"}</td>
                <td class="from">
                  {#each r.from_intl as fi, i}
                    <span class="anglo">{fi}</span>{#if i < r.from_intl.length - 1}, {/if}
                  {/each}
                </td>
                <td class="rules">
                  {#each r.anglo_rules as ru, i}
                    <span class="rule">{ru}</span>{#if i < r.anglo_rules.length - 1}{" "}{/if}
                  {/each}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>

        {#if dataMatch.has_more}
          <div class="actions">
            <button class="more" onclick={loadMore} disabled={loading}>
              {loading ? "Chargement…" : "Charger 50 de plus"}
            </button>
            <span class="meta inline">Affichés : {dataMatch.results.length}</span>
          </div>
        {/if}
      {/if}
    {:else}
      <p class="meta">Renseigne tes filtres puis lance la recherche.</p>
    {/if}
  </section>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 340px 1fr;
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
  .modeswitch {
    display: flex;
    gap: 4px;
    background: white;
    padding: 4px;
    border: 1px solid var(--border);
  }
  .modeswitch button {
    flex: 1;
    background: transparent;
    color: var(--text-mute);
    border: none;
    padding: 6px 4px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .modeswitch button.active {
    background: var(--bleu-rep);
    color: white;
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
  .range,
  .algos {
    border: 1px solid var(--border);
    padding: var(--space);
    background: white;
  }
  .range {
    display: flex;
    align-items: center;
    gap: var(--space);
  }
  .algos {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .algos label {
    font-size: 0.85rem;
    display: flex;
    gap: var(--space);
    align-items: center;
  }
  .range legend,
  .algos legend {
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
  .score {
    font-weight: 700;
    color: var(--bleu-rep);
  }
  .badges {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .badge {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 2px 6px;
    letter-spacing: 0.04em;
    color: white;
    border-radius: 2px;
  }
  .badge-phonetic { background: #4f46e5; }
  .badge-lev2 { background: #059669; }
  .badge-anglicisation { background: #d97706; }
  .badge-us { background: #002868; }
  .badge-uk { background: #c8102e; }
  .sources-col { white-space: nowrap; }
  .anglo {
    font-family: ui-monospace, monospace;
    font-size: 0.8rem;
    color: var(--text-mute);
  }
  .rule {
    display: inline-block;
    font-family: ui-monospace, monospace;
    font-size: 0.7rem;
    padding: 1px 4px;
    background: #f3f4f6;
    border: 1px solid var(--border);
    margin-right: 3px;
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
</style>
