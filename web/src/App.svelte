<script lang="ts">
  import Header from "./lib/Header.svelte";
  import Filters from "./lib/Filters.svelte";
  import Results from "./lib/Results.svelte";
  import Context from "./lib/Context.svelte";
  import BirthsTab from "./lib/BirthsTab.svelte";
  import RaresNatTab from "./lib/RaresNatTab.svelte";
  import { fetchRarest, fetchBirthContext, type RarestResponse, type BirthContext } from "./lib/api";

  let tab = $state<"rares" | "rares_nat" | "births">("rares");

  // ---- Prénoms rares state ----
  let year = $state(2006);
  let dept = $state("76");
  let month = $state(5);
  let letter = $state("L");
  let sex = $state<0 | 1 | 2>(0);
  let search = $state("");
  let exclude = $state("");
  let limit = $state(20);

  let loading = $state(false);
  let error = $state<string | null>(null);
  let rarest = $state<RarestResponse | null>(null);
  let ctx = $state<BirthContext | null>(null);

  async function fetchNames(useLimit: number) {
    loading = true;
    error = null;
    try {
      const [r, c] = await Promise.all([
        fetchRarest({ year, dept, letter, sex, search, exclude, limit: useLimit }),
        fetchBirthContext({ year, dept, month })
      ]);
      rarest = r;
      ctx = c;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      rarest = null;
      ctx = null;
    } finally {
      loading = false;
    }
  }

  function run() {
    limit = 20;
    fetchNames(limit);
  }

  function loadMore() {
    limit += 20;
    fetchNames(limit);
  }
</script>

<Header />

<nav class="tabs">
  <button class:active={tab === "rares"} onclick={() => tab = "rares"}>Prénoms rares (dept)</button>
  <button class:active={tab === "rares_nat"} onclick={() => tab = "rares_nat"}>Rares nationaux</button>
  <button class:active={tab === "births"} onclick={() => tab = "births"}>Naissances 2006</button>
</nav>

{#if tab === "rares"}
  <div class="layout">
    <Filters bind:year bind:dept bind:month bind:letter bind:sex bind:search bind:exclude {loading} onsubmit={run} />
    <div class="main">
      <Results data={rarest} {loading} {error} onLoadMore={loadMore} />
      <Context data={ctx} />
    </div>
  </div>
{:else if tab === "rares_nat"}
  <RaresNatTab />
{:else}
  <BirthsTab />
{/if}

<style>
  .tabs {
    display: flex;
    background: white;
    border-bottom: 1px solid var(--border);
    padding: 0 calc(var(--space) * 3);
  }
  .tabs button {
    background: transparent;
    color: var(--text-mute);
    padding: calc(var(--space) * 1.5) calc(var(--space) * 3);
    border-bottom: 3px solid transparent;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .tabs button:hover { color: var(--text); }
  .tabs button.active {
    color: var(--bleu-rep);
    border-bottom-color: var(--bleu-rep);
  }
  .layout {
    display: grid;
    grid-template-columns: 320px 1fr;
    min-height: calc(100vh - 130px);
  }
  .main { display: flex; flex-direction: column; }
  @media (max-width: 800px) {
    .layout { grid-template-columns: 1fr; }
  }
</style>
