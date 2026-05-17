use anyhow::{Context, Result};
use encoding_rs::ISO_8859_15;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection};
use std::fs::File;
use std::io::BufReader;

pub fn load(conn: &mut Connection, csv_path: &str) -> Result<()> {
    let file = File::open(csv_path).with_context(|| format!("open {csv_path}"))?;
    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(ISO_8859_15))
        .bom_sniffing(true)
        .strip_bom(true)
        .build(BufReader::new(file));

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(transcoded);

    let tx = conn.transaction()?;
    let mut inserted: u64 = 0;
    let mut skipped: u64 = 0;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO prenoms (sexe, prenom, annee, dept, nombre) VALUES (?, ?, ?, ?, ?)",
        )?;
        for rec in rdr.records() {
            let rec = rec?;
            if rec.len() < 5 { skipped += 1; continue; }
            let sexe: i32 = match rec[0].trim().parse() { Ok(v) => v, Err(_) => { skipped += 1; continue; } };
            let prenom = rec[1].trim().to_string();
            let annais = rec[2].trim();
            if annais == "XXXX" { skipped += 1; continue; }
            let annee: i32 = match annais.parse() { Ok(v) => v, Err(_) => { skipped += 1; continue; } };
            let dpt_raw = rec[3].trim();
            if dpt_raw == "XX" || dpt_raw.is_empty() { skipped += 1; continue; }
            let dept = normalize_dept(dpt_raw);
            let nombre: i64 = match rec[4].trim().parse() { Ok(v) => v, Err(_) => { skipped += 1; continue; } };

            stmt.execute(params![sexe, prenom, annee, dept, nombre])?;
            inserted += 1;
        }
    }
    tx.commit()?;
    println!("  prenoms: {inserted} inserted, {skipped} skipped");
    Ok(())
}

/// Normalize the 3-char DPT field used by INSEE to a 2-char code where applicable.
/// "076" -> "76", "02A" -> "2A", "02B" -> "2B", "971" -> "971", "001" -> "01".
fn normalize_dept(s: &str) -> String {
    let s = s.trim();
    if s.len() == 3 && s.starts_with('9') {
        return s.to_string();
    }
    if s.len() == 3 && s.starts_with('0') {
        return s[1..].to_string();
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_dept;
    #[test]
    fn drop_leading_zero() {
        assert_eq!(normalize_dept("076"), "76");
        assert_eq!(normalize_dept("001"), "01");
        assert_eq!(normalize_dept("009"), "09");
    }
    #[test]
    fn corsica() {
        assert_eq!(normalize_dept("02A"), "2A");
        assert_eq!(normalize_dept("02B"), "2B");
    }
    #[test]
    fn dom() {
        assert_eq!(normalize_dept("971"), "971");
        assert_eq!(normalize_dept("976"), "976");
    }
}
