mod api {
    pub mod handlers;
    pub mod routes;
}

mod domain {
    pub mod model;
    pub mod repository;
    pub mod service;
}

pub mod dto {
    pub mod class_dto;
}

mod infra {
    pub(crate) mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::class_routes;
pub use domain::model::Class;
pub use domain::repository::ClassRepositoryTrait;
pub use domain::service::ClassServiceTrait;
pub use infra::impl_service::ClassService;
use sqlx::PgPool;
use std::sync::Arc;

pub fn create_class_service(pool: PgPool) -> Arc<dyn ClassServiceTrait> {
    ClassService::<infra::impl_repository::ClassRepository>::create_class_service(pool)
}
pub use api::routes::ClassesApiDoc;
