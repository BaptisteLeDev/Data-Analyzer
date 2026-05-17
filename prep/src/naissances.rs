use anyhow::{Context, Result};
use dbase::{FieldValue, Reader};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

type Key = (i32, String, String, i32);

pub fn load(conn: &mut Connection, dbf_path: &str) -> Result<()> {
    let file = File::open(dbf_path).with_context(|| format!("open {dbf_path}"))?;
    let mut reader = Reader::new(BufReader::new(file))?;
    let mut counts: HashMap<Key, i64> = HashMap::with_capacity(50_000);

    for record in reader.iter_records() {
        let record = record?;
        let mois = field_int(&record, "MNAIS").unwrap_or(0);
        let dept_dom = field_str(&record, "DEPDOM").unwrap_or_default();
        let dept_nais = field_str(&record, "DEPNAIS").unwrap_or_default();
        let sexe = field_int(&record, "SEXE").unwrap_or(0);
        if mois < 1 || mois > 12 { continue; }
        let key = (mois, dept_dom, dept_nais, sexe);
        *counts.entry(key).or_insert(0) += 1;
    }

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO naissances (mois, dept_dom, dept_nais, sexe, count) VALUES (?, ?, ?, ?, ?)",
        )?;
        for ((mois, dd, dn, sx), n) in &counts {
            stmt.execute(params![mois, dd, dn, sx, n])?;
        }
    }
    tx.commit()?;
    println!("  naissances: {} aggregated rows", counts.len());
    Ok(())
}

fn field_str(record: &dbase::Record, name: &str) -> Option<String> {
    match record.get(name) {
        Some(FieldValue::Character(Some(s))) => Some(s.trim().to_string()),
        Some(FieldValue::Character(None)) => None,
        _ => None,
    }
}

fn field_int(record: &dbase::Record, name: &str) -> Option<i32> {
    match record.get(name) {
        Some(FieldValue::Character(Some(s))) => s.trim().parse().ok(),
        Some(FieldValue::Numeric(Some(n))) => Some(*n as i32),
        _ => None,
    }
}
