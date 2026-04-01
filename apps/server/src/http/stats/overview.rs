use astrolabe::DateTime;
use axum::{Extension, Json};
use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::core::structs::AppResult;

#[utoipa::path(
    get,
    path = "/api/stats/overview",
    responses(
        (status = 200, description = "List of spaces with their current status", body = [SpaceOverview])
    ),
    tag = super::OPENAPI_TAG
)]
pub async fn overview(
    Extension(db_pool): Extension<SqlitePool>,
) -> AppResult<Json<Vec<SpaceOverview>>> {
    let spaces = sqlx::query!(
        r#"
        SELECT
            s.name,
            sl.open as "open: bool",
            sl.created_at as "last_fetched: DateTime"
        FROM space s
        LEFT JOIN space_log sl ON sl.space_log_id = (
            SELECT space_log_id FROM space_log
            WHERE space_id = s.space_id
            ORDER BY created_at DESC
            LIMIT 1
        )
        WHERE s.active = TRUE
        "#
    )
    .fetch_all(&db_pool)
    .await?;

    let result = spaces
        .into_iter()
        .map(|row| SpaceOverview {
            name: row.name,
            open: row.open,
            last_fetched: row.last_fetched,
        })
        .collect();

    Ok(Json(result))
}

#[derive(Serialize, ToSchema)]
pub struct SpaceOverview {
    name: String,
    open: Option<bool>,
    #[schema(value_type = String)]
    last_fetched: Option<DateTime>,
}
