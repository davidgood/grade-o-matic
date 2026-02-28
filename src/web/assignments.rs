use axum::response::Html;

pub async fn assignments_page() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Assignments</title>
    <script src="https://unpkg.com/htmx.org@2.0.4" defer></script>
    <script src="https://unpkg.com/alpinejs@3.14.9" defer></script>
    <style>
      body { font-family: ui-sans-serif, system-ui, sans-serif; max-width: 56rem; margin: 2rem auto; padding: 0 1rem; }
      .toolbar { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
      button { background: #111827; color: #fff; border: 0; border-radius: 0.5rem; padding: 0.5rem 0.75rem; cursor: pointer; }
      .panel { border: 1px solid #d1d5db; border-radius: 0.75rem; padding: 1rem; }
    </style>
  </head>
  <body>
    <h1>Assignments</h1>

    <div class="toolbar" x-data="{ showFilters: false }">
      <button @click="showFilters = !showFilters">Toggle Filters</button>
      <button hx-get="/ui/fragments/assignments/table" hx-target="#assignments-table" hx-swap="innerHTML">Refresh Table</button>
    </div>

    <div x-show="showFilters" x-transition class="panel" style="margin-bottom: 1rem;">
      <strong>Filters</strong>
      <p>Filter controls will go here.</p>
    </div>

    <div id="assignments-table" class="panel" hx-get="/ui/fragments/assignments/table" hx-trigger="load" hx-swap="innerHTML">
      Loading assignments...
    </div>
  </body>
</html>"##,
    )
}
