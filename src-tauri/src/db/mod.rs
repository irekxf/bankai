pub mod messages;
pub mod sessions;
pub mod tools;
pub mod tool_calls;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    ConnectOptions, Row, SqlitePool,
};
use tauri::{AppHandle, Manager};

use crate::error::AppError;

pub async fn init(app: &AppHandle) -> Result<SqlitePool, AppError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Message(error.to_string()))?;
    std::fs::create_dir_all(&app_data_dir).map_err(|error| AppError::Message(error.to_string()))?;

    let database_path = app_data_dir.join("bankai.sqlite3");
    let connect_options = SqliteConnectOptions::new()
        .filename(&database_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
            content TEXT NOT NULL,
            tool_call_id TEXT,
            tool_message_kind TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tool_calls (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            response_id TEXT,
            tool_call_id TEXT,
            tool_name TEXT NOT NULL,
            arguments_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'rejected', 'completed')),
            result_text TEXT,
            rejection_reason TEXT,
            request_message_id TEXT,
            resolution_message_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    ensure_column(&pool, "messages", "tool_call_id", "TEXT").await?;
    ensure_column(&pool, "messages", "tool_message_kind", "TEXT").await?;
    ensure_column(&pool, "tool_calls", "result_text", "TEXT").await?;
    ensure_column(&pool, "tool_calls", "rejection_reason", "TEXT").await?;
    ensure_column(&pool, "tool_calls", "request_message_id", "TEXT").await?;
    ensure_column(&pool, "tool_calls", "resolution_message_id", "TEXT").await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tool_settings (
            name TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL CHECK (enabled IN (0, 1))
        );
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|error| AppError::Message(error.to_string()))?;

    tools::seed_tool_settings(&pool).await?;

    Ok(pool)
}

async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), AppError> {
    let query = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    let exists = rows.iter().any(|row| {
        row.try_get::<String, _>("name")
            .map(|name| name == column)
            .unwrap_or(false)
    });

    if exists {
        return Ok(());
    }

    let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
    sqlx::query(&alter)
        .execute(pool)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(())
}
