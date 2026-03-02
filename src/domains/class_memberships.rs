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
    pub mod class_membership_dto;
}

mod infra {
    pub(crate) mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::{ClassMembershipsApiDoc, class_membership_routes};
pub use domain::model::{ClassMembership, ClassMembershipRole};
pub use domain::repository::ClassMembershipRepositoryTrait;
pub use domain::service::ClassMembershipServiceTrait;
pub use infra::impl_service::ClassMembershipService;

use sqlx::PgPool;
use std::sync::Arc;

pub fn create_class_membership_service(pool: PgPool) -> Arc<dyn ClassMembershipServiceTrait> {
    ClassMembershipService::<infra::impl_repository::ClassMembershipRepository>::create_service(
        pool,
    )
}
