<script lang="ts">
  import { DEPARTEMENTS } from "./departements";

  type Props = {
    year: number;
    dept: string;
    month: number;
    letter: string;
    sex: 0 | 1 | 2;
    search: string;
    exclude: string;
    loading: boolean;
    onsubmit: () => void;
  };

  let {
    year = $bindable(),
    dept = $bindable(),
    month = $bindable(),
    letter = $bindable(),
    sex = $bindable(),
    search = $bindable(),
    exclude = $bindable(),
    loading,
    onsubmit
  }: Props = $props();

  const YEARS = Array.from({ length: 22 }, (_, i) => 2000 + i);
  const MONTHS = [
    [1, "Janvier"], [2, "Février"], [3, "Mars"], [4, "Avril"],
    [5, "Mai"], [6, "Juin"], [7, "Juillet"], [8, "Août"],
    [9, "Septembre"], [10, "Octobre"], [11, "Novembre"], [12, "Décembre"]
  ] as const;
</script>

<aside class="filters">
  <h2>Filtres</h2>

  <div class="field">
    <label for="f-year">Année</label>
    <select id="f-year" bind:value={year}>
      {#each YEARS as y}
        <option value={y}>{y}</option>
      {/each}
    </select>
  </div>

  <div class="field">
    <label for="f-dept">Département</label>
    <select id="f-dept" bind:value={dept}>
      <option value="">Tous les départements</option>
      {#each DEPARTEMENTS as d}
        <option value={d.code}>{d.code} — {d.nom}</option>
      {/each}
    </select>
  </div>

  <div class="field">
    <label for="f-sex">Sexe</label>
    <select id="f-sex" bind:value={sex}>
      <option value={0}>Tous</option>
      <option value={1}>Masculin</option>
      <option value={2}>Féminin</option>
    </select>
  </div>

  <div class="field">
    <label for="f-month">Mois <span class="hint">(contexte uniquement)</span></label>
    <select id="f-month" bind:value={month}>
      {#each MONTHS as [n, label]}
        <option value={n}>{label}</option>
      {/each}
    </select>
  </div>

  <div class="field">
    <label for="f-letter">Lettre obligatoire <span class="hint">(vide = aucune)</span></label>
    <input
      id="f-letter"
      type="text"
      maxlength="1"
      bind:value={letter}
      oninput={(e) => letter = (e.currentTarget as HTMLInputElement).value.toUpperCase()}
    />
  </div>

  <div class="field">
    <label for="f-search">Recherche <span class="hint">(doit contenir)</span></label>
    <input
      id="f-search"
      type="text"
      placeholder="ex : LOU"
      bind:value={search}
      oninput={(e) => search = (e.currentTarget as HTMLInputElement).value.toUpperCase()}
    />
  </div>

  <div class="field">
    <label for="f-exclude">Exclusion <span class="hint">(ne doit pas contenir)</span></label>
    <input
      id="f-exclude"
      type="text"
      placeholder="ex : LL → exclut Elliot"
      bind:value={exclude}
      oninput={(e) => exclude = (e.currentTarget as HTMLInputElement).value.toUpperCase()}
    />
  </div>

  <button onclick={onsubmit} disabled={loading}>
    {loading ? "Calcul…" : "▶ Lancer"}
  </button>
</aside>

<style>
  .filters {
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    padding: calc(var(--space) * 3);
    min-width: 280px;
    display: flex;
    flex-direction: column;
    gap: calc(var(--space) * 2);
  }
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
</style>
