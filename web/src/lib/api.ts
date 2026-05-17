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
