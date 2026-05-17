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
