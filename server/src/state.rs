use crate::metrics::Metrics;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub metrics: Metrics,
    pub db_path: String,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder().max_size(8).build(manager)?;
        Ok(Self {
            pool,
            metrics: Metrics::new(10),
            db_path: db_path.to_string(),
        })
    }
}
