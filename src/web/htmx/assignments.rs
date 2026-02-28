use axum::response::Html;

pub async fn assignments_table_fragment() -> Html<&'static str> {
    Html(
        r##"<div>
  <p>No assignments yet.</p>
</div>"##,
    )
}
