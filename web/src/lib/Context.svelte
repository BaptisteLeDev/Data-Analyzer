<script lang="ts">
  import type { BirthContext } from "./api";
  import { DEPARTEMENTS } from "./departements";

  type Props = { data: BirthContext | null };
  const { data }: Props = $props();

  const MONTH_NAMES = [
    "", "Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
    "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"
  ];

  function deptName(code: string): string {
    return DEPARTEMENTS.find(d => d.code === code)?.nom ?? code;
  }
</script>

{#if data}
  <section class="context">
    <h3>Contexte démographique</h3>
    <p class="line">
      <strong>{MONTH_NAMES[data.month]} 2006</strong> ·
      {deptName(data.dept)} ({data.dept})
    </p>
    <p class="line big">
      <span class="num">{data.month_births.toLocaleString("fr-FR")}</span> naissances
    </p>
    <p class="line muted">
      {data.share_pct.toLocaleString("fr-FR")}% des {data.year_births.toLocaleString("fr-FR")} naissances annuelles dans ce département
    </p>
    <p class="footnote">
      Source : INSEE — fichier détail des naissances. Le filtre Mois pilote ce contexte uniquement, il ne ré-ordonne pas la liste des prénoms rares (le fichier prénoms INSEE est agrégé à l'année).
    </p>
  </section>
{/if}

<style>
  .context {
    margin-top: calc(var(--space) * 3);
    padding: calc(var(--space) * 2) calc(var(--space) * 3);
    border-top: 2px solid var(--border);
    background: var(--bg-panel);
  }
  h3 {
    margin: 0 0 var(--space);
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    color: var(--text-mute);
    text-transform: uppercase;
  }
  .line { margin: var(--space) 0; }
  .big { font-size: 1.2rem; }
  .num {
    font-weight: 700;
    color: var(--bleu-rep);
    font-variant-numeric: tabular-nums;
  }
  .muted { color: var(--text-mute); font-size: 0.9rem; }
  .footnote {
    font-size: 0.75rem;
    color: var(--text-mute);
    margin-top: calc(var(--space) * 2);
    border-left: 3px solid var(--rouge-rep);
    padding-left: var(--space);
  }
</style>
