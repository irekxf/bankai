use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::approval::ApprovalState;

#[derive(Default)]
pub struct AppState {
    pub approval: Arc<Mutex<ApprovalState>>,
}
