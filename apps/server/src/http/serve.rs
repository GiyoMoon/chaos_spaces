use axum::{Extension, Router};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use super::stats;
use crate::config::Config;

pub async fn serve(config: Config, db_pool: SqlitePool) {
    let bind_address = &config.bind_address.clone();

    let app = router()
        .layer(Extension(config))
        .layer(Extension(db_pool))
        .layer(
            TraceLayer::new_for_http()
                .on_request(())
                .on_response(())
                .on_body_chunk(())
                .on_eos(()),
        );

    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .expect("Failed binding to address");

    tracing::info!("Listening on {}", bind_address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed starting application");
}

fn router() -> Router {
    let api_routes = Router::new().nest("/stats", stats::router());
    Router::new().nest("/api", api_routes)
}
