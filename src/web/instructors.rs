use axum::response::Html;
use axum::{
    Extension,
    extract::{Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use minijinja::context;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::classes::dto::class_dto::{CreateClassDto, UpdateClassDto};
use crate::domains::user::UserRole;

use super::render_template;

pub async fn instructors_page(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let classes = class_service.list().await?;
    let owned_classes = classes
        .into_iter()
        .filter(|class| class.owner_id == Some(claims.sub))
        .collect::<Vec<_>>();

    let html = render_template(
        "instructors/index.html",
        context! {
            title => "Instructors",
            classes => owned_classes,
        },
    )?;
    Ok(Html(html))
}

pub async fn instructor_class_detail_page(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    // Allow owner instructors and admins to access class details.
    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let assignments = assignment_service.list_by_class(class_id).await?;

    let html = render_template(
        "classes/class_detail.html",
        context! {
            title => class.title.clone(),
            class => class,
            assignments => assignments,
        },
    )?;
    Ok(Html(html))
}

pub async fn create_class_page(
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let html = render_template(
        "classes/create_class.html",
        context! {
            title => "Create Class",
            error => "",
            class => None::<crate::domains::classes::dto::class_dto::ClassDto>,
            form_action => "/ui/instructors/classes/new",
            title_value => "",
            term_value => "",
            description_value => "",
        },
    )?;
    Ok(Html(html))
}

pub async fn edit_class_page(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let class = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && class.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let html = render_template(
        "classes/create_class.html",
        context! {
            title => "Edit Class",
            error => "",
            class => class,
            form_action => format!("/ui/instructors/classes/{class_id}/edit"),
            title_value => "",
            term_value => "",
            description_value => "",
        },
    )?;
    Ok(Html(html))
}

#[derive(serde::Deserialize)]
pub struct CreateClassForm {
    title: String,
    description: Option<String>,
    term: Option<String>,
}

pub async fn create_class_submit(
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<CreateClassForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let title = form.title.trim().to_string();
    let term_value = form.term.as_deref().unwrap_or("").trim().to_string();
    let description_value = form.description.as_deref().unwrap_or("").trim().to_string();

    if title.is_empty() {
        let html = render_template(
            "classes/create_class.html",
            context! {
                title => "Create Class",
                error => "Title is required.",
                title_value => "",
                term_value => term_value.clone(),
                description_value => description_value.clone(),
            },
        )?;
        return Ok((StatusCode::BAD_REQUEST, Html(html)).into_response());
    }

    let payload = CreateClassDto {
        title,
        description: if description_value.is_empty() {
            None
        } else {
            Some(description_value.clone())
        },
        term: if term_value.is_empty() {
            None
        } else {
            Some(term_value.clone())
        },
        owner_id: Some(claims.sub),
        modified_by: claims.sub,
    };

    match class_service.create(payload).await {
        Ok(created) => Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(&format!("/ui/instructors/classes/{}", created.id)),
        )
            .into_response()),
        Err(err) => {
            let html = render_template(
                "classes/create_class.html",
                context! {
                    title => "Create Class",
                    error => err.to_string(),
                    title_value => form.title,
                    term_value => term_value,
                    description_value => description_value,
                },
            )?;
            Ok((StatusCode::BAD_REQUEST, Html(html)).into_response())
        }
    }
}

pub async fn edit_class_submit(
    Path(class_id): Path<Uuid>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<CreateClassForm>,
) -> Result<Response, AppError> {
    if !matches!(claims.user_role, UserRole::Instructor | UserRole::Admin) {
        return Err(AppError::Forbidden);
    }

    let existing = class_service
        .find_by_id(class_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Class not found".into()))?;

    if !matches!(claims.user_role, UserRole::Admin) && existing.owner_id != Some(claims.sub) {
        return Err(AppError::Forbidden);
    }

    let title = form.title.trim().to_string();
    let term_value = form.term.as_deref().unwrap_or("").trim().to_string();
    let description_value = form.description.as_deref().unwrap_or("").trim().to_string();

    if title.is_empty() {
        let html = render_template(
            "classes/create_class.html",
            context! {
                title => "Edit Class",
                error => "Title is required.",
                class => existing,
                form_action => format!("/ui/instructors/classes/{class_id}/edit"),
                title_value => title,
                term_value => term_value,
                description_value => description_value,
            },
        )?;
        return Ok((StatusCode::BAD_REQUEST, Html(html)).into_response());
    }

    let payload = UpdateClassDto {
        id: class_id,
        title,
        description: if description_value.is_empty() {
            None
        } else {
            Some(description_value.clone())
        },
        term: if term_value.is_empty() {
            None
        } else {
            Some(term_value.clone())
        },
        owner_id: existing.owner_id,
        modified_by: claims.sub,
    };

    match class_service.update(payload).await {
        Ok(Some(updated)) => Ok((
            StatusCode::SEE_OTHER,
            Redirect::to(&format!("/ui/instructors/classes/{}", updated.id)),
        )
            .into_response()),
        Ok(None) => Err(AppError::NotFound("Class not found".into())),
        Err(err) => {
            let html = render_template(
                "classes/create_class.html",
                context! {
                    title => "Edit Class",
                    error => err.to_string(),
                    class => existing,
                    form_action => format!("/ui/instructors/classes/{class_id}/edit"),
                    title_value => "",
                    term_value => term_value,
                    description_value => description_value,
                },
            )?;
            Ok((StatusCode::BAD_REQUEST, Html(html)).into_response())
        }
    }
}
