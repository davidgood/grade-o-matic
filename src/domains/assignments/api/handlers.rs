use axum::Json;

use crate::domains::assignments::dto::assignment_dto::AssignmentDto;

pub async fn list_assignments() -> Json<Vec<AssignmentDto>> {
    Json(Vec::new())
}
