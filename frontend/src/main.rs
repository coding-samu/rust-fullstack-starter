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
    // Resource per il fetch reattivo dei post
    let posts = create_resource(
        || (),
        |_| async move {
            eprintln!("[frontend] RESOURCE GET {}/api/posts", API_BASE);
            match reqwest::Client::new()
                .get(format!("{}/api/posts", API_BASE))
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    eprintln!("[frontend] RESOURCE GET status {status}");
                    match resp.json::<Vec<PostItem>>().await {
                        Ok(data) => {
                            eprintln!("[frontend] RESOURCE GET ok items={}", data.len());
                            data
                        }
                        Err(e) => {
                            eprintln!("[frontend] RESOURCE GET json error: {e}");
                            vec![]
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[frontend] RESOURCE GET error: {e}");
                    vec![]
                }
            }
        },
    );

    // Closure per refresh dopo submit
    let refresh = move || posts.refetch();

    view! {
      <div class="container">
        <h1>"Homepage"</h1>
        <CreateForm on_created=refresh />
        <ul>
          { move || match posts.get() {
              Some(data) => {
                eprintln!("[frontend] Rendering {} posts", data.len());
                data.into_iter()
                    .map(|p| view!{ <li><b>{p.title.clone()}</b> - {p.content.clone()}</li> })
                    .collect_view()
              },
              None => view!{ <li>"Caricamento..."</li> }.into_view(),
          }}
        </ul>
      </div>
    }
}

#[component]
fn CreateForm(on_created: impl Fn() + 'static) -> impl IntoView {
    use std::rc::Rc;
    let on_created = Rc::new(on_created);
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
                        on_created(); // refetch
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
