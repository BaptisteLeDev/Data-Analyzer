<script lang="ts">
  import Header from "./lib/Header.svelte";
  import Filters from "./lib/Filters.svelte";
  import Results from "./lib/Results.svelte";
  import Context from "./lib/Context.svelte";
  import { fetchRarest, fetchBirthContext, type RarestResponse, type BirthContext } from "./lib/api";

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

<div class="layout">
  <Filters
    bind:year
    bind:dept
    bind:month
    bind:letter
    bind:sex
    bind:search
    bind:exclude
    {loading}
    onsubmit={run}
  />
  <div class="main">
    <Results data={rarest} {loading} {error} onLoadMore={loadMore} />
    <Context data={ctx} />
  </div>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 320px 1fr;
    min-height: calc(100vh - 73px);
  }
  .main { display: flex; flex-direction: column; }
  @media (max-width: 800px) {
    .layout { grid-template-columns: 1fr; }
  }
</style>
