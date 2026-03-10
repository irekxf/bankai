use std::collections::HashMap;

use sqlx::{FromRow, SqlitePool};

use crate::{error::AppError, tools::builtin_tools};

#[derive(Debug, Clone, FromRow)]
struct ToolSettingRecord {
    name: String,
    enabled: i64,
}

pub async fn seed_tool_settings(pool: &SqlitePool) -> Result<(), AppError> {
    for tool in builtin_tools() {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO tool_settings (name, enabled)
            VALUES (?1, ?2)
            "#,
        )
        .bind(tool.name)
        .bind(if tool.enabled_by_default { 1_i64 } else { 0_i64 })
        .execute(pool)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;
    }

    Ok(())
}

pub async fn list_tool_settings(pool: &SqlitePool) -> Result<HashMap<String, bool>, AppError> {
    let records = sqlx::query_as::<_, ToolSettingRecord>(
        r#"
        SELECT name, enabled
        FROM tool_settings
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(records
        .into_iter()
        .map(|record| (record.name, record.enabled != 0))
        .collect())
}

pub async fn list_enabled_tool_names(pool: &SqlitePool) -> Result<Vec<String>, AppError> {
    let enabled_map = list_tool_settings(pool).await?;

    Ok(builtin_tools()
        .iter()
        .filter(|tool| enabled_map.get(tool.name).copied().unwrap_or(tool.enabled_by_default))
        .map(|tool| tool.name.to_string())
        .collect())
}

pub async fn set_tool_enabled(
    pool: &SqlitePool,
    tool_name: &str,
    enabled: bool,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE tool_settings
        SET enabled = ?2
        WHERE name = ?1
        "#,
    )
    .bind(tool_name)
    .bind(if enabled { 1_i64 } else { 0_i64 })
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}
