use std::sync::Arc;

use backend_rust::{
    config::Config, create_app, db::Database, init_tracing, run_migrations, state::AppState,
};
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = Config::from_env();
    let pool = Database::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database_user,
        config.database_password,
        config.database_host,
        config.database_port,
        config.database_name,
    ))
    .await;
    run_migrations(&pool.pool).await?;
    let app_state = Arc::new(AppState::new(pool, config.clone()));
    let app = create_app(app_state.clone());

    let listener_address = format!("0.0.0.0:{}", config.backend_port);
    tracing::info!("listening on {}", listener_address);

    let listener = tokio::net::TcpListener::bind(listener_address).await?;

    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal(app_state));

    if let Err(e) = server.await {
        tracing::error!(error = %e, "server error");
    }

    Ok(())
}

async fn shutdown_signal(_state: Arc<AppState>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("received CTRL+C, shutting down...");
        }
        _ = terminate => {
            tracing::info!("received terminate signal, shutting down...");
        }
    };
}
