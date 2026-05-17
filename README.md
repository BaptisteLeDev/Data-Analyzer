# Data-Analyzer — Prénoms rares INSEE 2006

Web application showing the **20 rarest French first names of 2006** containing a chosen letter,
filtered by département. Includes a contextual monthly-births panel from the INSEE detailed births file.

Design inspired by the official French Republic visual identity (palette `#000091` / `#E1000F` / Marianne).

![logo: tricolor block with DA monogram](web/public/favicon.svg)

## Stack
- **`prep/`** — Rust ETL: reads `nais2006.dbf` + `dpt2006.csv` + Etalab JSON → SQLite (`data/analyzer.sqlite`)
- **`server/`** — Rust axum API on `:8787` (3 endpoints, r2d2-pooled SQLite)
- **`web/`** — Svelte 5 + Vite + Bun front on `:5173`, proxied to the API

## Prerequisites
- Rust 1.75+ (`rustup`)
- Bun 1.x (`bun --version`)
- INSEE source files placed in `data/`:
  - `nais2006.dbf` — extracted from `etatcivil2006_nais2006_dbase.zip` (INSEE fichier détail naissances 2006)
  - `dpt2006.csv` — extracted from `dpt_2000_2021_csv.zip`; the ETL ingests every year present, the API only queries the year you request

## Run

```powershell
# 1. Build database (one-shot)
cd prep
cargo run --release

# 2. Launch API (keep running)
cd ../server
cargo run --release

# 3. In another terminal: launch web
cd ../web
bun install
bun run dev
```

Open http://localhost:5173. Defaults: 2006 · Seine-Maritime (76) · Mai · Lettre "L".

## API

| Endpoint                                                    | Returns                                    |
| ----------------------------------------------------------- | ------------------------------------------ |
| `GET /api/departements`                                     | Official Etalab dept list                  |
| `GET /api/rarest?year=2006&dept=76&letter=L&limit=20`       | Top N rarest names ascending by count      |
| `GET /api/birth-context?year=2006&dept=76&month=5`          | Monthly births and yearly share for dept   |

## Data caveats

- The INSEE first-names file is aggregated **by year × department × name × sex** — no month granularity.
- The INSEE detailed-births file is **anonymous** — no first names. Provides month/dept demographics only.
- The Mois filter therefore drives the contextual panel only; it does not re-rank the names list.
- Names with < 3 occurrences are censored by INSEE and aggregated under `_PRENOMS_RARES` (filtered out).

## Documentation

- Design: `docs/superpowers/specs/2026-05-17-rare-names-design.md`
- Plan: `docs/superpowers/plans/2026-05-17-rare-names.md`
