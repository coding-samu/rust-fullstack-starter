use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

const API_BASE: &str = "http://rustfs_backend:3000"; // Docker service hostname

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PostItem {
    id: String,
    title: String,
    content: String,
    created_at: String,
}

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState {
        client: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/", get(homepage).post(create_and_redirect))
        .route("/create", post(create_and_redirect))
        .nest_service("/assets", ServeDir::new("target/site"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!(%addr, "frontend listening");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn homepage(State(state): State<AppState>) -> impl IntoResponse {
    // Fetch posts from backend
    let list = fetch_posts(&state).await.unwrap_or_default();
    let body = render_index(&list);
    Html(body)
}

#[derive(Deserialize)]
struct CreateFormInput {
    title: String,
    content: String,
}

async fn create_and_redirect(
    State(state): State<AppState>,
    Form(input): Form<CreateFormInput>,
) -> impl IntoResponse {
    let _ = state
        .client
        .post(format!("{}/api/posts", API_BASE))
        .json(&serde_json::json!({"title": input.title, "content": input.content}))
        .send()
        .await;

    // Redirect back to /
    (StatusCode::SEE_OTHER, [(header::LOCATION, "/")]).into_response()
}

async fn fetch_posts(state: &AppState) -> anyhow::Result<Vec<PostItem>> {
    let resp = state
        .client
        .get(format!("{}/api/posts", API_BASE))
        .send()
        .await?;
    let posts = resp.json::<Vec<PostItem>>().await?;
    Ok(posts)
}

fn render_index(posts: &Vec<PostItem>) -> String {
    let mut items = String::new();
    for p in posts {
        items.push_str(&format!(
            "<li><strong>{}</strong> — {}</li>",
            html_escape(&p.title),
            html_escape(&p.content)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="it">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Homepage</title>
  <style>
    body {{ font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif; margin: 24px; }}
    .container {{ max-width: 720px; margin: 0 auto; }}
    form {{ display: grid; gap: 12px; margin: 16px 0 24px; }}
    input[type=text] {{ padding: 10px 12px; border: 1px solid #ccc; border-radius: 8px; }}
    button {{ padding: 10px 14px; border-radius: 8px; border: none; background: #21808d; color: #fff; cursor: pointer; }}
    button:hover {{ background: #1d7480; }}
    ul {{ padding-left: 18px; }}
  </style>
</head>
<body>
  <div class="container">
    <h1>Homepage</h1>

    <form method="post" action="/create">
      <input type="text" name="title" placeholder="Titolo" required />
      <input type="text" name="content" placeholder="Contenuto" required />
      <button type="submit">Crea Post</button>
    </form>

    <h2>Post</h2>
    <ul>
      {}
    </ul>
  </div>
</body>
</html>"#,
        items
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}