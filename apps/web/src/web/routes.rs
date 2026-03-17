use axum::{
    Router,
    extract::FromRef,
    middleware,
    routing::{get, post},
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use std::sync::Arc;

use crate::common::jwt;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::auth::AuthServiceTrait;
use crate::domains::class_memberships::ClassMembershipServiceTrait;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::file::FileServiceTrait;

use super::{
    assignments::assignments_page,
    handlers::{
        admin_create_user_submit, admin_users_page, login_page, login_submit, logout,
        server_time_fragment, ui_index, ui_instructors_root_redirect,
    },
    htmx::assignments::assignments_table_fragment,
    instructors::{
        add_student_to_roster, create_class_page, create_class_submit,
        delete_student_assignment_extension, edit_assignment_page, edit_assignment_submit,
        edit_class_page, edit_class_submit, instructor_class_detail_page,
        instructor_student_submission_history_page, instructors_page, remove_student_from_roster,
        upload_assignment_attachments, upsert_student_assignment_extension,
    },
    students::{
        student_assignment_detail_page, students_assignments_page, students_classes_page,
        submit_student_assignment,
    },
};
use crate::domains::user::{UserAssetPattern, UserServiceTrait};

pub fn web_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<dyn AuthServiceTrait>: FromRef<S>,
    Arc<dyn AssignmentServiceTrait>: FromRef<S>,
    Arc<dyn ClassMembershipServiceTrait>: FromRef<S>,
    Arc<dyn ClassServiceTrait>: FromRef<S>,
    Arc<dyn FileServiceTrait>: FromRef<S>,
    Arc<dyn UserServiceTrait>: FromRef<S>,
    UserAssetPattern: FromRef<S>,
{
    let shared_ui_routes = Router::new()
        .route("/", get(ui_index))
        .route("/ui", get(ui_index))
        .route("/ui/fragments/server-time", get(server_time_fragment))
        .route("/ui/logout", get(logout))
        .layer(middleware::from_fn(jwt::require_ui_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    let instructor_ui_routes = Router::new()
        .route("/ui/assignments", get(assignments_page))
        .route(
            "/ui/fragments/assignments/table",
            get(assignments_table_fragment),
        )
        .route(
            "/ui/instructors/fragments/assignments/table",
            get(assignments_table_fragment),
        )
        .route("/ui/instructors", get(ui_instructors_root_redirect))
        .route("/ui/instructors/classes", get(instructors_page))
        .route("/ui/instructors/assignments", get(assignments_page))
        .route("/ui/instructors/classes/new", get(create_class_page))
        .route("/ui/instructors/classes/new", post(create_class_submit))
        .route("/ui/instructors/classes/{id}/edit", get(edit_class_page))
        .route("/ui/instructors/classes/{id}/edit", post(edit_class_submit))
        .route(
            "/ui/instructors/classes/{id}",
            get(instructor_class_detail_page),
        )
        .route(
            "/ui/instructors/classes/{id}/roster",
            post(add_student_to_roster),
        )
        .route(
            "/ui/instructors/classes/{id}/roster/{membership_id}/delete",
            post(remove_student_from_roster),
        )
        .route(
            "/ui/instructors/assignments/{id}/edit",
            get(edit_assignment_page),
        )
        .route(
            "/ui/instructors/assignments/{id}/edit",
            post(edit_assignment_submit),
        )
        .route(
            "/ui/instructors/assignments/{id}/attachments",
            post(upload_assignment_attachments),
        )
        .route(
            "/ui/instructors/assignments/{assignment_id}/students/{student_id}/extension",
            post(upsert_student_assignment_extension),
        )
        .route(
            "/ui/instructors/assignments/{assignment_id}/students/{student_id}/extension/delete",
            post(delete_student_assignment_extension),
        )
        .route(
            "/ui/instructors/assignments/{assignment_id}/students/{student_id}/submissions",
            get(instructor_student_submission_history_page),
        )
        .layer(middleware::from_fn(jwt::require_instructor_ui_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    let student_ui_routes = Router::new()
        .route("/ui/students/classes", get(students_classes_page))
        .route("/ui/students/assignments", get(students_assignments_page))
        .route(
            "/ui/students/assignments/{id}",
            get(student_assignment_detail_page),
        )
        .route(
            "/ui/students/assignments/{id}/submit",
            post(submit_student_assignment),
        )
        .layer(middleware::from_fn(jwt::require_student_ui_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    let admin_ui_routes = Router::new()
        .route("/ui/admin/users", get(admin_users_page))
        .route("/ui/admin/users", post(admin_create_user_submit))
        .layer(middleware::from_fn(jwt::require_admin_access))
        .layer(middleware::from_fn(jwt::jwt_auth_web));

    Router::new()
        .route("/ui/login", get(login_page))
        .route("/ui/login", post(login_submit))
        .merge(shared_ui_routes)
        .merge(instructor_ui_routes)
        .merge(student_ui_routes)
        .merge(admin_ui_routes)
        .layer(CsrfLayer::new(CsrfConfig::default()))
}
