use utoipa::OpenApi;

use crate::core::error::ValidationError;

use super::stats::StatsApiDoc;

#[derive(OpenApi)]
#[openapi(info(title = "chaos_spaces API"), components(schemas(ValidationError)))]
pub struct ApiDoc;

pub fn generate_openapi() -> String {
    let open_api = ApiDoc::openapi().nest("/api/stats", StatsApiDoc::openapi());
    serde_json::to_string_pretty(&open_api).unwrap()
}
