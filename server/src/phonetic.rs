//! Phonetic materialization + helpers.
//!
//! We use DoubleMetaphone (the most language-tolerant general phonetic algo
//! shipped by `rphonetic`). It is not strictly French-tuned, but works
//! reasonably across French and anglicised names — which is exactly the
//! cross-language matching we need for the "anglais ↔ français" scenario.
//!
//! The phonetic codes are materialized once into two tables:
//!   - `prenoms_nat_phon (prenom TEXT PRIMARY KEY, phon TEXT, phon_alt TEXT)`
//!   - `prenoms_intl_phon (prenom TEXT PRIMARY KEY, phon TEXT, phon_alt TEXT)`
//!
//! `ensure_materialized()` runs at server start: it creates tables/indexes if
//! missing and populates them when empty. For the current data set this is
//! sub-second and adds ~5 MB to the SQLite file.

use anyhow::Result;
use rphonetic::{DoubleMetaphone, Encoder};
use rusqlite::Connection;

/// Cheap wrapper so callers (handlers) can compute codes for query terms
/// without re-allocating an encoder each time.
pub struct Phon {
    dm: DoubleMetaphone,
}

impl Phon {
    pub fn new() -> Self {
        // 6 chars is enough headroom for short first names while still
        // collapsing close variants. Default (4) loses too much signal.
        Self { dm: DoubleMetaphone::new(Some(6)) }
    }

    pub fn encode(&self, s: &str) -> (String, String) {
        // rphonetic DoubleMetaphone operates on ASCII bytes internally and
        // panics on multi-byte UTF-8 codepoints. Skip non-ASCII names to
        // avoid a crash; they will simply not appear in the phonetic index.
        if !s.is_ascii() {
            return (String::new(), String::new());
        }
        let primary = self.dm.encode(s);
        let alt = self.dm.encode_alternate(s);
        (primary, alt)
    }
}

impl Default for Phon {
    fn default() -> Self { Self::new() }
}

/// Create the phonetic tables (idempotent) and populate them from
/// `prenoms_nat` / `prenoms_intl` if they are empty.
pub fn ensure_materialized(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "
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
        ",
    )?;

    let nat_count: i64 = conn.query_row("SELECT COUNT(*) FROM prenoms_nat_phon", [], |r| r.get(0))?;
    let intl_count: i64 = conn.query_row("SELECT COUNT(*) FROM prenoms_intl_phon", [], |r| r.get(0))?;

    let phon = Phon::new();

    if nat_count == 0 {
        let names: Vec<String> = {
            let mut s = conn.prepare(
                "SELECT DISTINCT prenom FROM prenoms_nat
                  WHERE prenom NOT IN ('_PRENOMS_RARES', 'XXXX') AND length(prenom) > 1",
            )?;
            let out: Vec<String> = s
                .query_map([], |row| row.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            out
        };
        let tx = conn.transaction()?;
        {
            let mut ins = tx.prepare(
                "INSERT OR IGNORE INTO prenoms_nat_phon(prenom, phon, phon_alt) VALUES (?, ?, ?)",
            )?;
            for n in &names {
                let (p, a) = phon.encode(n);
                ins.execute(rusqlite::params![n, p, a])?;
            }
        }
        tx.commit()?;
        tracing::info!("phonetic: materialized {} entries into prenoms_nat_phon", names.len());
    }

    if intl_count == 0 {
        let names: Vec<String> = {
            let mut s = conn.prepare(
                "SELECT DISTINCT prenom FROM prenoms_intl WHERE length(prenom) > 1",
            )?;
            let out: Vec<String> = s
                .query_map([], |row| row.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            out
        };
        let tx = conn.transaction()?;
        {
            let mut ins = tx.prepare(
                "INSERT OR IGNORE INTO prenoms_intl_phon(prenom, phon, phon_alt) VALUES (?, ?, ?)",
            )?;
            for n in &names {
                let (p, a) = phon.encode(n);
                ins.execute(rusqlite::params![n, p, a])?;
            }
        }
        tx.commit()?;
        tracing::info!("phonetic: materialized {} entries into prenoms_intl_phon", names.len());
    }

    Ok(())
}
