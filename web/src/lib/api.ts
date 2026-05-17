const BASE = "/api";

export type Dept = { code: string; nom: string };

export type RarestRow = { rank: number; prenom: string; sexe: 1 | 2; n: number };

export type RarestResponse = {
  year: number;
  dept: string;
  letter: string;
  sex: 0 | 1 | 2;
  search: string;
  exclude: string;
  limit: number;
  has_more: boolean;
  censored_count: number;
  results: RarestRow[];
};

export type BirthContext = {
  dept: string;
  month: number;
  month_births: number;
  year_births: number;
  share_pct: number;
};

export async function fetchDepartements(): Promise<Dept[]> {
  const r = await fetch(`${BASE}/departements`);
  if (!r.ok) throw new Error(`departements: ${r.status}`);
  return r.json();
}

export async function fetchRarest(p: {
  year: number;
  dept: string;
  letter: string;
  sex?: 0 | 1 | 2;
  search?: string;
  exclude?: string;
  limit?: number;
}): Promise<RarestResponse> {
  const qs = new URLSearchParams({
    year: String(p.year),
    dept: p.dept,
    letter: p.letter,
    limit: String(p.limit ?? 20)
  });
  if (p.sex && p.sex !== 0) qs.set("sex", String(p.sex));
  if (p.search && p.search.trim()) qs.set("search", p.search.trim());
  if (p.exclude && p.exclude.trim()) qs.set("exclude", p.exclude.trim());
  const r = await fetch(`${BASE}/rarest?${qs}`);
  if (!r.ok) throw new Error(`rarest: ${r.status}`);
  return r.json();
}

export async function fetchBirthContext(p: {
  year: number;
  dept: string;
  month: number;
}): Promise<BirthContext> {
  const qs = new URLSearchParams({
    year: String(p.year),
    dept: p.dept,
    month: String(p.month)
  });
  const r = await fetch(`${BASE}/birth-context?${qs}`);
  if (!r.ok) throw new Error(`birth-context: ${r.status}`);
  return r.json();
}

export type BirthRow = {
  mois: number;
  dept_dom: string | null;
  dept_nais: string | null;
  sexe: 1 | 2;
  age_mere: number | null;
  age_pere: number | null;
  situ_mere: string | null;
  situ_pere: string | null;
  nat_mere: 1 | 2 | null;
  nat_pere: 1 | 2 | null;
  ln_mere: string | null;
  ln_pere: string | null;
  nbenfpre: number | null;
};

export type BirthsResponse = {
  total: number;
  has_more: boolean;
  limit: number;
  offset: number;
  results: BirthRow[];
};

export type ForeignFilter = "excl" | "only" | "all";

export type RarestNatRow = {
  rank: number;
  prenom: string;
  sexe: 1 | 2;
  n: number;
  dept_count: number;
};

export type RarestNatResponse = {
  year: number;
  letter: string;
  sex: 0 | 1 | 2;
  search: string;
  exclude: string;
  limit: number;
  has_more: boolean;
  censored_count: number;
  results: RarestNatRow[];
};

export type CandidateRow = {
  rank: number;
  prenom: string;
  sexe: 1 | 2;
  first_year: number;
  last_year: number;
  total_hist: number;
};

export type CandidatesResponse = {
  year: number;
  letter: string;
  sex: 0 | 1 | 2;
  search: string;
  exclude: string;
  limit: number;
  has_more: boolean;
  results: CandidateRow[];
};

export async function fetchCandidates(p: {
  year: number;
  letter: string;
  sex?: 0 | 1 | 2;
  search?: string;
  exclude?: string;
  limit?: number;
}): Promise<CandidatesResponse> {
  const qs = new URLSearchParams({
    year: String(p.year),
    letter: p.letter,
    limit: String(p.limit ?? 20)
  });
  if (p.sex && p.sex !== 0) qs.set("sex", String(p.sex));
  if (p.search && p.search.trim()) qs.set("search", p.search.trim());
  if (p.exclude && p.exclude.trim()) qs.set("exclude", p.exclude.trim());
  const r = await fetch(`${BASE}/candidates?${qs}`);
  if (!r.ok) throw new Error(`candidates: ${r.status}`);
  return r.json();
}

export async function fetchRarestNat(p: {
  year: number;
  letter: string;
  sex?: 0 | 1 | 2;
  search?: string;
  exclude?: string;
  limit?: number;
}): Promise<RarestNatResponse> {
  const qs = new URLSearchParams({
    year: String(p.year),
    letter: p.letter,
    limit: String(p.limit ?? 20)
  });
  if (p.sex && p.sex !== 0) qs.set("sex", String(p.sex));
  if (p.search && p.search.trim()) qs.set("search", p.search.trim());
  if (p.exclude && p.exclude.trim()) qs.set("exclude", p.exclude.trim());
  const r = await fetch(`${BASE}/rarest-nat?${qs}`);
  if (!r.ok) throw new Error(`rarest-nat: ${r.status}`);
  return r.json();
}

export type IntlRow = {
  rank: number;
  prenom: string;
  sex: 1 | 2;
  total_us: number;
  total_uk: number;
  sources: ("US" | "UK")[];
  first_year: number;
  last_year: number;
  era_count: number;
  has_double_variant: boolean;
  variant_example: string | null;
};

export type IntlResponse = {
  letter: string;
  sex: 0 | 1 | 2;
  search: string;
  exclude: string;
  era_start: number;
  era_end: number;
  absent_fr: string;
  double_variant: boolean;
  limit: number;
  has_more: boolean;
  results: IntlRow[];
};

export async function fetchIntl(p: {
  letter: string;
  sex?: 0 | 1 | 2;
  search?: string;
  exclude?: string;
  era_start?: number;
  era_end?: number;
  absent_fr?: string;
  double_variant?: boolean;
  limit?: number;
}): Promise<IntlResponse> {
  const qs = new URLSearchParams({
    letter: p.letter,
    era_start: String(p.era_start ?? 1985),
    era_end: String(p.era_end ?? 2005),
    absent_fr: p.absent_fr ?? "any",
    limit: String(p.limit ?? 30)
  });
  if (p.sex && p.sex !== 0) qs.set("sex", String(p.sex));
  if (p.search && p.search.trim()) qs.set("search", p.search.trim());
  if (p.exclude && p.exclude.trim()) qs.set("exclude", p.exclude.trim());
  if (p.double_variant) qs.set("double_variant", "1");
  const r = await fetch(`${BASE}/intl-search?${qs}`);
  if (!r.ok) throw new Error(`intl-search: ${r.status}`);
  return r.json();
}

export async function fetchBirths(p: {
  month: number;
  dept: string;
  sex: 0 | 1 | 2;
  age_mere_min: number;
  age_mere_max: number;
  age_pere_min: number;
  age_pere_max: number;
  foreign?: ForeignFilter;
  limit?: number;
  offset?: number;
}): Promise<BirthsResponse> {
  const qs = new URLSearchParams({
    month: String(p.month),
    dept: p.dept,
    sex: String(p.sex),
    age_mere_min: String(p.age_mere_min),
    age_mere_max: String(p.age_mere_max),
    age_pere_min: String(p.age_pere_min),
    age_pere_max: String(p.age_pere_max),
    foreign: p.foreign ?? "excl",
    limit: String(p.limit ?? 50),
    offset: String(p.offset ?? 0)
  });
  const r = await fetch(`${BASE}/births?${qs}`);
  if (!r.ok) throw new Error(`births: ${r.status}`);
  return r.json();
}

export type TableCount = {
  name: string;
  rows: number;
  description: string;
};

export type DataSource = {
  table: string;
  origin: string;
  grain: string;
};

export type YearRange = {
  table: string;
  min_year: number | null;
  max_year: number | null;
};

export type RequestLog = {
  method: string;
  path: string;
  status: number;
  duration_ms: number;
  timestamp: number;
};

export type DashboardResponse = {
  db_path: string;
  db_size_bytes: number;
  db_size_human: string;
  sqlite_version: string;
  total_rows: number;
  tables: TableCount[];
  sources: DataSource[];
  year_ranges: YearRange[];
  distinct_prenoms_nat: number;
  distinct_prenoms_intl: number;
  distinct_depts: number;
  recent_requests: RequestLog[];
};

export async function fetchDashboard(): Promise<DashboardResponse> {
  const r = await fetch(`${BASE}/dashboard`);
  if (!r.ok) throw new Error(`dashboard: ${r.status}`);
  return r.json();
}

// ---- /intl-match (multi-algo cross-language matching) ----

export type IntlMatchAlgo = "phonetic" | "lev2" | "anglicisation";

export type IntlMatchRow = {
  rank: number;
  prenom: string;
  matched_by: IntlMatchAlgo[];
  score: number;
  lev_distance: number | null;
  from_intl: string[];
  anglo_rules: string[];
  insee_total: number;
  insee_years: number;
};

export type IntlMatchResponse = {
  letter: string;
  sex: 0 | 1 | 2;
  n_min: number;
  n_max: number;
  one_l: boolean;
  lev_max: number;
  algos: IntlMatchAlgo[];
  intl_seed_limit: number;
  limit: number;
  has_more: boolean;
  results: IntlMatchRow[];
};

export async function fetchIntlMatch(p: {
  letter?: string;
  sex?: 0 | 1 | 2;
  search?: string;
  exclude?: string;
  era_start?: number;
  era_end?: number;
  n_min?: number;
  n_max?: number;
  one_l?: boolean;
  lev_max?: number;
  intl_seed_limit?: number;
  algos?: IntlMatchAlgo[];
  limit?: number;
}): Promise<IntlMatchResponse> {
  const qs = new URLSearchParams({
    letter: p.letter ?? "",
    era_start: String(p.era_start ?? 1985),
    era_end: String(p.era_end ?? 2005),
    n_min: String(p.n_min ?? 5),
    n_max: String(p.n_max ?? 100),
    lev_max: String(p.lev_max ?? 2),
    intl_seed_limit: String(p.intl_seed_limit ?? 800),
    limit: String(p.limit ?? 50)
  });
  if (p.sex && p.sex !== 0) qs.set("sex", String(p.sex));
  if (p.search && p.search.trim()) qs.set("search", p.search.trim());
  if (p.exclude && p.exclude.trim()) qs.set("exclude", p.exclude.trim());
  if (p.one_l) qs.set("one_l", "1");
  if (p.algos && p.algos.length) qs.set("algos", p.algos.join(","));
  const r = await fetch(`${BASE}/intl-match?${qs}`);
  if (!r.ok) throw new Error(`intl-match: ${r.status}`);
  return r.json();
}
