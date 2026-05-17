use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Dept {
    code: String,
    nom: String,
}

pub fn load(conn: &mut Connection, json_path: &str) -> Result<()> {
    let raw = fs::read_to_string(json_path)
        .with_context(|| format!("reading {json_path}"))?;
    let list: Vec<Dept> = serde_json::from_str(&raw)?;

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO departements (code, nom) VALUES (?, ?)")?;
        for d in &list {
            stmt.execute(params![d.code, d.nom])?;
        }
    }
    tx.commit()?;
    println!("  departements: {} loaded", list.len());
    Ok(())
}
