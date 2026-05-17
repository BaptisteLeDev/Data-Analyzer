# Data-Analyzer — Prénoms rares INSEE 2006

Explore the 20 rarest French first names of 2006 by département and letter.

## Stack
- Rust ETL (`prep/`) → SQLite
- Rust axum API (`server/`)
- Svelte + Vite + Bun front (`web/`)

## Run

1. Drop INSEE files in `data/`:
   - `nais2006.dbf` (from `etatcivil2006_nais2006_dbase.zip`)
   - `dpt2006.csv` (from `dpt_2000_2021_csv.zip`)
2. Build the SQLite database:
   ```powershell
   cd prep
   cargo run --release
   ```
3. Launch the API:
   ```powershell
   cd server
   cargo run --release
   ```
4. Launch the web UI:
   ```powershell
   cd web
   bun install
   bun run dev
   ```
5. Open http://localhost:5173

## Data sources
- INSEE Fichier des prénoms par département (allégé 2000-2021): https://www.insee.fr/fr/statistiques/2540004
- INSEE Fichier détail naissances 2006
- Etalab decoupage-administratif: https://github.com/etalab/decoupage-administratif

See `docs/superpowers/specs/2026-05-17-rare-names-design.md` for the full design.
