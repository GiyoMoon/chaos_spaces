use std::collections::HashMap;

use astrolabe::{CronSchedule, DateTime};
use sqlx::SqlitePool;

type JobResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const DIRECTORY_URL: &str =
    "https://raw.githubusercontent.com/SpaceApi/directory/refs/heads/master/directory.json";

pub async fn run(db_pool: SqlitePool) {
    if let Err(e) = sync(&db_pool).await {
        tracing::error!("Directory sync failed: {e}");
    }

    let schedule = CronSchedule::parse("0 7 * * *").unwrap();
    for next in schedule {
        let duration = next.duration_between(&DateTime::now());
        tokio::time::sleep(duration).await;

        if let Err(e) = sync(&db_pool).await {
            tracing::error!("Directory sync failed: {e}");
        }
    }
}

async fn sync(db_pool: &SqlitePool) -> JobResult {
    tracing::info!("Syncing space directory");

    let directory: HashMap<String, String> = reqwest::get(DIRECTORY_URL).await?.json().await?;

    let existing = sqlx::query!("SELECT space_id, name, space_api_url, active FROM space")
        .fetch_all(db_pool)
        .await?;

    let existing_map: HashMap<String, _> = existing
        .into_iter()
        .map(|row| (row.name.clone(), row))
        .collect();

    for (name, url) in &directory {
        match existing_map.get(name) {
            Some(space) if &space.space_api_url != url => {
                sqlx::query!(
                    "UPDATE space SET space_api_url = ?, active = TRUE, updated_at = CURRENT_TIMESTAMP WHERE space_id = ?",
                    url,
                    space.space_id
                )
                .execute(db_pool)
                .await?;
            }
            Some(space) if !space.active => {
                sqlx::query!(
                    "UPDATE space SET active = TRUE, updated_at = CURRENT_TIMESTAMP WHERE space_id = ?",
                    space.space_id
                )
                .execute(db_pool)
                .await?;
            }
            None => {
                let space_id = ulid::Ulid::new().to_string();
                sqlx::query!(
                    "INSERT INTO space (space_id, name, space_api_url) VALUES (?, ?, ?)",
                    space_id,
                    name,
                    url
                )
                .execute(db_pool)
                .await?;
            }
            _ => {}
        }
    }

    for (name, space) in &existing_map {
        if !directory.contains_key(name) && space.active {
            sqlx::query!(
                "UPDATE space SET active = FALSE, updated_at = CURRENT_TIMESTAMP WHERE space_id = ?",
                space.space_id
            )
            .execute(db_pool)
            .await?;
        }
    }

    tracing::info!("Directory sync complete: {} spaces", directory.len());
    Ok(())
}
