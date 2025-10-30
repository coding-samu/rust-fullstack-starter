use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post, put}, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, net::SocketAddr};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct AppState { pool: PgPool }

#[derive(Serialize, sqlx::FromRow)]
struct PostItem { id: Uuid, title: String, content: String, created_at: chrono::DateTime<chrono::Utc> }

#[derive(Deserialize)]
struct CreatePost { title: String, content: String }

#[derive(Deserialize)]
struct UpdatePost { title: Option<String>, content: Option<String> }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3000);

    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState { pool };

    let app = Router::new()
        .route("/api/posts", get(list_posts).post(create_post))
        .route("/api/posts/:id", get(get_post).put(update_post))
        .route("/", get(homepage))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!(%addr, "listening");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn homepage(State(state): State<AppState>) -> impl IntoResponse {
    // Show latest posts
    let posts: Vec<PostItem> = sqlx::query_as!(PostItem, r#"SELECT id, title, content, created_at FROM posts ORDER BY created_at DESC LIMIT 20"#)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    Json(posts)
}

async fn list_posts(State(state): State<AppState>) -> impl IntoResponse {
    let posts: Vec<PostItem> = sqlx::query_as!(PostItem, r#"SELECT id, title, content, created_at FROM posts ORDER BY created_at DESC"#)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    Json(posts)
}

async fn get_post(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query_as!(PostItem, r#"SELECT id, title, content, created_at FROM posts WHERE id = $1"#, id)
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(p)) => Json(p).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn create_post(State(state): State<AppState>, Json(input): Json<CreatePost>) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let res = sqlx::query!(r#"INSERT INTO posts (id, title, content) VALUES ($1, $2, $3)"#, id, input.title, input.content)
        .execute(&state.pool)
        .await;
    match res {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn update_post(State(state): State<AppState>, Path(id): Path<Uuid>, Json(input): Json<UpdatePost>) -> impl IntoResponse {
    let res = sqlx::query!(
        r#"UPDATE posts SET title = COALESCE($2, title), content = COALESCE($3, content) WHERE id = $1"#,
        id,
        input.title,
        input.content
    )
    .execute(&state.pool)
    .await;
    match res {
        Ok(r) if r.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
