use std::collections::HashMap;

use sqlx::SqlitePool;

use crate::error::AppError;

pub async fn list_tool_enabled_map(
    pool: &SqlitePool,
) -> Result<HashMap<String, bool>, AppError> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT name, enabled FROM tool_settings",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(name, enabled)| (name, enabled != 0))
        .collect())
}

pub async fn set_tool_enabled(
    pool: &SqlitePool,
    name: &str,
    enabled: bool,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO tool_settings (name, enabled)
        VALUES (?1, ?2)
        ON CONFLICT(name) DO UPDATE SET enabled = ?2
        "#,
    )
    .bind(name)
    .bind(enabled as i64)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}
