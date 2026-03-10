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
    pub tool_call_id: Option<String>,
    pub tool_message_kind: Option<String>,
    pub tool_name: Option<String>,
    pub tool_status: Option<String>,
    pub tool_arguments_json: Option<String>,
    pub tool_result_text: Option<String>,
    pub tool_rejection_reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct MessageMetadata {
    pub tool_call_id: Option<String>,
    pub tool_message_kind: Option<String>,
}

pub async fn create_message(
    pool: &SqlitePool,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<MessageRecord, AppError> {
    create_message_with_metadata(pool, session_id, role, content, MessageMetadata::default()).await
}

pub async fn create_message_with_metadata(
    pool: &SqlitePool,
    session_id: &str,
    role: &str,
    content: &str,
    metadata: MessageMetadata,
) -> Result<MessageRecord, AppError> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO messages (id, session_id, role, content, tool_call_id, tool_message_kind)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(&id)
    .bind(session_id)
    .bind(role)
    .bind(content)
    .bind(&metadata.tool_call_id)
    .bind(&metadata.tool_message_kind)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    fetch_message(pool, &id).await
}

pub async fn list_messages(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<MessageRecord>, AppError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            messages.id AS id,
            messages.session_id AS session_id,
            messages.role AS role,
            messages.content AS content,
            messages.tool_call_id AS tool_call_id,
            messages.tool_message_kind AS tool_message_kind,
            tool_calls.tool_name AS tool_name,
            tool_calls.status AS tool_status,
            tool_calls.arguments_json AS tool_arguments_json,
            tool_calls.result_text AS tool_result_text,
            tool_calls.rejection_reason AS tool_rejection_reason,
            messages.created_at AS created_at
        FROM messages
        LEFT JOIN tool_calls ON tool_calls.id = messages.tool_call_id
        WHERE messages.session_id = ?1
        ORDER BY messages.created_at ASC, messages.id ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))
}

async fn fetch_message(pool: &SqlitePool, id: &str) -> Result<MessageRecord, AppError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            messages.id AS id,
            messages.session_id AS session_id,
            messages.role AS role,
            messages.content AS content,
            messages.tool_call_id AS tool_call_id,
            messages.tool_message_kind AS tool_message_kind,
            tool_calls.tool_name AS tool_name,
            tool_calls.status AS tool_status,
            tool_calls.arguments_json AS tool_arguments_json,
            tool_calls.result_text AS tool_result_text,
            tool_calls.rejection_reason AS tool_rejection_reason,
            messages.created_at AS created_at
        FROM messages
        LEFT JOIN tool_calls ON tool_calls.id = messages.tool_call_id
        WHERE messages.id = ?1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))
}
