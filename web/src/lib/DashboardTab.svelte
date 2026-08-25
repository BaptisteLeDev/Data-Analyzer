<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fetchDashboard, type DashboardResponse } from "./api";

  let data = $state<DashboardResponse | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let autoRefresh = $state(true);
  let interval: ReturnType<typeof setInterval> | null = null;
  let lastFetchMs = $state<number | null>(null);

  async function load() {
    loading = true;
    error = null;
    const t0 = performance.now();
    try {
      data = await fetchDashboard();
      lastFetchMs = Math.round(performance.now() - t0);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function toggleAuto() {
    autoRefresh = !autoRefresh;
    if (autoRefresh) {
      interval = setInterval(load, 5000);
    } else if (interval) {
      clearInterval(interval);
      interval = null;
    }
  }

  function formatTime(ts: number): string {
    if (!ts) return "—";
    const d = new Date(ts * 1000);
    return d.toLocaleTimeString("fr-FR", { hour12: false });
  }

  function statusClass(s: number): string {
    if (s >= 500) return "bad";
    if (s >= 400) return "warn";
    return "ok";
  }

  function durClass(ms: number): string {
    if (ms >= 500) return "bad";
    if (ms >= 100) return "warn";
    return "ok";
  }

  onMount(() => {
    load();
    interval = setInterval(load, 5000);
  });

  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
</script>

<div class="dashboard">
  <header class="head">
    <div>
      <h2>Tableau de bord</h2>
      <p class="sub">État de la base SQLite et trafic API en temps réel</p>
    </div>
    <div class="actions">
      <label class="toggle">
        <input type="checkbox" checked={autoRefresh} onchange={toggleAuto} />
        Auto-refresh 5 s
      </label>
      <button onclick={load} disabled={loading}>
        {loading ? "…" : "Rafraîchir"}
      </button>
    </div>
  </header>

  {#if error}
    <p class="error">Erreur : {error}</p>
  {/if}

  {#if data}
    <section class="grid kpis">
      <article class="kpi primary">
        <span class="kpi-label">Lignes totales</span>
        <span class="kpi-val">{data.total_rows.toLocaleString("fr-FR")}</span>
        <span class="kpi-hint">sur {data.tables.length} tables</span>
      </article>
      <article class="kpi">
        <span class="kpi-label">Taille DB</span>
        <span class="kpi-val">{data.db_size_human}</span>
        <span class="kpi-hint">{data.db_size_bytes.toLocaleString("fr-FR")} octets</span>
      </article>
      <article class="kpi">
        <span class="kpi-label">Prénoms distincts (FR)</span>
        <span class="kpi-val">{data.distinct_prenoms_nat.toLocaleString("fr-FR")}</span>
        <span class="kpi-hint">table prenoms_nat</span>
      </article>
      <article class="kpi">
        <span class="kpi-label">Prénoms distincts (US)</span>
        <span class="kpi-val">{data.distinct_prenoms_intl.toLocaleString("fr-FR")}</span>
        <span class="kpi-hint">table prenoms_intl</span>
      </article>
      <article class="kpi">
        <span class="kpi-label">Départements</span>
        <span class="kpi-val">{data.distinct_depts}</span>
        <span class="kpi-hint">y compris DOM</span>
      </article>
      <article class="kpi">
        <span class="kpi-label">SQLite</span>
        <span class="kpi-val mono">{data.sqlite_version}</span>
        <span class="kpi-hint">moteur embarqué</span>
      </article>
    </section>

    <section>
      <h3>Tables</h3>
      <table class="data">
        <thead>
          <tr>
            <th>Table</th>
            <th class="num">Lignes</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          {#each data.tables as t}
            <tr>
              <td class="mono">{t.name}</td>
              <td class="num">{t.rows.toLocaleString("fr-FR")}</td>
              <td>{t.description}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section>
      <h3>Plages d'années couvertes</h3>
      <table class="data">
        <thead>
          <tr>
            <th>Table</th>
            <th class="num">Année min</th>
            <th class="num">Année max</th>
            <th class="num">Étendue</th>
          </tr>
        </thead>
        <tbody>
          {#each data.year_ranges as y}
            <tr>
              <td class="mono">{y.table}</td>
              <td class="num">{y.min_year ?? "—"}</td>
              <td class="num">{y.max_year ?? "—"}</td>
              <td class="num">
                {y.min_year && y.max_year ? `${y.max_year - y.min_year + 1} ans` : "—"}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section>
      <h3>Origine des données</h3>
      <table class="data">
        <thead>
          <tr>
            <th>Table</th>
            <th>Source</th>
            <th>Granularité</th>
          </tr>
        </thead>
        <tbody>
          {#each data.sources as s}
            <tr>
              <td class="mono">{s.table}</td>
              <td>{s.origin}</td>
              <td class="muted">{s.grain}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <section>
      <h3>Accès base SQLite</h3>
      <dl class="meta">
        <dt>Chemin</dt>
        <dd class="mono path">{data.db_path}</dd>
        <dt>Moteur</dt>
        <dd>SQLite {data.sqlite_version} via rusqlite (bundled) + pool r2d2</dd>
        <dt>Dernière requête /api/dashboard</dt>
        <dd>{lastFetchMs ?? "—"} ms (mesuré client)</dd>
      </dl>
    </section>

    <section>
      <h3>Dernières requêtes API ({data.recent_requests.length}/10)</h3>
      {#if data.recent_requests.length === 0}
        <p class="muted">Aucune requête enregistrée encore.</p>
      {:else}
        <table class="data requests">
          <thead>
            <tr>
              <th>Heure</th>
              <th>Méthode</th>
              <th>Chemin</th>
              <th class="num">Status</th>
              <th class="num">Durée</th>
            </tr>
          </thead>
          <tbody>
            {#each data.recent_requests as r}
              <tr>
                <td class="mono">{formatTime(r.timestamp)}</td>
                <td class="mono">{r.method}</td>
                <td class="mono path">{r.path}</td>
                <td class="num {statusClass(r.status)}">{r.status}</td>
                <td class="num {durClass(r.duration_ms)}">{r.duration_ms} ms</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
      <p class="footnote">
        Buffer circulaire en mémoire (10 entrées). Reset au redémarrage du serveur. Tracking via middleware axum sur <code>/api/*</code>.
      </p>
    </section>
  {:else if !loading}
    <p>Chargement…</p>
  {/if}
</div>

<style>
  .dashboard {
    padding: calc(var(--space) * 3);
    display: flex;
    flex-direction: column;
    gap: calc(var(--space) * 3);
  }
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: var(--space);
    flex-wrap: wrap;
  }
  h2 {
    margin: 0;
    font-size: 1.4rem;
    color: var(--bleu-rep);
  }
  .sub {
    margin: 0;
    color: var(--text-mute);
    font-size: 0.9rem;
  }
  h3 {
    margin: 0 0 var(--space);
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    color: var(--text-mute);
    text-transform: uppercase;
  }
  .actions {
    display: flex;
    gap: var(--space);
    align-items: center;
  }
  .toggle {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    font-size: 0.85rem;
    color: var(--text-mute);
  }
  .error {
    background: #fee;
    border-left: 3px solid var(--rouge-rep);
    padding: var(--space) calc(var(--space) * 2);
    color: var(--rouge-rep);
  }
  .grid.kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: var(--space);
  }
  .kpi {
    background: white;
    border: 1px solid var(--border);
    padding: calc(var(--space) * 1.5) calc(var(--space) * 2);
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .kpi.primary {
    border-left: 4px solid var(--bleu-rep);
  }
  .kpi-label {
    font-size: 0.75rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-mute);
  }
  .kpi-val {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--bleu-rep);
    font-variant-numeric: tabular-nums;
  }
  .kpi-hint {
    font-size: 0.75rem;
    color: var(--text-mute);
  }
  table.data {
    width: 100%;
    border-collapse: collapse;
    background: white;
    font-size: 0.9rem;
  }
  table.data th,
  table.data td {
    padding: 0.55rem var(--space);
    border-bottom: 1px solid var(--border);
    text-align: left;
  }
  table.data th {
    background: var(--bg-panel);
    font-size: 0.75rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-mute);
  }
  table.data td.num,
  table.data th.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .mono {
    font-family: ui-monospace, "JetBrains Mono", Consolas, monospace;
    font-size: 0.85rem;
  }
  .path {
    word-break: break-all;
    max-width: 28rem;
  }
  .muted { color: var(--text-mute); }
  .ok    { color: #0a7d2a; }
  .warn  { color: #b16a00; }
  .bad   { color: var(--rouge-rep); font-weight: 700; }
  dl.meta {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.4rem var(--space);
    margin: 0;
    background: white;
    border: 1px solid var(--border);
    padding: calc(var(--space) * 1.5);
  }
  dl.meta dt {
    font-size: 0.75rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-mute);
    align-self: center;
  }
  dl.meta dd {
    margin: 0;
    align-self: center;
  }
  .footnote {
    font-size: 0.75rem;
    color: var(--text-mute);
    margin-top: var(--space);
    border-left: 3px solid var(--rouge-rep);
    padding-left: var(--space);
  }
  button {
    background: var(--bleu-rep);
    color: white;
    border: none;
    padding: 0.5rem var(--space);
    cursor: pointer;
    font-weight: 600;
    letter-spacing: 0.04em;
  }
  button:disabled { opacity: 0.5; cursor: wait; }
  code {
    font-family: ui-monospace, "JetBrains Mono", Consolas, monospace;
    font-size: 0.85rem;
    background: var(--bg-panel);
    padding: 0.05rem 0.3rem;
  }
</style>
