use crate::routes::{handle_error, routes};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use std::env;
use std::time::Duration;
use tokio::signal;
use tracing::log::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod respons;
mod routes;
mod utils;
mod core;
mod config;

#[tokio::main]
async fn main() {
    if dotenvy::var("HOST_MODE").unwrap() != "production" {
        unsafe { env::set_var("RUST_LOG", "debug") }
    }
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = app::state::AppState::new().await;

    Migrator::up(&state.database_connection, None).await
        .expect("Gagal menjalankan migrasi database");
    println!("✅ Migrasi database berhasil dijalankan.");

    let server_url = format!("{}:{}", &state.env.host, &state.env.port);

    let router = routes(state).merge(handle_error());
    info!("Server was run by {:?}", server_url);
    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Error running server");
    info!("Server stopped");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
