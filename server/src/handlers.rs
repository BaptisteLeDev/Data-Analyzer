use crate::errors::ApiError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use rusqlite::params;
use serde::{Deserialize, Serialize};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/departements", get(departements))
        .route("/rarest", get(rarest))
        .route("/rarest-nat", get(rarest_nat))
        .route("/candidates", get(candidates))
        .route("/birth-context", get(birth_context))
        .route("/births", get(births))
        .route("/intl-search", get(intl_search))
        .route("/intl-match", get(intl_match))
}

/// Log EXPLAIN QUERY PLAN at debug level. Useful when adding new queries —
/// scan over indexed tables shows up as `SCAN <table>` (bad) vs
/// `SEARCH <table> USING INDEX <name>` (good).
#[allow(dead_code)]
pub fn explain_query_plan(conn: &rusqlite::Connection, sql: &str, params: &[&dyn rusqlite::ToSql]) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    let eqp = format!("EXPLAIN QUERY PLAN {sql}");
    let Ok(mut stmt) = conn.prepare(&eqp) else { return };
    let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(params.iter()), |r| {
        Ok(format!(
            "{}|{}|{}|{}",
            r.get::<_, i64>(0).unwrap_or(0),
            r.get::<_, i64>(1).unwrap_or(0),
            r.get::<_, i64>(2).unwrap_or(0),
            r.get::<_, String>(3).unwrap_or_default()
        ))
    }) else { return };
    for row in rows.flatten() {
        if row.contains("SCAN ") && !row.contains("USING INDEX") && !row.contains("USING COVERING") {
            tracing::warn!("EQP SCAN (no index): {row} for {sql}");
        } else {
            tracing::debug!("EQP: {row}");
        }
    }
}

// ---------- /departements ----------

#[derive(Serialize)]
struct Dept {
    code: String,
    nom: String,
}

async fn departements(State(s): State<AppState>) -> Result<Json<Vec<Dept>>, ApiError> {
    let conn = s.pool.get()?;
    let mut stmt = conn.prepare("SELECT code, nom FROM departements ORDER BY code")?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Dept {
                code: row.get(0)?,
                nom: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(rows))
}

// ---------- /rarest ----------

#[derive(Deserialize)]
pub struct RarestQuery {
    pub year: Option<i32>,
    pub dept: Option<String>,
    pub letter: Option<String>,
    pub sex: Option<i32>,
    pub search: Option<String>,
    pub exclude: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct RarestRow {
    rank: usize,
    prenom: String,
    sexe: i32,
    n: i64,
}

#[derive(Serialize)]
struct RarestResp {
    year: i32,
    dept: String,
    letter: String,
    sex: i32,
    search: String,
    exclude: String,
    limit: i64,
    has_more: bool,
    censored_count: i64,
    results: Vec<RarestRow>,
}

async fn rarest(
    State(s): State<AppState>,
    Query(q): Query<RarestQuery>,
) -> Result<Json<RarestResp>, ApiError> {
    let year = q.year.unwrap_or(2006);
    let dept = q.dept.unwrap_or_default().trim().to_string();
    let letter = q.letter.unwrap_or_default().to_uppercase();
    let letter_pattern = if letter.is_empty() { "%".to_string() } else { format!("%{}%", letter) };
    let sex = q.sex.filter(|&v| v == 1 || v == 2).unwrap_or(0);
    let search = q.search.unwrap_or_default().trim().to_uppercase();
    let search_pattern = format!("%{}%", search);
    let exclude = q.exclude.unwrap_or_default().trim().to_uppercase();
    let exclude_pattern = format!("%{}%", exclude);
    let limit = q.limit.unwrap_or(20).min(500) as i64;

    let conn = s.pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT prenom, sexe, SUM(nombre) AS n
         FROM prenoms
         WHERE annee = ?1
           AND (?2 = '' OR dept = ?2)
           AND UPPER(prenom) LIKE ?3
           AND (?4 = 0 OR sexe = ?4)
           AND (?5 = '' OR UPPER(prenom) LIKE ?6)
           AND (?7 = '' OR UPPER(prenom) NOT LIKE ?8)
           AND prenom NOT IN ('_PRENOMS_RARES', 'XXXX')
           AND length(prenom) > 1
         GROUP BY prenom, sexe
         ORDER BY n ASC, prenom ASC
         LIMIT ?9",
    )?;
    let rows = stmt
        .query_map(
            params![year, dept, letter_pattern, sex, search, search_pattern, exclude, exclude_pattern, limit + 1],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?, row.get::<_, i64>(2)?)),
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = rows.len() as i64 > limit;
    let results: Vec<RarestRow> = rows
        .into_iter()
        .take(limit as usize)
        .enumerate()
        .map(|(i, (prenom, sexe, n))| RarestRow { rank: i + 1, prenom, sexe, n })
        .collect();

    let censored_count: i64 = conn.query_row(
        "SELECT COALESCE(SUM(nombre), 0)
         FROM prenoms
         WHERE annee = ?1
           AND (?2 = '' OR dept = ?2)
           AND (?3 = 0 OR sexe = ?3)
           AND prenom = '_PRENOMS_RARES'",
        params![year, dept, sex],
        |row| row.get(0),
    )?;

    Ok(Json(RarestResp {
        year, dept, letter, sex, search, exclude, limit, has_more, censored_count, results,
    }))
}

// ---------- /birth-context ----------

#[derive(Deserialize)]
pub struct ContextQuery {
    pub year: Option<i32>,
    pub dept: Option<String>,
    pub month: Option<u32>,
}

#[derive(Serialize)]
struct ContextResp {
    dept: String,
    month: u32,
    month_births: i64,
    year_births: i64,
    share_pct: f64,
}

async fn birth_context(
    State(s): State<AppState>,
    Query(q): Query<ContextQuery>,
) -> Result<Json<ContextResp>, ApiError> {
    let _year = q.year.unwrap_or(2006);
    let dept = q.dept.unwrap_or_default().trim().to_string();
    let month = q.month.unwrap_or(5);

    let conn = s.pool.get()?;

    let month_births: i64 = conn.query_row(
        "SELECT COUNT(*) FROM naissances
         WHERE (?1 = '' OR dept_nais = ?1) AND mois = ?2",
        params![dept, month],
        |row| row.get(0),
    )?;

    let year_births: i64 = conn.query_row(
        "SELECT COUNT(*) FROM naissances WHERE (?1 = '' OR dept_nais = ?1)",
        params![dept],
        |row| row.get(0),
    )?;

    let share_pct = if year_births > 0 {
        (month_births as f64 * 100.0 / year_births as f64 * 10.0).round() / 10.0
    } else {
        0.0
    };

    Ok(Json(ContextResp {
        dept,
        month,
        month_births,
        year_births,
        share_pct,
    }))
}

// ---------- /births ----------

#[derive(Deserialize)]
pub struct BirthsQuery {
    pub month: Option<i32>,
    pub dept: Option<String>,
    pub sex: Option<i32>,
    pub age_mere_min: Option<i32>,
    pub age_mere_max: Option<i32>,
    pub age_pere_min: Option<i32>,
    pub age_pere_max: Option<i32>,
    /// "excl" (default) — both parents not flagged as étranger
    /// "only" — at least one parent étranger
    /// "all"  — no nationality filter
    pub foreign: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Serialize)]
struct BirthRow {
    mois: i32,
    dept_dom: Option<String>,
    dept_nais: Option<String>,
    sexe: i32,
    age_mere: Option<i32>,
    age_pere: Option<i32>,
    situ_mere: Option<String>,
    situ_pere: Option<String>,
    nat_mere: Option<i32>,
    nat_pere: Option<i32>,
    ln_mere: Option<String>,
    ln_pere: Option<String>,
    nbenfpre: Option<i32>,
}

#[derive(Serialize)]
struct BirthsResp {
    total: i64,
    has_more: bool,
    limit: i64,
    offset: i64,
    results: Vec<BirthRow>,
}

async fn births(
    State(s): State<AppState>,
    Query(q): Query<BirthsQuery>,
) -> Result<Json<BirthsResp>, ApiError> {
    let dept = q.dept.unwrap_or_default().trim().to_string();
    let sex = q.sex.filter(|&v| v == 1 || v == 2).unwrap_or(0);
    let month = q.month.filter(|&m| (1..=12).contains(&m)).unwrap_or(0);
    let age_mere_min = q.age_mere_min.unwrap_or(0);
    let age_mere_max = q.age_mere_max.unwrap_or(99);
    let age_pere_min = q.age_pere_min.unwrap_or(0);
    let age_pere_max = q.age_pere_max.unwrap_or(99);
    let foreign = q.foreign.unwrap_or_else(|| "excl".to_string());
    let foreign_sql = match foreign.as_str() {
        "only" => "AND (nat_mere = 2 OR nat_pere = 2)",
        "all"  => "",
        _      => "AND (nat_mere IS NULL OR nat_mere = 1) AND (nat_pere IS NULL OR nat_pere = 1)",
    };
    let limit = q.limit.unwrap_or(50).min(500) as i64;
    let offset = q.offset.unwrap_or(0) as i64;

    let conn = s.pool.get()?;

    let where_clause = format!("
         WHERE (?1 = '' OR dept_nais = ?1)
           AND (?2 = 0 OR mois = ?2)
           AND (?3 = 0 OR sexe = ?3)
           AND (age_mere IS NULL OR age_mere BETWEEN ?4 AND ?5)
           AND (age_pere IS NULL OR age_pere BETWEEN ?6 AND ?7)
           {}", foreign_sql);

    let count_sql = format!("SELECT COUNT(*) FROM naissances {}", where_clause);
    let total: i64 = conn.query_row(
        &count_sql,
        params![dept, month, sex, age_mere_min, age_mere_max, age_pere_min, age_pere_max],
        |row| row.get(0),
    )?;

    let list_sql = format!(
        "SELECT mois, dept_dom, dept_nais, sexe, age_mere, age_pere,
                situ_mere, situ_pere, nat_mere, nat_pere, ln_mere, ln_pere, nbenfpre
         FROM naissances {} ORDER BY id LIMIT ?8 OFFSET ?9",
        where_clause
    );
    let mut stmt = conn.prepare(&list_sql)?;
    let rows = stmt
        .query_map(
            params![dept, month, sex, age_mere_min, age_mere_max, age_pere_min, age_pere_max, limit, offset],
            |row| Ok(BirthRow {
                mois: row.get(0)?,
                dept_dom: row.get(1)?,
                dept_nais: row.get(2)?,
                sexe: row.get(3)?,
                age_mere: row.get(4)?,
                age_pere: row.get(5)?,
                situ_mere: row.get(6)?,
                situ_pere: row.get(7)?,
                nat_mere: row.get(8)?,
                nat_pere: row.get(9)?,
                ln_mere: row.get(10)?,
                ln_pere: row.get(11)?,
                nbenfpre: row.get(12)?,
            }),
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = offset + limit < total;
    Ok(Json(BirthsResp { total, has_more, limit, offset, results: rows }))
}

// ---------- /rarest-nat ----------

#[derive(Deserialize)]
pub struct RarestNatQuery {
    pub year: Option<i32>,
    pub letter: Option<String>,
    pub sex: Option<i32>,
    pub search: Option<String>,
    pub exclude: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct RarestNatRow {
    rank: usize,
    prenom: String,
    sexe: i32,
    n: i64,
    dept_count: i64,
}

#[derive(Serialize)]
struct RarestNatResp {
    year: i32,
    letter: String,
    sex: i32,
    search: String,
    exclude: String,
    limit: i64,
    has_more: bool,
    censored_count: i64,
    results: Vec<RarestNatRow>,
}

async fn rarest_nat(
    State(s): State<AppState>,
    Query(q): Query<RarestNatQuery>,
) -> Result<Json<RarestNatResp>, ApiError> {
    let year = q.year.unwrap_or(2006);
    let letter = q.letter.unwrap_or_default().to_uppercase();
    let letter_pattern = if letter.is_empty() { "%".to_string() } else { format!("%{}%", letter) };
    let sex = q.sex.filter(|&v| v == 1 || v == 2).unwrap_or(0);
    let search = q.search.unwrap_or_default().trim().to_uppercase();
    let search_pattern = format!("%{}%", search);
    let exclude = q.exclude.unwrap_or_default().trim().to_uppercase();
    let exclude_pattern = format!("%{}%", exclude);
    let limit = q.limit.unwrap_or(20).min(500) as i64;

    let conn = s.pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT prenom, sexe, nombre,
                (SELECT COUNT(DISTINCT dept) FROM prenoms p2
                  WHERE p2.annee = prenoms_nat.annee
                    AND p2.prenom = prenoms_nat.prenom
                    AND p2.sexe = prenoms_nat.sexe) AS dept_count
         FROM prenoms_nat
         WHERE annee = ?1
           AND UPPER(prenom) LIKE ?2
           AND (?3 = 0 OR sexe = ?3)
           AND (?4 = '' OR UPPER(prenom) LIKE ?5)
           AND (?6 = '' OR UPPER(prenom) NOT LIKE ?7)
           AND prenom NOT IN ('_PRENOMS_RARES', 'XXXX')
           AND length(prenom) > 1
         ORDER BY nombre ASC, prenom ASC
         LIMIT ?8",
    )?;
    let rows = stmt
        .query_map(
            params![year, letter_pattern, sex, search, search_pattern, exclude, exclude_pattern, limit + 1],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            )),
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = rows.len() as i64 > limit;
    let results: Vec<RarestNatRow> = rows
        .into_iter()
        .take(limit as usize)
        .enumerate()
        .map(|(i, (prenom, sexe, n, dept_count))| RarestNatRow {
            rank: i + 1, prenom, sexe, n, dept_count,
        })
        .collect();

    let censored_count: i64 = conn.query_row(
        "SELECT COALESCE(SUM(nombre), 0)
         FROM prenoms_nat
         WHERE annee = ?1 AND (?2 = 0 OR sexe = ?2) AND prenom = '_PRENOMS_RARES'",
        params![year, sex],
        |row| row.get(0),
    )?;

    Ok(Json(RarestNatResp {
        year, letter, sex, search, exclude, limit, has_more, censored_count, results,
    }))
}

// ---------- /candidates ----------
//
// "Theoretical" candidate names that might be in the censored bucket for a given year.
// Strategy: take every distinct (prenom, sexe) known to INSEE across 1900-2021
// (universe set), then exclude those present in the target year's national file.
// What remains are names INSEE saw historically but NOT in `year` — plausible
// candidates for the _PRENOMS_RARES bucket of that year.

#[derive(Deserialize)]
pub struct CandidatesQuery {
    pub year: Option<i32>,
    pub letter: Option<String>,
    pub sex: Option<i32>,
    pub search: Option<String>,
    pub exclude: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct CandidateRow {
    rank: usize,
    prenom: String,
    sexe: i32,
    first_year: i32,
    last_year: i32,
    total_hist: i64,
}

#[derive(Serialize)]
struct CandidatesResp {
    year: i32,
    letter: String,
    sex: i32,
    search: String,
    exclude: String,
    limit: i64,
    has_more: bool,
    results: Vec<CandidateRow>,
}

async fn candidates(
    State(s): State<AppState>,
    Query(q): Query<CandidatesQuery>,
) -> Result<Json<CandidatesResp>, ApiError> {
    let year = q.year.unwrap_or(2006);
    let letter = q.letter.unwrap_or_default().to_uppercase();
    let letter_pattern = if letter.is_empty() { "%".to_string() } else { format!("%{}%", letter) };
    let sex = q.sex.filter(|&v| v == 1 || v == 2).unwrap_or(0);
    let search = q.search.unwrap_or_default().trim().to_uppercase();
    let search_pattern = format!("%{}%", search);
    let exclude = q.exclude.unwrap_or_default().trim().to_uppercase();
    let exclude_pattern = format!("%{}%", exclude);
    let limit = q.limit.unwrap_or(20).min(500) as i64;

    let conn = s.pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT prenom, sexe, MIN(annee) AS first_year, MAX(annee) AS last_year, SUM(nombre) AS total_hist
         FROM prenoms_nat
         WHERE prenom NOT IN ('_PRENOMS_RARES', 'XXXX')
           AND length(prenom) > 1
           AND UPPER(prenom) LIKE ?1
           AND (?2 = 0 OR sexe = ?2)
           AND (?3 = '' OR UPPER(prenom) LIKE ?4)
           AND (?5 = '' OR UPPER(prenom) NOT LIKE ?6)
           AND NOT EXISTS (
             SELECT 1 FROM prenoms_nat p2
             WHERE p2.annee = ?7
               AND p2.prenom = prenoms_nat.prenom
               AND p2.sexe = prenoms_nat.sexe
           )
         GROUP BY prenom, sexe
         ORDER BY total_hist ASC, prenom ASC
         LIMIT ?8",
    )?;
    let rows = stmt
        .query_map(
            params![letter_pattern, sex, search, search_pattern, exclude, exclude_pattern, year, limit + 1],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, i64>(4)?,
            )),
        )?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = rows.len() as i64 > limit;
    let results: Vec<CandidateRow> = rows
        .into_iter()
        .take(limit as usize)
        .enumerate()
        .map(|(i, (prenom, sexe, first_year, last_year, total_hist))| CandidateRow {
            rank: i + 1, prenom, sexe, first_year, last_year, total_hist,
        })
        .collect();

    Ok(Json(CandidatesResp {
        year, letter, sex, search, exclude, limit, has_more, results,
    }))
}

// ---------- /intl-search ----------
//
// Find rare English/international names that are likely "imports" — present in
// US SSA records but absent from French INSEE. Useful for tracking down a name
// inspired by 90s English-language pop culture.

#[derive(Deserialize)]
pub struct IntlQuery {
    pub letter: Option<String>,
    pub sex: Option<i32>,
    pub search: Option<String>,
    pub exclude: Option<String>,
    /// If set (e.g. 1990, 2000), restrict to names that had ≥1 US occurrences during that era
    pub era_start: Option<i32>,
    pub era_end: Option<i32>,
    /// "any" (default) — filter only names absent from prenoms_nat for any year
    /// "year:2006" — absent from prenoms_nat for that specific year only
    pub absent_fr: Option<String>,
    /// If "1", only return names whose doubling-letter variant ALSO exists in intl
    /// (e.g. ELIOT → checks if ELLIOT also exists)
    pub double_variant: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct IntlRow {
    rank: usize,
    prenom: String,
    sex: i32,
    total_us: i64,
    total_uk: i64,
    sources: Vec<String>,
    first_year: i32,
    last_year: i32,
    era_count: i64,
    has_double_variant: bool,
    variant_example: Option<String>,
}

#[derive(Serialize)]
struct IntlResp {
    letter: String,
    sex: i32,
    search: String,
    exclude: String,
    era_start: i32,
    era_end: i32,
    absent_fr: String,
    double_variant: bool,
    limit: i64,
    has_more: bool,
    results: Vec<IntlRow>,
}

async fn intl_search(
    State(s): State<AppState>,
    Query(q): Query<IntlQuery>,
) -> Result<Json<IntlResp>, ApiError> {
    let letter = q.letter.unwrap_or_default().to_uppercase();
    // prenoms_intl stores names already uppercased — use direct LIKE (no UPPER() wrapper)
    let letter_pattern = if letter.is_empty() {
        "%".to_string()
    } else {
        format!("%{}%", letter)
    };
    let sex = q.sex.filter(|&v| v == 1 || v == 2).unwrap_or(0);
    let search = q.search.unwrap_or_default().trim().to_uppercase();
    let exclude = q.exclude.unwrap_or_default().trim().to_uppercase();
    let era_start = q.era_start.unwrap_or(1985);
    let era_end = q.era_end.unwrap_or(2005);
    let absent_fr = q.absent_fr.unwrap_or_else(|| "any".to_string());
    let double_variant = q.double_variant.as_deref() == Some("1");
    let limit = q.limit.unwrap_or(30).min(500) as i64;

    let conn = s.pool.get()?;

    // Step 1: build distinct set of uppercased French names to use for anti-join.
    // prenoms_nat prenom values come from INSEE (already uppercase in the DBF).
    // We materialise into a HashSet to avoid N correlated subqueries.
    let french_names: std::collections::HashSet<String> = {
        if absent_fr.starts_with("year:") {
            if let Some(y) = absent_fr
                .strip_prefix("year:")
                .and_then(|s| s.parse::<i32>().ok())
            {
                let mut s2 = conn.prepare(
                    "SELECT DISTINCT UPPER(prenom) FROM prenoms_nat WHERE annee = ?1",
                )?;
                let x: std::collections::HashSet<String> = s2
                    .query_map(params![y], |row| row.get::<_, String>(0))?
                    .filter_map(|r| r.ok())
                    .collect();
                x
            } else {
                std::collections::HashSet::new()
            }
        } else {
            // All years
            let mut s2 = conn.prepare(
                "SELECT DISTINCT UPPER(prenom) FROM prenoms_nat",
            )?;
            let x: std::collections::HashSet<String> = s2
                .query_map([], |row| row.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            x
        }
    };

    // Step 2: build distinct set of known intl names for fast double-variant lookup
    // Only needed when double_variant is requested.
    let intl_name_set: std::collections::HashSet<String> = if double_variant {
        let mut s2 = conn.prepare("SELECT DISTINCT prenom FROM prenoms_intl")?;
        let x: std::collections::HashSet<String> = s2
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        x
    } else {
        std::collections::HashSet::new()
    };

    // Step 3: aggregate prenoms_intl — no anti-join in SQL, we do it in Rust.
    // Split totals by source so UI can show US vs UK counts separately.
    let sql =
        "SELECT prenom, sex,
                SUM(CASE WHEN source = 'US' THEN nombre ELSE 0 END) AS total_us,
                SUM(CASE WHEN source = 'UK' THEN nombre ELSE 0 END) AS total_uk,
                MIN(annee) AS first_y,
                MAX(annee) AS last_y,
                SUM(CASE WHEN annee BETWEEN ?3 AND ?4 THEN nombre ELSE 0 END) AS era_cnt
         FROM prenoms_intl
         WHERE prenom LIKE ?1
           AND (?2 = 0 OR sex = ?2)
           AND length(prenom) > 1
         GROUP BY prenom, sex
         HAVING era_cnt > 0
         ORDER BY (total_us + total_uk) ASC, prenom ASC";

    let mut stmt = conn.prepare(sql)?;
    let raw_rows = stmt
        .query_map(
            params![letter_pattern, sex, era_start, era_end],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    // Step 4: apply Rust-side filters (search, exclude, anti-FR, double-variant)
    // and paginate.
    let mut results: Vec<IntlRow> = Vec::new();
    let mut total_after_filter: usize = 0;

    for (prenom, sex_v, total_us, total_uk, first_y, last_y, era_cnt) in raw_rows {
        // search filter
        if !search.is_empty() && !prenom.contains(&search) {
            continue;
        }
        // exclude filter
        if !exclude.is_empty() && prenom.contains(&exclude) {
            continue;
        }
        // anti-join: skip if name appears in French INSEE records
        if !french_names.is_empty() && french_names.contains(&prenom) {
            continue;
        }

        // double-variant detection (in-memory, O(len) per name)
        let mut has_variant = false;
        let mut variant_example: Option<String> = None;
        if double_variant {
            let chars: Vec<char> = prenom.chars().collect();
            'outer: for k in 0..chars.len() {
                let next_same = k + 1 < chars.len() && chars[k] == chars[k + 1];
                if next_same {
                    continue;
                }
                let mut v = String::with_capacity(prenom.len() + 1);
                v.extend(chars[..=k].iter());
                v.push(chars[k]);
                v.extend(chars[k + 1..].iter());
                if intl_name_set.contains(&v) {
                    has_variant = true;
                    variant_example = Some(v);
                    break 'outer;
                }
            }
            if !has_variant {
                continue;
            }
        }

        // Build sources list from which corpora have data for this name
        let mut sources: Vec<String> = Vec::new();
        if total_us > 0 { sources.push("US".to_string()); }
        if total_uk > 0 { sources.push("UK".to_string()); }

        total_after_filter += 1;

        if results.len() < (limit + 1) as usize {
            results.push(IntlRow {
                rank: 0, // set below
                prenom,
                sex: sex_v,
                total_us,
                total_uk,
                sources,
                first_year: first_y,
                last_year: last_y,
                era_count: era_cnt,
                has_double_variant: has_variant,
                variant_example,
            });
        }
    }

    let has_more = total_after_filter > limit as usize;
    let truncated_len = results.len().min(limit as usize);
    results.truncate(truncated_len);
    for (i, r) in results.iter_mut().enumerate() {
        r.rank = i + 1;
    }

    Ok(Json(IntlResp {
        letter,
        sex,
        search,
        exclude,
        era_start,
        era_end,
        absent_fr,
        double_variant,
        limit,
        has_more,
        results,
    }))
}

// ---------- /intl-match ----------
//
// Multi-algo cross-language matching. Given the seed scenario:
//   "a rare French INSEE name (~25 occurrences) that's the French spelling
//    of an anglo name from a 90s English-language TV series, one letter
//    swapped, exactly one 'L'",
// this runs three independent matchers and merges their results.
//
// See `intl_match.rs` for algo details.

#[derive(Deserialize)]
pub struct IntlMatchQuery {
    pub letter: Option<String>,
    pub sex: Option<i32>,
    pub search: Option<String>,
    pub exclude: Option<String>,
    pub era_start: Option<i32>,
    pub era_end: Option<i32>,
    /// Total INSEE occurrences (all years) lower bound, default 5
    pub n_min: Option<i64>,
    /// Upper bound, default 100
    pub n_max: Option<i64>,
    /// "1" to require exactly one 'L' (case-insensitive)
    pub one_l: Option<String>,
    /// Max Levenshtein distance; default 2
    pub lev_max: Option<u32>,
    /// How many intl seeds to expand on (controls cost). Default 800.
    pub intl_seed_limit: Option<u32>,
    /// CSV subset of {phonetic, lev2, anglicisation}; default all three.
    pub algos: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct IntlMatchRow {
    rank: usize,
    prenom: String,
    matched_by: Vec<&'static str>,
    score: f64,
    lev_distance: Option<u32>,
    from_intl: Vec<String>,
    anglo_rules: Vec<String>,
    insee_total: i64,
    insee_years: i64,
}

#[derive(Serialize)]
struct IntlMatchResp {
    letter: String,
    sex: i32,
    n_min: i64,
    n_max: i64,
    one_l: bool,
    lev_max: u32,
    algos: Vec<String>,
    intl_seed_limit: u32,
    limit: i64,
    has_more: bool,
    results: Vec<IntlMatchRow>,
}

async fn intl_match(
    State(s): State<AppState>,
    Query(q): Query<IntlMatchQuery>,
) -> Result<Json<IntlMatchResp>, ApiError> {
    let letter = q.letter.unwrap_or_default().to_uppercase();
    let sex = q.sex.filter(|&v| v == 1 || v == 2).unwrap_or(0);
    let search = q.search.unwrap_or_default().trim().to_uppercase();
    let exclude = q.exclude.unwrap_or_default().trim().to_uppercase();
    let era_start = q.era_start.unwrap_or(1985);
    let era_end = q.era_end.unwrap_or(2005);
    let n_min = q.n_min.unwrap_or(5);
    let n_max = q.n_max.unwrap_or(100);
    let one_l = q.one_l.as_deref() == Some("1");
    let lev_max = q.lev_max.unwrap_or(2).min(3);
    let intl_seed_limit = q.intl_seed_limit.unwrap_or(800).min(5000) as usize;
    let limit = q.limit.unwrap_or(50).min(500) as i64;

    let algos_csv = q.algos.unwrap_or_else(|| "phonetic,lev2,anglicisation".to_string());
    let use_phonetic = algos_csv.contains("phonetic");
    let use_lev2 = algos_csv.contains("lev2");
    let use_anglicisation = algos_csv.contains("anglicisation");

    let params = crate::intl_match::MatchParams {
        letter: letter.clone(),
        sex,
        search,
        exclude,
        era_start,
        era_end,
        n_min,
        n_max,
        one_l,
        lev_max,
        intl_seed_limit,
        limit: (limit as usize) + 1, // peek for has_more
        use_phonetic,
        use_lev2,
        use_anglicisation,
    };

    // Run on a blocking thread — these algos are CPU-bound (Levenshtein
    // over ~30k INSEE names × hundreds of seeds).
    let conn = s.pool.get()?;
    let matches = tokio::task::spawn_blocking(move || {
        crate::intl_match::run(&conn, &params)
    })
    .await
    .map_err(|e| ApiError(anyhow::anyhow!("join error: {e}")))??;

    let has_more = matches.len() as i64 > limit;
    let results: Vec<IntlMatchRow> = matches
        .into_iter()
        .take(limit as usize)
        .enumerate()
        .map(|(i, m)| IntlMatchRow {
            rank: i + 1,
            prenom: m.prenom,
            matched_by: m.matched_by,
            score: (m.score * 100.0).round() / 100.0,
            lev_distance: m.lev_distance,
            from_intl: m.from_intl,
            anglo_rules: m.anglo_rules,
            insee_total: m.insee_total,
            insee_years: m.insee_years,
        })
        .collect();

    let mut algos_used = Vec::new();
    if use_phonetic { algos_used.push("phonetic".to_string()); }
    if use_lev2 { algos_used.push("lev2".to_string()); }
    if use_anglicisation { algos_used.push("anglicisation".to_string()); }

    Ok(Json(IntlMatchResp {
        letter,
        sex,
        n_min,
        n_max,
        one_l,
        lev_max,
        algos: algos_used,
        intl_seed_limit: intl_seed_limit as u32,
        limit,
        has_more,
        results,
    }))
}
