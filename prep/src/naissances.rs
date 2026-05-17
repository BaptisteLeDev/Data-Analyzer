use anyhow::{Context, Result};
use dbase::{FieldValue, Reader};
use rusqlite::{params, Connection};
use std::fs::File;
use std::io::BufReader;

pub fn load(conn: &mut Connection, dbf_path: &str) -> Result<()> {
    let file = File::open(dbf_path).with_context(|| format!("open {dbf_path}"))?;
    let mut reader = Reader::new(BufReader::new(file))?;

    let tx = conn.transaction()?;
    let mut count: u64 = 0;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO naissances
             (mois, dept_dom, dept_nais, sexe, age_mere, age_pere,
              situ_mere, situ_pere, nat_mere, nat_pere, ln_mere, ln_pere,
              accouchr, nbenfpre, dmarnais, tudom)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )?;
        for record in reader.iter_records() {
            let record = record?;
            let mois = field_int(&record, "MNAIS").unwrap_or(0);
            if mois < 1 || mois > 12 { continue; }
            stmt.execute(params![
                mois,
                field_str(&record, "DEPDOM"),
                field_str(&record, "DEPNAIS"),
                field_int(&record, "SEXE").unwrap_or(0),
                field_int(&record, "AGEMERE"),
                field_int(&record, "AGEPERE"),
                field_str(&record, "SITUATMR"),
                field_str(&record, "SITUATPR"),
                field_int(&record, "INDNATM"),
                field_int(&record, "INDNATP"),
                field_str(&record, "INDLNM"),
                field_str(&record, "INDLNP"),
                field_str(&record, "ACCOUCHR"),
                field_int(&record, "NBENFPRE"),
                field_str(&record, "DMARNAIS"),
                field_str(&record, "TUDOM"),
            ])?;
            count += 1;
        }
    }
    tx.commit()?;
    println!("  naissances: {count} individual records");
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
