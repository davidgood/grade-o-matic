use axum::response::Html;
use chrono::Utc;

pub async fn ui_index() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Grade-O-Matic UI</title>
    <script src="https://unpkg.com/htmx.org@2.0.4" defer></script>
    <script src="https://unpkg.com/alpinejs@3.14.9" defer></script>
    <style>
      body { font-family: ui-sans-serif, system-ui, sans-serif; max-width: 56rem; margin: 2rem auto; padding: 0 1rem; }
      .card { border: 1px solid #d1d5db; border-radius: 0.75rem; padding: 1rem; margin-bottom: 1rem; }
      button { background: #111827; color: #fff; border: 0; border-radius: 0.5rem; padding: 0.5rem 0.75rem; cursor: pointer; }
      code { background: #f3f4f6; padding: 0.1rem 0.3rem; border-radius: 0.25rem; }
    </style>
  </head>
  <body>
    <h1>Grade-O-Matic Web UI</h1>

    <div class="card" x-data="{ open: false }">
      <h2>Alpine.js</h2>
      <button @click="open = !open">Toggle Details</button>
      <p x-show="open" x-transition>Client-side interaction is handled by Alpine.</p>
    </div>

    <div class="card">
      <h2>HTMX</h2>
      <button
        hx-get="/ui/fragments/server-time"
        hx-target="#server-time"
        hx-swap="innerHTML"
      >
        Load Server Time
      </button>
      <div id="server-time" style="margin-top: 0.75rem;">Click the button to fetch a partial.</div>
    </div>

    <p>Use this as the starting point for server-rendered pages + HTMX partials.</p>
  </body>
</html>"##,
    )
}

pub async fn server_time_fragment() -> Html<String> {
    let now = Utc::now().to_rfc3339();
    Html(format!("<code>Server time: {now}</code>"))
}
