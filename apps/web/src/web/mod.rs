use chrono::{DateTime, Utc};
use minijinja::{Environment, Value, path_loader};
use once_cell::sync::Lazy;

use crate::common::error::AppError;

pub mod assignments;
pub mod handlers;
pub mod htmx;
mod instructors;
pub mod routes;
pub mod students;
pub mod view_models;

pub use routes::web_routes;

const TEMPLATE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");

macro_rules! embedded_template {
    ($relative:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/",
            $relative
        ))
    };
}

fn build_embedded_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("layouts/base.html", embedded_template!("layouts/base.html"))
        .expect("Failed to register layouts/base.html");
    env.add_template("index.html", embedded_template!("index.html"))
        .expect("Failed to register index.html");
    env.add_template("login.html", embedded_template!("login.html"))
        .expect("Failed to register login.html");
    env.add_template(
        "admin/users_new.html",
        embedded_template!("admin/users_new.html"),
    )
    .expect("Failed to register admin/users_new.html");
    env.add_template(
        "assignments/index.html",
        embedded_template!("assignments/index.html"),
    )
    .expect("Failed to register assignments/index.html");
    env.add_template(
        "assignments/student_index.html",
        embedded_template!("assignments/student_index.html"),
    )
    .expect("Failed to register assignments/student_index.html");
    env.add_template(
        "assignments/student_detail.html",
        embedded_template!("assignments/student_detail.html"),
    )
    .expect("Failed to register assignments/student_detail.html");
    env.add_template(
        "assignments/detail.html",
        embedded_template!("assignments/detail.html"),
    )
    .expect("Failed to register assignments/detail.html");
    env.add_template(
        "assignments/create_assignment.html",
        embedded_template!("assignments/create_assignment.html"),
    )
    .expect("Failed to register assignments/create_assignment.html");
    env.add_template(
        "assignments/instructor_submission_history.html",
        embedded_template!("assignments/instructor_submission_history.html"),
    )
    .expect("Failed to register assignments/instructor_submission_history.html");
    env.add_template(
        "assignments/partials/_row.html",
        embedded_template!("assignments/partials/_row.html"),
    )
    .expect("Failed to register assignments/partials/_row.html");
    env.add_template(
        "assignments/partials/_table.html",
        embedded_template!("assignments/partials/_table.html"),
    )
    .expect("Failed to register assignments/partials/_table.html");
    env.add_template(
        "assignments/_status_badge.html",
        embedded_template!("assignments/_status_badge.html"),
    )
    .expect("Failed to register assignments/_status_badge.html");
    env.add_template(
        "assignments/partials/_attachments_panel.html",
        embedded_template!("assignments/partials/_attachments_panel.html"),
    )
    .expect("Failed to register assignments/partials/_attachments_panel.html");
    env.add_template(
        "classes/index.html",
        embedded_template!("classes/index.html"),
    )
    .expect("Failed to register classes/index.html");
    env.add_template(
        "classes/class_detail.html",
        embedded_template!("classes/class_detail.html"),
    )
    .expect("Failed to register classes/class_detail.html");
    env.add_template(
        "classes/create_class.html",
        embedded_template!("classes/create_class.html"),
    )
    .expect("Failed to register classes/create_class.html");
    env.add_template(
        "classes/student_index.html",
        embedded_template!("classes/student_index.html"),
    )
    .expect("Failed to register classes/student_index.html");

    register_filters(&mut env);

    env
}

fn register_filters(env: &mut Environment<'static>) {
    env.add_filter("format_date", format_date);
    env.add_filter("format_datetime_local", format_datetime_local);
    env.add_filter("format_datetime_display", format_datetime_display);
}

fn format_date(value: Value) -> String {
    if value.is_undefined() {
        return "-".to_string();
    }

    let raw = value.to_string();
    if raw.is_empty() || raw == "none" || raw == "null" {
        return "-".to_string();
    }

    let unquoted = raw.trim_matches('"');
    match DateTime::parse_from_rfc3339(unquoted) {
        Ok(dt) => dt.with_timezone(&Utc).format("%b %-d, %Y").to_string(),
        Err(_) => raw,
    }
}

fn format_datetime_local(value: Value) -> String {
    if value.is_undefined() {
        return "".to_string();
    }

    let raw = value.to_string();
    if raw.is_empty() || raw == "none" || raw == "null" {
        return "".to_string();
    }

    let unquoted = raw.trim_matches('"');
    match DateTime::parse_from_rfc3339(unquoted) {
        Ok(dt) => dt.with_timezone(&Utc).format("%Y-%m-%dT%H:%M").to_string(),
        Err(_) => "".to_string(),
    }
}

fn format_datetime_display(value: Value) -> String {
    if value.is_undefined() {
        return "-".to_string();
    }

    let raw = value.to_string();
    if raw.is_empty() || raw == "none" || raw == "null" {
        return "-".to_string();
    }

    let unquoted = raw.trim_matches('"');
    match DateTime::parse_from_rfc3339(unquoted) {
        Ok(dt) => dt
            .with_timezone(&Utc)
            .format("%b %-d, %Y %H:%M UTC")
            .to_string(),
        Err(_) => raw,
    }
}

fn build_dev_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(path_loader(TEMPLATE_ROOT));
    register_filters(&mut env);
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
