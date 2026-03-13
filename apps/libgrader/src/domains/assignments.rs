use std::sync::Arc;

use sqlx::PgPool;

mod api {
    mod handlers;
    pub mod routes;
}

mod domain {
    pub mod model;
    pub mod repository;
    pub mod service;
}

pub mod dto {
    pub mod assignment_dto;
}

mod infra {
    pub(crate) mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::assignment_routes;
pub use domain::model::{
    Assignment, AssignmentAttachment, AssignmentWithAttachmentCount, StudentAssignmentSubmission,
};
pub use domain::repository::AssignmentRepositoryTrait;
pub use domain::service::AssignmentServiceTrait;
pub use infra::impl_service::AssignmentService;

pub fn create_assignment_service(pool: PgPool) -> Arc<dyn AssignmentServiceTrait> {
    AssignmentService::<infra::impl_repository::AssignmentRepository>::create_service(pool)
}
pub use api::routes::AssignmentsApiDoc;
