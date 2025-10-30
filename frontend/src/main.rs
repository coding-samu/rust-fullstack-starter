use leptos::prelude::*;
use leptos_axum::LeptosRoutes;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PostItem { id: String, title: String, content: String, created_at: String }

#[component]
fn HomePage() -> impl IntoView {
    let (posts, set_posts) = create_signal::<Vec<PostItem>>(vec![]);

    let fetch = move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = reqwest::Client::new().get("/api/posts").send().await {
                if let Ok(data) = resp.json::<Vec<PostItem>>().await { set_posts.set(data); }
            }
        });
    };

    on_mount(fetch);

    view! {
      <div class="container">
        <h1>"Homepage"</h1>
        <CreateForm on_created=fetch />
        <ul>
          { move || posts.get().into_iter().map(|p| view!{ <li><b>{p.title.clone()}</b> - {p.content.clone()}</li> }).collect_view() }
        </ul>
      </div>
    }
}

#[component]
fn CreateForm(on_created: Callback<()>) -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let t = title.get();
        let c = content.get();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = reqwest::Client::new()
                .post("/api/posts")
                .json(&serde_json::json!({"title": t, "content": c}))
                .send().await;
            on_created(());
        });
    };

    view! {
      <form on:submit=submit>
        <input type="text" placeholder="Titolo" prop:value=title on:input=move |e| set_title.set(event_target_value(&e)) />
        <input type="text" placeholder="Contenuto" prop:value=content on:input=move |e| set_content.set(event_target_value(&e)) />
        <button type="submit">"Crea Post"</button>
      </form>
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let app = Router::new()
        .route("/", get(leptos_axum::render_app_to_stream(|| view!{ <HomePage/> })))
        .fallback_service(axum::routing::get_service(tower_http::services::ServeDir::new("target/site")).handle_error(|err| async move {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")).into_response()
        }));

    let addr = std::net::SocketAddr::from(([0,0,0,0], 3001));
    tracing::info!(%addr, "frontend listening");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}
