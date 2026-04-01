pub mod config;
pub mod core;
pub mod http;
pub mod jobs;

use clap::Parser;
use config::{Commands, Config};
use http::generate_openapi;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chaos_spaces=debug".into()),
        )
        .init();

    let config = Config::parse();

    if let Some(Commands::GenerateOpenapi) = config.command {
        let schema = generate_openapi();
        std::fs::write("openapi.json", schema).expect("Failed writing openapi.json");
        std::process::exit(0);
    }

    let db_pool = SqlitePool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed running sqlx migrations");

    tokio::spawn(jobs::sync_directory::run(db_pool.clone()));
    tokio::spawn(jobs::fetch_spaces::run(db_pool.clone()));

    http::serve(config, db_pool).await;
}
