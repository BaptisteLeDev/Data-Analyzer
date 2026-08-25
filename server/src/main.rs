use anyhow::{Context, Result};
use axum::extract::{Request, State};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod dashboard;
mod errors;
mod handlers;
mod intl_match;
mod metrics;
mod phonetic;
mod state;

use state::AppState;

fn resolve(rel: &str) -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let candidates = [
        exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.join(rel)),
        Some(PathBuf::from(rel)),
        Some(PathBuf::from(format!("../{rel}"))),
    ];
    for c in candidates.into_iter().flatten() {
        if c.exists() {
            return Ok(c);
        }
    }
    anyhow::bail!("could not locate {rel} (run prep first, or `bun run build` in web/)")
}

async fn track_metrics(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let response = next.run(req).await;
    let duration_ms = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();
    state.metrics.record(method, path, status, duration_ms);
    response
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let db = resolve("data/analyzer.sqlite").context("SQLite database not found")?;
    tracing::info!("using db: {}", db.display());
    let app_state = AppState::new(db.to_str().unwrap())?;

    // Materialize phonetic codes (idempotent: skipped if tables already populated)
    {
        let mut conn = app_state.pool.get()?;
        phonetic::ensure_materialized(&mut conn)?;
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = handlers::router()
        .merge(dashboard::router())
        .route_layer(middleware::from_fn_with_state(app_state.clone(), track_metrics));

    let mut app = Router::new()
        .nest("/api", api)
        .with_state(app_state);

    match resolve("web/dist") {
        Ok(dist) => {
            tracing::info!("serving static files from {}", dist.display());
            app = app.fallback_service(ServeDir::new(dist));
        }
        Err(_) => {
            tracing::warn!("web/dist not found; only /api routes are served. Run `bun run build` in web/.");
        }
    }

    let app = app.layer(cors).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
