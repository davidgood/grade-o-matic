use axum::Json;

use crate::domains::assignments::dto::assignment_dto::AssignmentSummaryDto;

pub async fn list_assignments() -> Json<Vec<AssignmentSummaryDto>> {
    Json(Vec::new())
}
