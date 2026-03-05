use crate::common::error::AppError;
use crate::common::jwt::Claims;
use crate::domains::assignments::AssignmentServiceTrait;
use crate::domains::class_memberships::ClassMembershipServiceTrait;
use crate::domains::classes::ClassServiceTrait;
use crate::domains::user::UserRole;
use crate::web::render_template;
use axum::Extension;
use axum::extract::State;
use axum::response::Html;
use minijinja::context;
use serde::Serialize;
use std::sync::Arc;

pub async fn students_classes_page(
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Student) {
        return Err(AppError::Forbidden);
    }

    let memberships = class_membership_service.list_by_user_id(claims.sub).await?;

    let mut classes = Vec::new();
    for membership in memberships {
        if let Some(class) = class_service.find_by_id(membership.class_id).await? {
            classes.push(class);
        }
    }
    classes.sort_by(|a, b| {
        a.term
            .cmp(&b.term)
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    let html = render_template(
        "classes/student_index.html",
        context! {
            title => "Classes",
            classes => classes,
        },
    )?;
    Ok(Html(html))
}

#[derive(Debug, Serialize)]
struct StudentAssignmentRow {
    class_title: String,
    title: String,
    description: Option<String>,
    due_at: Option<chrono::DateTime<chrono::Utc>>,
    points: Option<i16>,
}

pub async fn students_assignments_page(
    State(class_membership_service): State<Arc<dyn ClassMembershipServiceTrait>>,
    State(class_service): State<Arc<dyn ClassServiceTrait>>,
    State(assignment_service): State<Arc<dyn AssignmentServiceTrait>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    if !matches!(claims.user_role, UserRole::Student) {
        return Err(AppError::Forbidden);
    }

    let memberships = class_membership_service.list_by_user_id(claims.sub).await?;

    let mut rows = Vec::new();

    for membership in memberships {
        let class = match class_service.find_by_id(membership.class_id).await? {
            Some(class) => class,
            None => continue,
        };

        let assignments = assignment_service.list_by_class(class.id).await?;
        for assignment in assignments {
            rows.push(StudentAssignmentRow {
                class_title: class.title.clone(),
                title: assignment.title,
                description: assignment.description,
                due_at: assignment.due_at,
                points: assignment.points,
            });
        }
    }

    rows.sort_by(|a, b| {
        a.due_at
            .cmp(&b.due_at)
            .then_with(|| {
                a.class_title
                    .to_lowercase()
                    .cmp(&b.class_title.to_lowercase())
            })
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });

    let html = render_template(
        "assignments/student_index.html",
        context! {
            title => "My Assignments",
            assignments => rows,
            count => rows.len(),
        },
    )?;
    Ok(Html(html))
}
