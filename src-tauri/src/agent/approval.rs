use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: String,
    pub tool_name: String,
    pub arguments_json: String,
}

#[derive(Debug, Default)]
pub struct ApprovalState {
    pub pending: Vec<PendingApproval>,
}
