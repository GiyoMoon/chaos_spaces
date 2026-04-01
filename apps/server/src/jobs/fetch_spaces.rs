use std::time::Duration;

use astrolabe::{CronSchedule, DateTime};
use reqwest::Client;
use sqlx::SqlitePool;

type JobResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(db_pool: SqlitePool) {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    let schedule = CronSchedule::parse("*/10 * * * *").unwrap();
    for next in schedule {
        let duration = next.duration_between(&DateTime::now());
        tokio::time::sleep(duration).await;

        if let Err(e) = fetch_all(&db_pool, &client).await {
            tracing::error!("Space fetch failed: {e}");
        }
    }
}

async fn fetch_all(db_pool: &SqlitePool, client: &Client) -> JobResult {
    let spaces = sqlx::query!("SELECT space_id, space_api_url FROM space WHERE active = TRUE")
        .fetch_all(db_pool)
        .await?;

    tracing::info!("Fetching {} active spaces", spaces.len());

    let mut join_set = tokio::task::JoinSet::new();

    for space in spaces {
        let db_pool = db_pool.clone();
        let client = client.clone();
        join_set.spawn(async move {
            if let Err(e) =
                fetch_space(&db_pool, &client, &space.space_id, &space.space_api_url).await
            {
                tracing::warn!("[space_fetch] [{}] {e}", space.space_api_url);
            }
        });
    }

    while join_set.join_next().await.is_some() {}

    tracing::info!("Finished fetching spaces");

    Ok(())
}

async fn fetch_space(
    db_pool: &SqlitePool,
    client: &Client,
    space_id: &str,
    url: &str,
) -> JobResult {
    let raw = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let json: serde_json::Value = serde_json::from_str(&raw)?;
    let open: Option<bool> = json
        .get("state")
        .and_then(|v| v.get("open"))
        .and_then(|v| v.as_bool());
    let response = serde_json::to_string(&json)?;

    let log_id = ulid::Ulid::new().to_string();
    sqlx::query!(
        "INSERT INTO space_log (space_log_id, space_id, open, response) VALUES (?, ?, ?, ?)",
        log_id,
        space_id,
        open,
        response
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
