use anyhow::{Context, Result};
use dbase::{FieldValue, Reader};
use rusqlite::{params, Connection};
use std::fs::File;
use std::io::BufReader;

pub fn load(conn: &mut Connection, dbf_path: &str) -> Result<()> {
    let file = File::open(dbf_path).with_context(|| format!("open {dbf_path}"))?;
    let mut reader = Reader::new(BufReader::new(file))?;

    let tx = conn.transaction()?;
    let mut inserted: u64 = 0;
    let mut skipped: u64 = 0;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO prenoms_nat (sexe, prenom, annee, nombre) VALUES (?, ?, ?, ?)",
        )?;
        for record in reader.iter_records() {
            let record = record?;
            // Field names in nat2021.dbf are lowercase
            let sexe = field_int(&record, "sexe").unwrap_or(0);
            if sexe != 1 && sexe != 2 { skipped += 1; continue; }
            let prenom = field_str(&record, "preusuel").unwrap_or_default();
            if prenom.is_empty() { skipped += 1; continue; }
            let annee_str = field_str(&record, "annais").unwrap_or_default();
            if annee_str == "XXXX" { skipped += 1; continue; }
            let annee: i32 = match annee_str.parse() { Ok(v) => v, Err(_) => { skipped += 1; continue; } };
            let nombre = field_num(&record, "nombre").unwrap_or(0.0) as i64;
            stmt.execute(params![sexe, prenom, annee, nombre])?;
            inserted += 1;
        }
    }
    tx.commit()?;
    println!("  prenoms_nat: {inserted} inserted, {skipped} skipped");
    Ok(())
}

fn field_str(record: &dbase::Record, name: &str) -> Option<String> {
    match record.get(name) {
        Some(FieldValue::Character(Some(s))) => {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        }
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

fn field_num(record: &dbase::Record, name: &str) -> Option<f64> {
    match record.get(name) {
        Some(FieldValue::Numeric(Some(n))) => Some(*n),
        Some(FieldValue::Character(Some(s))) => s.trim().parse().ok(),
        _ => None,
    }
}
