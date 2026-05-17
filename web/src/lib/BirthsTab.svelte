<script lang="ts">
  import { DEPARTEMENTS } from "./departements";
  import { fetchBirths, type BirthsResponse, type BirthRow } from "./api";

  let month = $state(5);
  let dept = $state("76");
  let sex = $state<0 | 1 | 2>(0);
  let age_mere_min = $state(15);
  let age_mere_max = $state(50);
  let age_pere_min = $state(15);
  let age_pere_max = $state(60);

  let loading = $state(false);
  let error = $state<string | null>(null);
  let data = $state<BirthsResponse | null>(null);

  const MONTHS = [
    [0, "Tous mois"], [1, "Janvier"], [2, "Février"], [3, "Mars"], [4, "Avril"],
    [5, "Mai"], [6, "Juin"], [7, "Juillet"], [8, "Août"],
    [9, "Septembre"], [10, "Octobre"], [11, "Novembre"], [12, "Décembre"]
  ] as const;

  const MONTH_NAMES = ["", "janvier", "février", "mars", "avril", "mai", "juin",
    "juillet", "août", "septembre", "octobre", "novembre", "décembre"];

  async function load(offset = 0) {
    loading = true;
    error = null;
    try {
      const r = await fetchBirths({
        month, dept, sex,
        age_mere_min, age_mere_max,
        age_pere_min, age_pere_max,
        limit: 50, offset
      });
      if (offset === 0) {
        data = r;
      } else if (data) {
        data = { ...r, results: [...data.results, ...r.results] };
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function loadMore() {
    if (data) load(data.offset + data.results.length);
  }

  function describe(r: BirthRow): string {
    const sx = r.sexe === 1 ? "Garçon" : "Fille";
    const dept = r.dept_nais ?? "?";
    const m = r.mois > 0 ? MONTH_NAMES[r.mois] : "?";
    const parts: string[] = [`${sx} né${r.sexe === 2 ? "e" : ""} en ${m} 2006 dans le ${dept}`];

    const motherBits: string[] = [];
    if (r.age_mere) motherBits.push(`${r.age_mere} ans`);
    if (r.nat_mere === 1) motherBits.push("FR");
    else if (r.nat_mere === 2) motherBits.push("étrangère");
    if (r.situ_mere === "S") motherBits.push("salariée");
    else if (r.situ_mere === "NS") motherBits.push("non salariée");
    if (motherBits.length) parts.push(`Mère ${motherBits.join(", ")}`);

    const fatherBits: string[] = [];
    if (r.age_pere) fatherBits.push(`${r.age_pere} ans`);
    if (r.nat_pere === 1) fatherBits.push("FR");
    else if (r.nat_pere === 2) fatherBits.push("étranger");
    if (r.situ_pere === "S") fatherBits.push("salarié");
    else if (r.situ_pere === "NS") fatherBits.push("non salarié");
    if (fatherBits.length) parts.push(`Père ${fatherBits.join(", ")}`);

    if (r.nbenfpre !== null && r.nbenfpre > 0) {
      parts.push(`${r.nbenfpre + 1}${r.nbenfpre + 1 === 1 ? "er" : "e"} enfant`);
    } else if (r.nbenfpre === 0) {
      parts.push("1er enfant");
    }

    return parts.join(" · ");
  }
</script>

<div class="layout">
  <aside class="filters">
    <h2>Filtres naissance</h2>

    <div class="field">
      <label for="b-month">Mois</label>
      <select id="b-month" bind:value={month}>
        {#each MONTHS as [n, label]}
          <option value={n}>{label}</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label for="b-dept">Département de naissance</label>
      <select id="b-dept" bind:value={dept}>
        <option value="">Tous</option>
        {#each DEPARTEMENTS as d}
          <option value={d.code}>{d.code} — {d.nom}</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label for="b-sex">Sexe enfant</label>
      <select id="b-sex" bind:value={sex}>
        <option value={0}>Tous</option>
        <option value={1}>Masculin</option>
        <option value={2}>Féminin</option>
      </select>
    </div>

    <fieldset class="range">
      <legend>Âge mère</legend>
      <input type="number" min="15" max="50" bind:value={age_mere_min} />
      <span>→</span>
      <input type="number" min="15" max="50" bind:value={age_mere_max} />
    </fieldset>

    <fieldset class="range">
      <legend>Âge père</legend>
      <input type="number" min="15" max="60" bind:value={age_pere_min} />
      <span>→</span>
      <input type="number" min="15" max="60" bind:value={age_pere_max} />
    </fieldset>

    <button onclick={() => load(0)} disabled={loading}>
      {loading ? "Calcul…" : "▶ Lancer"}
    </button>
  </aside>

  <section class="main">
    <h2>Naissances {data ? `(${data.total.toLocaleString("fr-FR")} trouvées)` : ""}</h2>

    {#if loading && !data}
      <p class="meta">Calcul en cours…</p>
    {:else if error}
      <p class="error">Erreur : {error}</p>
    {:else if !data}
      <p class="meta">Choisis tes filtres puis lance la recherche.</p>
    {:else if data.results.length === 0}
      <p class="meta">Aucune naissance ne correspond à ces filtres.</p>
    {:else}
      <p class="meta">Profils anonymes — INSEE ne publie pas les noms ni le jour exact.</p>
      <ol class="profiles">
        {#each data.results as r, i}
          <li class="profile">
            <span class="num">{i + 1}</span>
            <span class="text">{describe(r)}</span>
          </li>
        {/each}
      </ol>

      {#if data.has_more}
        <div class="actions">
          <button class="more" onclick={loadMore} disabled={loading}>
            {loading ? "Chargement…" : "Charger 50 de plus"}
          </button>
          <span class="meta inline">Affichés : {data.results.length} / {data.total.toLocaleString("fr-FR")}</span>
        </div>
      {:else}
        <p class="meta done">— Fin de la liste —</p>
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
    width: 60px;
    padding: 4px 6px;
    border: 1px solid var(--border);
  }
  .meta {
    font-size: 0.9rem;
    color: var(--text-mute);
    margin: 0 0 calc(var(--space) * 2);
  }
  .meta.inline { margin: 0; }
  .meta.done { text-align: center; margin-top: calc(var(--space) * 2); }
  .error { color: var(--rouge-rep); }
  .profiles {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .profile {
    display: flex;
    gap: calc(var(--space) * 2);
    padding: calc(var(--space) * 1.5);
    border-bottom: 1px solid var(--border);
    font-size: 0.92rem;
  }
  .profile:hover { background: var(--bg-panel); }
  .num {
    color: var(--text-mute);
    font-variant-numeric: tabular-nums;
    min-width: 40px;
    text-align: right;
  }
  .text { flex: 1; }
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
