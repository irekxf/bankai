use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

pub async fn create_message(
    pool: &SqlitePool,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<MessageRecord, AppError> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO messages (id, session_id, role, content)
        VALUES (?1, ?2, ?3, ?4)
        "#,
    )
    .bind(&id)
    .bind(session_id)
    .bind(role)
    .bind(content)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT id, session_id, role, content, created_at
        FROM messages
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))
}

pub async fn list_messages(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<MessageRecord>, AppError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT id, session_id, role, content, created_at
        FROM messages
        WHERE session_id = ?1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))
}
