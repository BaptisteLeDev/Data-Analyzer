use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

/// Load the US SSA baby-names CSV into prenoms_intl.
///
/// Supports two file layouts inside `ssa_dir`:
///   1. SSA yob-style files: `yob{YEAR}.txt` with lines `name,sex,count` (M/F).
///   2. A single flat CSV (any name ending in `.csv`) with header
///      `year,sex,name,n,prop` (tidytuesday / babynames R package format).
///      The `sex` column uses "M"/"F" or "boy"/"girl".
pub fn load(conn: &mut Connection, ssa_dir: &str) -> Result<()> {
    let dir = Path::new(ssa_dir);
    if !dir.exists() {
        println!("  intl: skipped ({} not found)", ssa_dir);
        return Ok(());
    }

    // Collect yob*.txt files and *.csv files separately
    let entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("read_dir {ssa_dir}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    let yob_files: Vec<_> = entries
        .iter()
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.starts_with("yob") && name.ends_with(".txt")
        })
        .cloned()
        .collect();

    let csv_files: Vec<_> = entries
        .iter()
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).unwrap_or("") == "csv"
        })
        .cloned()
        .collect();

    if yob_files.is_empty() && csv_files.is_empty() {
        println!("  intl: skipped (no data files in {})", ssa_dir);
        return Ok(());
    }

    let tx = conn.transaction()?;
    let mut inserted: u64 = 0;
    let mut file_count = 0;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO prenoms_intl (prenom, sex, annee, nombre, source) VALUES (?, ?, ?, ?, 'US')",
        )?;

        // --- Format 1: yob{YEAR}.txt ---
        let mut yob_sorted = yob_files.clone();
        yob_sorted.sort();
        for path in &yob_sorted {
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let year: i32 = match fname
                .trim_start_matches("yob")
                .trim_end_matches(".txt")
                .parse()
            {
                Ok(v) => v,
                Err(_) => continue,
            };
            let content = fs::read_to_string(path)?;
            for line in content.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() != 3 {
                    continue;
                }
                let name = parts[0].trim().to_uppercase();
                let sex: i32 = match parts[1].trim() {
                    "M" => 1,
                    "F" => 2,
                    _ => continue,
                };
                let count: i64 = match parts[2].trim().parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                stmt.execute(params![name, sex, year, count])?;
                inserted += 1;
            }
            file_count += 1;
        }

        // --- Format 2: flat CSV with header year,sex,name,n,prop ---
        for path in &csv_files {
            let content = fs::read_to_string(path)?;
            let mut lines = content.lines();
            // Skip header
            let header = lines.next().unwrap_or("");
            // Detect column order from header
            let cols: Vec<&str> = header.split(',').collect();
            let idx_year = cols.iter().position(|c| *c == "year");
            let idx_sex  = cols.iter().position(|c| *c == "sex");
            let idx_name = cols.iter().position(|c| *c == "name");
            let idx_n    = cols.iter().position(|c| *c == "n");
            match (idx_year, idx_sex, idx_name, idx_n) {
                (Some(iy), Some(is), Some(im), Some(ic)) => {
                    for line in lines {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() <= iy.max(is).max(im).max(ic) {
                            continue;
                        }
                        let year: i32 = match parts[iy].trim().parse() {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let sex_raw = parts[is].trim();
                        let sex: i32 = match sex_raw {
                            "M" | "boy"  => 1,
                            "F" | "girl" => 2,
                            _ => continue,
                        };
                        let name = parts[im].trim().to_uppercase();
                        let count: i64 = match parts[ic].trim().parse() {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        stmt.execute(params![name, sex, year, count])?;
                        inserted += 1;
                    }
                    file_count += 1;
                }
                _ => {
                    println!("  intl: skipping {:?} — unrecognised CSV header: {}", path, header);
                }
            }
        }
    }

    tx.commit()?;
    println!(
        "  intl: {} year-name rows inserted ({} files)",
        inserted, file_count
    );
    Ok(())
}
