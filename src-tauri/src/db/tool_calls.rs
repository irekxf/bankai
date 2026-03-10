use sqlx::{FromRow, SqlitePool};

use crate::{agent::approval::PendingApproval, error::AppError};

#[derive(Debug, Clone, FromRow)]
pub struct ToolCallRecord {
    pub id: String,
    pub session_id: String,
    pub response_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: String,
    pub arguments_json: String,
}

pub async fn create_pending_tool_call(
    pool: &SqlitePool,
    pending: &PendingApproval,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO tool_calls (
            id,
            session_id,
            response_id,
            tool_call_id,
            tool_name,
            arguments_json,
            status
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')
        "#,
    )
    .bind(&pending.id)
    .bind(&pending.session_id)
    .bind(&pending.response_id)
    .bind(&pending.tool_call_id)
    .bind(&pending.tool_name)
    .bind(&pending.arguments_json)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}

pub async fn list_pending_tool_calls(pool: &SqlitePool) -> Result<Vec<PendingApproval>, AppError> {
    let records = sqlx::query_as::<_, ToolCallRecord>(
        r#"
        SELECT
            id,
            session_id,
            response_id,
            tool_call_id,
            tool_name,
            arguments_json
        FROM tool_calls
        WHERE status = 'pending'
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(records
        .into_iter()
        .map(|record| PendingApproval {
            id: record.id,
            session_id: record.session_id,
            response_id: record.response_id,
            tool_call_id: record.tool_call_id,
            tool_name: record.tool_name,
            arguments_json: record.arguments_json,
        })
        .collect())
}

pub async fn mark_tool_call_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE tool_calls
        SET status = ?2
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}

pub async fn attach_request_message(
    pool: &SqlitePool,
    id: &str,
    request_message_id: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE tool_calls
        SET request_message_id = ?2
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .bind(request_message_id)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}

pub async fn complete_tool_call(
    pool: &SqlitePool,
    id: &str,
    result_text: &str,
    resolution_message_id: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE tool_calls
        SET status = 'completed',
            result_text = ?2,
            rejection_reason = NULL,
            resolution_message_id = ?3
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .bind(result_text)
    .bind(resolution_message_id)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}

pub async fn reject_tool_call(
    pool: &SqlitePool,
    id: &str,
    rejection_reason: &str,
    resolution_message_id: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE tool_calls
        SET status = 'rejected',
            rejection_reason = ?2,
            result_text = NULL,
            resolution_message_id = ?3
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .bind(rejection_reason)
    .bind(resolution_message_id)
    .execute(pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}
