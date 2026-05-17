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
        .route("/birth-context", get(birth_context))
        .route("/births", get(births))
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
    let limit = q.limit.unwrap_or(50).min(500) as i64;
    let offset = q.offset.unwrap_or(0) as i64;

    let conn = s.pool.get()?;

    let where_clause = "
         WHERE (?1 = '' OR dept_nais = ?1)
           AND (?2 = 0 OR mois = ?2)
           AND (?3 = 0 OR sexe = ?3)
           AND (age_mere IS NULL OR age_mere BETWEEN ?4 AND ?5)
           AND (age_pere IS NULL OR age_pere BETWEEN ?6 AND ?7)";

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
