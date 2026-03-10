use sqlx::SqlitePool;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::agent::approval::ApprovalState;

pub struct AppState {
    pub approval: Arc<Mutex<ApprovalState>>,
    pub db: SqlitePool,
    pub workspace_root: PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool, workspace_root: PathBuf, approval: ApprovalState) -> Self {
        Self {
            approval: Arc::new(Mutex::new(approval)),
            db,
            workspace_root,
        }
    }
}
