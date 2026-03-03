use minijinja::{Environment, Value, path_loader};
use once_cell::sync::Lazy;

use crate::common::error::AppError;

pub mod assignments;
pub mod handlers;
pub mod htmx;
mod instructors;
pub mod routes;
pub mod view_models;

pub use routes::web_routes;

fn build_embedded_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template(
        "layouts/base.html",
        include_str!("../../templates/layouts/base.html"),
    )
    .expect("Failed to register layouts/base.html");
    env.add_template("index.html", include_str!("../../templates/index.html"))
        .expect("Failed to register index.html");
    env.add_template("login.html", include_str!("../../templates/login.html"))
        .expect("Failed to register login.html");
    env.add_template(
        "admin/users_new.html",
        include_str!("../../templates/admin/users_new.html"),
    )
    .expect("Failed to register admin/users_new.html");
    env.add_template(
        "assignments/index.html",
        include_str!("../../templates/assignments/index.html"),
    )
    .expect("Failed to register assignments/index.html");
    env.add_template(
        "assignments/detail.html",
        include_str!("../../templates/assignments/detail.html"),
    )
    .expect("Failed to register assignments/detail.html");
    env.add_template(
        "assignments/form.html",
        include_str!("../../templates/assignments/form.html"),
    )
    .expect("Failed to register assignments/form.html");
    env.add_template(
        "assignments/_row.html",
        include_str!("../../templates/assignments/_row.html"),
    )
    .expect("Failed to register assignments/_row.html");
    env.add_template(
        "assignments/_status_badge.html",
        include_str!("../../templates/assignments/_status_badge.html"),
    )
    .expect("Failed to register assignments/_status_badge.html");
    env.add_template(
        "partials/empty_state.html",
        include_str!("../../templates/partials/empty_state.html"),
    )
    .expect("Failed to register partials/empty_state.html");
    env.add_template(
        "instructors/index.html",
        include_str!("../../templates/instructors/index.html"),
    )
    .expect("Failed to register instructors/index.html");
    env.add_template(
        "classes/class_detail.html",
        include_str!("../../templates/classes/class_detail.html"),
    )
    .expect("Failed to register classes/class_detail.html");
    env.add_template(
        "classes/create_class.html",
        include_str!("../../templates/classes/create_class.html"),
    )
    .expect("Failed to register classes/create_class.html");
    env
}

fn build_dev_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    env
}

static WEB_TEMPLATES: Lazy<Environment<'static>> = Lazy::new(build_embedded_environment);

pub fn render_template(name: &str, context: Value) -> Result<String, AppError> {
    let dev_env;
    let env = if cfg!(debug_assertions) {
        dev_env = build_dev_environment();
        &dev_env
    } else {
        &WEB_TEMPLATES
    };

    let template = env.get_template(name).map_err(|err| {
        tracing::error!(template = %name, error = %err, "failed to load template");
        AppError::InternalError
    })?;

    template.render(context).map_err(|err| {
        tracing::error!(template = %name, error = %err, "failed to render template");
        AppError::InternalError
    })
}
