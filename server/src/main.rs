use anyhow::{Context, Result};
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod errors;
mod handlers;
mod state;

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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let db = resolve("data/analyzer.sqlite").context("SQLite database not found")?;
    tracing::info!("using db: {}", db.display());
    let app_state = state::AppState::new(db.to_str().unwrap())?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut app = Router::new()
        .nest("/api", handlers::router())
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
