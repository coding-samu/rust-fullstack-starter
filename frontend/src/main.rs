use leptos::prelude::*;
use leptos::view;
use leptos::component;
use leptos::IntoView;
use leptos::event_target_value;
use leptos::CollectView;
use axum::{response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PostItem { id: String, title: String, content: String, created_at: String }

const API_BASE: &str = "http://rustfs_backend:3000"; // use service name inside Docker network

#[component]
fn HomePage() -> impl IntoView {
    let (posts, set_posts) = create_signal::<Vec<PostItem>>(vec![]);

    // Local fetch function used safely without reactive owner requirements
    let do_fetch = {
        let set_posts = set_posts.clone();
        move || {
            eprintln!("[frontend] GET {}/api/posts", API_BASE);
            leptos::spawn_local(async move {
                match reqwest::Client::new().get(format!("{}/api/posts", API_BASE)).send().await {
                    Ok(resp) => {
                        let status = resp.status();
                        eprintln!("[frontend] GET status {status}");
                        match resp.json::<Vec<PostItem>>().await {
                            Ok(data) => {
                                eprintln!("[frontend] GET ok items={}", data.len());
                                if let Some(first) = data.get(0) { eprintln!("[frontend] First item: {:?}", first); }
                                set_posts.set(data);
                            }
                            Err(e) => eprintln!("[frontend] GET json error: {e}"),
                        }
                    }
                    Err(e) => eprintln!("[frontend] GET error: {e}"),
                }
            });
        }
    };

    // Initial fetch
    do_fetch();

    view! {
      <div class="container">
        <h1>"Homepage"</h1>
        <CreateForm on_created={{
          let do_fetch = do_fetch.clone();
          move || do_fetch()
        }} />
        <ul>
          { move || {
              let posts_data = posts.get();
              eprintln!("[frontend] Rendering {} posts", posts_data.len());
              posts_data.into_iter().map(|p| view!{ <li><b>{p.title.clone()}</b> - {p.content.clone()}</li> }).collect_view()
          }}
        </ul>
      </div>
    }
}

#[component]
fn CreateForm(on_created: impl Fn() + 'static) -> impl IntoView {
    let on_created = std::rc::Rc::new(on_created);
    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());

    let submit = {
        let on_created = on_created.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let t = title.get();
            let c = content.get();
            let on_created = on_created.clone();
            eprintln!("[frontend] POST {}/api/posts title='{t}'", API_BASE);
            leptos::spawn_local(async move {
                match reqwest::Client::new()
                    .post(format!("{}/api/posts", API_BASE))
                    .json(&serde_json::json!({"title": t, "content": c}))
                    .send().await {
                    Ok(resp) => {
                        let status = resp.status();
                        eprintln!("[frontend] POST status {status}");
                        (on_created)();
                    }
                    Err(e) => eprintln!("[frontend] POST error: {e}"),
                }
            });
        }
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

    let leptos_options = leptos::LeptosOptions::builder()
        .output_name("frontend")
        .site_addr(std::net::SocketAddr::from(([0,0,0,0], 3001)))
        .build();

    let app = Router::new()
        .route("/", get(leptos_axum::render_app_to_stream(leptos_options, || view!{ <HomePage/> })))
        .fallback_service(axum::routing::get_service(tower_http::services::ServeDir::new("target/site")).handle_error(|err| async move {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")).into_response()
        }));

    let addr = std::net::SocketAddr::from(([0,0,0,0], 3001));
    tracing::info!(%addr, "frontend listening");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}
