use crate::errors::ApiError;
use crate::metrics::RequestLog;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::fs;

pub fn router() -> Router<AppState> {
    Router::new().route("/dashboard", get(dashboard))
}

#[derive(Serialize)]
struct TableCount {
    name: String,
    rows: i64,
    description: String,
}

#[derive(Serialize)]
struct DataSource {
    table: String,
    origin: String,
    grain: String,
}

#[derive(Serialize)]
struct YearRange {
    table: String,
    min_year: Option<i32>,
    max_year: Option<i32>,
}

#[derive(Serialize)]
struct DashboardResp {
    db_path: String,
    db_size_bytes: u64,
    db_size_human: String,
    sqlite_version: String,
    total_rows: i64,
    tables: Vec<TableCount>,
    sources: Vec<DataSource>,
    year_ranges: Vec<YearRange>,
    distinct_prenoms_nat: i64,
    distinct_prenoms_intl: i64,
    distinct_depts: i64,
    recent_requests: Vec<RequestLog>,
}

async fn dashboard(State(s): State<AppState>) -> Result<Json<DashboardResp>, ApiError> {
    let conn = s.pool.get()?;

    let sqlite_version: String =
        conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))?;

    let db_size_bytes = fs::metadata(&s.db_path).map(|m| m.len()).unwrap_or(0);
    let db_size_human = format_size(db_size_bytes);

    let table_meta: &[(&str, &str)] = &[
        ("departements", "Liste officielle des départements (Etalab)"),
        ("prenoms", "Prénoms par année × département × sexe (INSEE)"),
        ("prenoms_nat", "Prénoms au niveau national par année × sexe (INSEE)"),
        ("naissances", "Naissances individuelles anonymisées 2006 (INSEE)"),
        ("prenoms_intl", "Prénoms enregistrés aux États-Unis (US SSA)"),
    ];

    let mut tables = Vec::new();
    let mut total_rows: i64 = 0;
    for (name, desc) in table_meta {
        let n: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {}", name), [], |row| row.get(0))
            .unwrap_or(0);
        total_rows += n;
        tables.push(TableCount {
            name: (*name).to_string(),
            rows: n,
            description: (*desc).to_string(),
        });
    }

    let sources = vec![
        DataSource {
            table: "departements".into(),
            origin: "Etalab — data.gouv.fr".into(),
            grain: "code → nom".into(),
        },
        DataSource {
            table: "prenoms".into(),
            origin: "INSEE — Fichier prénoms par département (Dpt*depuis2000.csv)".into(),
            grain: "sexe × prenom × annee × dept × nombre".into(),
        },
        DataSource {
            table: "prenoms_nat".into(),
            origin: "INSEE — Fichier prénoms national 1900-2021 (nat2021.dbf)".into(),
            grain: "sexe × prenom × annee × nombre".into(),
        },
        DataSource {
            table: "naissances".into(),
            origin: "INSEE — Fichier détail naissances 2006 (NAIS2006.dbf)".into(),
            grain: "1 ligne par naissance (mois, dept, âges parents, nationalité)".into(),
        },
        DataSource {
            table: "prenoms_intl".into(),
            origin: "US Social Security Administration — yobYYYY.txt".into(),
            grain: "prenom × sex × annee × nombre".into(),
        },
    ];

    let year_ranges = vec![
        year_range(&conn, "prenoms"),
        year_range(&conn, "prenoms_nat"),
        year_range(&conn, "prenoms_intl"),
    ];

    let distinct_prenoms_nat: i64 = conn
        .query_row("SELECT COUNT(DISTINCT prenom) FROM prenoms_nat", [], |r| r.get(0))
        .unwrap_or(0);
    let distinct_prenoms_intl: i64 = conn
        .query_row("SELECT COUNT(DISTINCT prenom) FROM prenoms_intl", [], |r| r.get(0))
        .unwrap_or(0);
    let distinct_depts: i64 = conn
        .query_row("SELECT COUNT(*) FROM departements", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(Json(DashboardResp {
        db_path: s.db_path.clone(),
        db_size_bytes,
        db_size_human,
        sqlite_version,
        total_rows,
        tables,
        sources,
        year_ranges,
        distinct_prenoms_nat,
        distinct_prenoms_intl,
        distinct_depts,
        recent_requests: s.metrics.snapshot(),
    }))
}

fn year_range(conn: &rusqlite::Connection, table: &str) -> YearRange {
    let sql = format!("SELECT MIN(annee), MAX(annee) FROM {}", table);
    let (min_year, max_year): (Option<i32>, Option<i32>) = conn
        .query_row(&sql, [], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap_or((None, None));
    YearRange { table: table.to_string(), min_year, max_year }
}

fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.2} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.1} KiB", b / KIB)
    } else {
        format!("{} B", bytes)
    }
}
