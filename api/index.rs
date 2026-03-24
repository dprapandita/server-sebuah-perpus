use vercel_runtime::{run, Error};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Panggil dari library kamu.
// Ganti "sewadah_perpus" dengan "name" yang ada di file Cargo.toml kamu
use sebuah_perpus::routes::routes;
use sebuah_perpus::app;

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    info!("[Vercel] Menjalankan inisialisasi fungsi serverless...");

    let state = app::AppState::New().await;

    let router = routes(state);

    info!("[Vercel] Router siap menerima request!");
    run(router).await
}