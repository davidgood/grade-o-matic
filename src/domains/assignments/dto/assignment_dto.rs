use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentSummaryDto {
    pub id: String,
    pub title: String,
    pub due_at: Option<String>,
}
