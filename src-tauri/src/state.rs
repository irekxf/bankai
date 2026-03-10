use std::sync::Arc;
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::agent::approval::ApprovalState;

pub struct AppState {
    pub approval: Arc<Mutex<ApprovalState>>,
    pub db: SqlitePool,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            approval: Arc::new(Mutex::new(ApprovalState::default())),
            db,
        }
    }
}
