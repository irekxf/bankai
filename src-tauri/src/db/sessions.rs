use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
}

pub async fn ensure_session(
    pool: &SqlitePool,
    session_id: &str,
    title: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO sessions (id, title)
        VALUES (?1, ?2)
        ON CONFLICT(id) DO NOTHING
        "#,
    )
    .bind(session_id)
    .bind(title)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}

pub async fn create_session(pool: &SqlitePool, title: &str) -> Result<SessionSummary, AppError> {
    let id = Uuid::new_v4().to_string();
    ensure_session(pool, &id, title).await?;

    get_session(pool, &id)
        .await?
        .ok_or_else(|| AppError::Message("Created session was not found".to_string()))
}

pub async fn list_sessions(pool: &SqlitePool) -> Result<Vec<SessionSummary>, AppError> {
    sqlx::query_as::<_, SessionSummary>(
        r#"
        SELECT id, COALESCE(title, 'Untitled session') AS title, updated_at
        FROM sessions
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))
}

pub async fn get_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Option<SessionSummary>, AppError> {
    sqlx::query_as::<_, SessionSummary>(
        r#"
        SELECT id, COALESCE(title, 'Untitled session') AS title, updated_at
        FROM sessions
        WHERE id = ?1
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))
}

pub async fn touch_session(
    pool: &SqlitePool,
    session_id: &str,
    title: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET title = COALESCE(?2, title),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        "#,
    )
    .bind(session_id)
    .bind(title)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}
