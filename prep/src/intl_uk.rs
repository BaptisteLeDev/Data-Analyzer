use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

/// Load UK ONS baby-names CSVs into prenoms_intl (source = 'UK').
///
/// Supports several file layouts that ONS mirrors use:
///
/// Layout A — single flat tidy CSV, header: `year,sex,name,count`  (or `year,sex,name,n`)
///   sex column: "M"/"F", "boy"/"girl", "1"/"2", "MALE"/"FEMALE"
///
/// Layout B — gender-segregated files, sex inferred from filename containing "boy"/"girl"/"male"/"female"
///   Sub-layouts:
///     B1: header `year,name,count` (or `year,name,n`)
///     B2: header `rank,name,count` (or `rank,name,frequency`) — requires year from filename (`*YYYY*.csv`)
///
/// Any file that does not match a recognised layout is skipped with a warning.
pub fn load(conn: &mut Connection, uk_dir: &str) -> Result<()> {
    let dir = Path::new(uk_dir);
    if !dir.exists() {
        println!("  intl_uk: skipped ({} not found)", uk_dir);
        return Ok(());
    }

    let mut files: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("read_dir {uk_dir}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "csv"))
        .collect();
    files.sort();

    if files.is_empty() {
        println!("  intl_uk: skipped (no CSV files in {})", uk_dir);
        return Ok(());
    }

    let tx = conn.transaction()?;
    let mut inserted: u64 = 0;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO prenoms_intl (prenom, sex, annee, nombre, source) VALUES (?, ?, ?, ?, 'UK')",
        )?;

        for path in &files {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            let content = fs::read_to_string(path)
                .with_context(|| format!("reading {:?}", path))?;
            let mut lines = content.lines();
            let header_line = match lines.next() {
                Some(h) => h,
                None => continue,
            };
            let header: Vec<String> = header_line
                .split(',')
                .map(|c| c.trim().trim_matches('"').to_lowercase())
                .collect();

            // Determine layout from header columns
            let idx_year = header.iter().position(|c| c == "year");
            let idx_sex  = header.iter().position(|c| c == "sex");
            let idx_name = header.iter().position(|c| c == "name");
            let idx_n    = header.iter().position(|c| c == "n" || c == "count" || c == "frequency" || c == "freq");
            let idx_rank = header.iter().position(|c| c == "rank");

            // --- Layout A: year + sex + name + count (all in one file) ---
            if let (Some(iy), Some(is), Some(im), Some(ic)) = (idx_year, idx_sex, idx_name, idx_n) {
                for line in &mut lines {
                    let parts: Vec<&str> = line.split(',').collect();
                    let max_idx = iy.max(is).max(im).max(ic);
                    if parts.len() <= max_idx {
                        continue;
                    }
                    let year: i32 = match parts[iy].trim().trim_matches('"').parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let sex = parse_sex(parts[is].trim().trim_matches('"'));
                    let sex = match sex {
                        Some(s) => s,
                        None => continue,
                    };
                    let name = parts[im].trim().trim_matches('"').to_uppercase();
                    if name.is_empty() {
                        continue;
                    }
                    let count: i64 = match parts[ic].trim().trim_matches('"').parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    stmt.execute(params![name, sex, year, count])?;
                    inserted += 1;
                }
                continue;
            }

            // Infer sex from filename for gender-segregated layouts
            let file_sex: Option<i32> = if fname.contains("boy") || fname.contains("male") || fname.contains("_m_") {
                Some(1)
            } else if fname.contains("girl") || fname.contains("female") || fname.contains("_f_") {
                Some(2)
            } else {
                None
            };

            // Extract year from filename (4 consecutive digits)
            let file_year: Option<i32> = fname
                .chars()
                .collect::<String>()
                .split(|c: char| !c.is_ascii_digit())
                .find(|s| s.len() == 4)
                .and_then(|s| s.parse().ok());

            // --- Layout B1: year + name + count (sex from filename) ---
            if let (Some(iy), Some(im), Some(ic)) = (idx_year, idx_name, idx_n) {
                if let Some(sex) = file_sex {
                    for line in &mut lines {
                        let parts: Vec<&str> = line.split(',').collect();
                        let max_idx = iy.max(im).max(ic);
                        if parts.len() <= max_idx {
                            continue;
                        }
                        let year: i32 = match parts[iy].trim().trim_matches('"').parse() {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let name = parts[im].trim().trim_matches('"').to_uppercase();
                        if name.is_empty() {
                            continue;
                        }
                        let count: i64 = match parts[ic].trim().trim_matches('"').parse() {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        stmt.execute(params![name, sex, year, count])?;
                        inserted += 1;
                    }
                    continue;
                }
            }

            // --- Layout B2: rank + name + count (sex and year from filename) ---
            if let (Some(_ir), Some(im), Some(ic)) = (idx_rank, idx_name, idx_n) {
                if let (Some(sex), Some(year)) = (file_sex, file_year) {
                    for line in &mut lines {
                        let parts: Vec<&str> = line.split(',').collect();
                        let max_idx = im.max(ic);
                        if parts.len() <= max_idx {
                            continue;
                        }
                        let name = parts[im].trim().trim_matches('"').to_uppercase();
                        if name.is_empty() {
                            continue;
                        }
                        let count: i64 = match parts[ic].trim().trim_matches('"').parse() {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        stmt.execute(params![name, sex, year, count])?;
                        inserted += 1;
                    }
                    continue;
                }
            }

            // --- Unrecognised layout ---
            println!(
                "  intl_uk: skipping {:?} — unrecognised CSV header: {}",
                path, header_line
            );
        }
    }
    tx.commit()?;
    println!(
        "  intl_uk: {} rows inserted ({} files)",
        inserted,
        files.len()
    );
    Ok(())
}

/// Parse sex string to 1 (M) or 2 (F). Returns None for unrecognised values.
fn parse_sex(s: &str) -> Option<i32> {
    match s.to_uppercase().as_str() {
        "M" | "MALE" | "BOY" | "BOYS" | "1" => Some(1),
        "F" | "FEMALE" | "GIRL" | "GIRLS" | "2" => Some(2),
        _ => None,
    }
}
