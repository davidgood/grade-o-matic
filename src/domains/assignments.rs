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
    mod impl_repository;
    pub mod impl_service;
}

pub use api::routes::assignment_routes;
pub use domain::service::AssignmentServiceTrait;
pub use infra::impl_service::AssignmentService;
