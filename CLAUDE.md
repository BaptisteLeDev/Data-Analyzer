# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Web app that surfaces the rarest French first names from INSEE open data, with several cross-referenced views (per-département, national, "theoretical candidates" missing from a given year, and US/international comparison). UI is in French. Default view: 2006 · Seine-Maritime (76) · Mai · "L".

## Architecture

Three independent components in one repo. No workspace file — each has its own toolchain.

```
prep/   (Rust binary)  →  data/analyzer.sqlite  ←  server/  (Rust axum :8787)
                                                       ↓ ServeDir fallback
                                                   web/dist  ←  web/  (Svelte 5 + Vite)
```

- **`prep/`** — One-shot ETL. `main.rs` deletes `data/analyzer.sqlite`, re-creates it via `schema.rs`, then loads in order: departements (JSON) → prenoms (Dpt*.csv, ISO-8859-15, `;`-delimited) → prenoms_nat (nat2021.dbf) → naissances (NAIS2006.dbf) → intl (US SSA `yobYYYY.txt` or flat CSV). Indices built last. Re-run any time raw data changes.
- **`server/`** — `main.rs` boots axum at `127.0.0.1:8787`, mounts `/api` from `handlers.rs`, and falls back to serving `web/dist` static files via `ServeDir` (so the prod deployment is a single binary on a single port). DB access is pooled via r2d2 + r2d2_sqlite (`state.rs`, max 8 connections). The `resolve()` helper walks up from the exe and from cwd to find `data/analyzer.sqlite` and `web/dist`, so `cargo run` from either repo root or `server/` works.
- **`web/`** — Svelte 5 (uses runes: `$state`, `bind:`) + Vite + Bun. `App.svelte` is a tab router over `Filters/Results/Context` (plus `BirthsTab`, `RaresNatTab`, `CandidatesTab`, `IntlTab`). All HTTP goes through `src/lib/api.ts` against a relative `/api` base — in dev mode (`bun run dev` on 5173) Vite proxies `/api` to `:8787`; in prod, the Rust server serves the built `dist/`.

### Database tables (created/dropped by `prep/src/schema.rs`)

| Table          | Grain                              | Source                                |
| -------------- | ---------------------------------- | ------------------------------------- |
| `departements` | code → nom                         | `web/src/data/departements.json`      |
| `prenoms`      | sexe × prenom × annee × dept × n   | `data/Dpt*depuis2000.csv`             |
| `prenoms_nat`  | sexe × prenom × annee × n          | `data/nat2021.dbf`                    |
| `naissances`   | one row per birth (no first names) | `data/NAIS2006.dbf`                   |
| `prenoms_intl` | prenom × sex × annee × n           | `data/ssa_names/` (US SSA, optional)  |

Department codes are normalized: `076 → 76`, `02A → 2A`, `971 → 971` (`prep/src/prenoms.rs::normalize_dept`).

### API endpoints (all under `/api`)

`/departements`, `/rarest` (per-dept), `/rarest-nat` (national + dept_count cross-ref), `/candidates` (in INSEE 1900-2021 but not in target year — proxy for the censored bucket), `/intl-search` (US-only names absent from INSEE, optional doubling-letter variant check), `/birth-context` (monthly share for dept), `/births` (filtered list of naissances rows). Defaults assume year=2006, dept=76, month=5.

## Commands

Bun and Cargo. PowerShell on Windows; commands work the same in bash.

```powershell
# Build the SQLite DB (run after raw data changes, ~30 s)
cd prep ; cargo run --release

# Build the front-end into web/dist (~500 ms)
cd web ; bun install ; bun run build

# Serve API + static front on :8787
cd server ; cargo run --release

# Dev front-end with HMR on :5173 (proxies /api to :8787 — server must also be running)
cd web ; bun run dev

# Unit tests (only prep has them right now — dept-normalization)
cd prep ; cargo test
cd prep ; cargo test normalize_dept::dom    # single test

# Type-check Svelte/TS
cd web ; bun x svelte-check
```

## Data caveats (important when modifying queries or adding endpoints)

- INSEE prénoms file is aggregated by **year × dept × name × sex** — no month granularity. The "Mois" filter only drives the `/birth-context` panel; it does **not** re-rank the names list.
- INSEE detailed-births file (`naissances`) is anonymous — no first names. Provides month/dept/parent-age/nationality demographics only.
- Names with < 3 occurrences are censored by INSEE and bucketed as `_PRENOMS_RARES`. All name queries must filter out `_PRENOMS_RARES` and `XXXX` and `length(prenom) > 1` (see `handlers.rs`).
- `nat_mere`/`nat_pere` = 1 means French, 2 means foreign. The `/births` `foreign` param defaults to `excl` (both parents French or NULL); `only` (≥1 foreign) and `all` are the alternatives.
- Raw data files live in `data/` and are gitignored (large INSEE/SSA archives). The ETL bails if expected files are missing — `ssa_names/` is the only optional input.

## Conventions

- Don't commit `Cargo.lock`, `node_modules`, `dist`, or anything under `data/` except `.gitkeep` (see `.gitignore`).
- French UI strings stay French; design palette is the French Republic identity (`#000091`, `#E1000F`).
- Svelte 5 runes only (no legacy `export let`); state is per-component, no store layer.
- All cross-cutting query filters (letter, sex, search, exclude, censored exclusion) are SQL-side, not post-filtered — keep it that way to preserve pagination/`has_more` semantics.

## Documentation

Design and plan documents live under `docs/superpowers/{specs,plans}/`.
