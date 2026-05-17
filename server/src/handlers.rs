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
    pub dept: String,
    pub letter: Option<String>,
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
    results: Vec<RarestRow>,
}

async fn rarest(
    State(s): State<AppState>,
    Query(q): Query<RarestQuery>,
) -> Result<Json<RarestResp>, ApiError> {
    let year = q.year.unwrap_or(2006);
    let letter = q.letter.unwrap_or_else(|| "L".into()).to_uppercase();
    let limit = q.limit.unwrap_or(20).min(200) as i64;
    let pattern = format!("%{}%", letter);

    let conn = s.pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT prenom, sexe, SUM(nombre) AS n
         FROM prenoms
         WHERE annee = ?1
           AND dept = ?2
           AND UPPER(prenom) LIKE ?3
           AND prenom NOT IN ('_PRENOMS_RARES', 'XXXX')
           AND length(prenom) > 1
         GROUP BY prenom, sexe
         ORDER BY n ASC, prenom ASC
         LIMIT ?4",
    )?;
    let rows = stmt
        .query_map(params![year, q.dept, pattern, limit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?, row.get::<_, i64>(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let results = rows
        .into_iter()
        .enumerate()
        .map(|(i, (prenom, sexe, n))| RarestRow { rank: i + 1, prenom, sexe, n })
        .collect();

    Ok(Json(RarestResp { year, dept: q.dept, letter, results }))
}

// ---------- /birth-context ----------

#[derive(Deserialize)]
pub struct ContextQuery {
    pub year: Option<i32>,
    pub dept: String,
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
    let month = q.month.unwrap_or(5);

    let conn = s.pool.get()?;

    let month_births: i64 = conn.query_row(
        "SELECT COALESCE(SUM(count), 0) FROM naissances WHERE dept_nais = ?1 AND mois = ?2",
        params![q.dept, month],
        |row| row.get(0),
    )?;

    let year_births: i64 = conn.query_row(
        "SELECT COALESCE(SUM(count), 0) FROM naissances WHERE dept_nais = ?1",
        params![q.dept],
        |row| row.get(0),
    )?;

    let share_pct = if year_births > 0 {
        (month_births as f64 * 100.0 / year_births as f64 * 10.0).round() / 10.0
    } else {
        0.0
    };

    Ok(Json(ContextResp {
        dept: q.dept,
        month,
        month_births,
        year_births,
        share_pct,
    }))
}
