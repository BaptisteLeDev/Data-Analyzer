-- Migration 001: phonetic materialization tables for /api/intl-match.
--
-- Apply on an existing data/analyzer.sqlite produced by an older prep build,
-- so the user doesn't have to re-run the full ETL (which re-loads the DBFs).
--
-- Usage:
--   sqlite3 data/analyzer.sqlite < prep/migrations/001_phonetic.sql
--
-- Note: this script only CREATES the tables/indexes. The phonetic codes
-- themselves are populated by the server on first boot
-- (see `server/src/phonetic.rs::ensure_materialized`). That keeps the
-- migration provider-free (no Rust needed to run the SQL) while still
-- benefiting from DoubleMetaphone via the server binary.

CREATE TABLE IF NOT EXISTS prenoms_nat_phon (
    prenom   TEXT PRIMARY KEY,
    phon     TEXT NOT NULL,
    phon_alt TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_nat_phon_phon     ON prenoms_nat_phon(phon);
CREATE INDEX IF NOT EXISTS idx_nat_phon_phon_alt ON prenoms_nat_phon(phon_alt);

CREATE TABLE IF NOT EXISTS prenoms_intl_phon (
    prenom   TEXT PRIMARY KEY,
    phon     TEXT NOT NULL,
    phon_alt TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_intl_phon_phon     ON prenoms_intl_phon(phon);
CREATE INDEX IF NOT EXISTS idx_intl_phon_phon_alt ON prenoms_intl_phon(phon_alt);
