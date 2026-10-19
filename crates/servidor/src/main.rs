//! Backend de Ratio: Dioxus sin interfaz, solo server functions y `/salud`.

use dioxus::server::{DioxusRouterExt, FullstackState, axum};

fn main() {
    let _ = api::analizar;

    dioxus::serve(|| async move {
        let router = axum::Router::new()
            .route("/salud", axum::routing::get(salud))
            .register_server_functions()
            .with_state(FullstackState::headless());

        Ok(router)
    });
}

async fn salud() -> &'static str {
    "ok"
}
