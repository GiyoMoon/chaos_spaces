mod overview;

pub use overview::__path_overview;

use axum::{Router, routing::get};
use utoipa::OpenApi;

pub fn router() -> Router {
    Router::new().route("/overview", get(overview::overview))
}

#[derive(OpenApi)]
#[openapi(
    paths(overview),
    tags((name = "Stats", description = "Stats control endpoints")),
)]
pub struct StatsApiDoc;

pub const OPENAPI_TAG: &str = "Stats";
