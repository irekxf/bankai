use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingApproval {
    pub id: String,
    pub session_id: String,
    pub response_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: String,
    pub arguments_json: String,
}

#[derive(Debug, Default)]
pub struct ApprovalState {
    pub pending: Vec<PendingApproval>,
}

impl ApprovalState {
    pub fn new(pending: Vec<PendingApproval>) -> Self {
        Self { pending }
    }
}
