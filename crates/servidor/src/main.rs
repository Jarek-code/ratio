//! Backend de Ratio: Dioxus sin interfaz, solo server functions y `/salud`.

mod config;
mod db;

use std::time::Duration;

use anyhow::Error;
use axum::http::StatusCode;
use axum::routing::get;
use dioxus::server::{DioxusRouterExt, FullstackState, axum};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    enlazar_contratos();

    let config = match config::Config::desde_entorno() {
        Ok(config) => config,
        Err(motivo) => {
            error!("{motivo}");
            std::process::exit(2);
        }
    };

    dioxus::serve(move || {
        let config = config.clone();
        async move { arrancar(config).await }
    });
}

/// Fuerza el enlace de las server functions: Dioxus registra solo las que el binario referencia.
fn enlazar_contratos() {
    let _ = (api::analizar, api::salud);
}

/// Arma el pool y el router. Vivimos dentro del runtime de Dioxus, que es el que atiende.
async fn arrancar(config: config::Config) -> Result<axum::Router, Error> {
    let pool = match db::conectar(&config.database_url).await {
        Ok(pool) => {
            info!(base = %config.destino(), "conectado a la base de datos");
            pool
        }
        Err(causa) => {
            warn!(
                base = %config.destino(),
                error = %causa,
                "sin conexión a la base de datos; se arranca en modo degradado"
            );
            db::perezoso(&config.database_url)
        }
    };

    api::db::instalar(pool.clone());

    if config.migrar {
        let mut migrador = sqlx::migrate!("../../migrations");
        migrador.set_ignore_missing(true);

        match migrador.run(&pool).await {
            Ok(()) => info!("migraciones aplicadas"),
            Err(causa) => {
                error!(error = %causa, "las migraciones fallaron; se arranca sin aplicarlas");
            }
        }
    }

    Ok(axum::Router::new()
        .route("/salud", get(salud))
        .register_server_functions()
        .with_state(FullstackState::headless()))
}

/// Lo que consulta Render y lo que enseña el cliente. `200` si la base contesta;
/// `503` con el motivo si no.
async fn salud() -> (StatusCode, String) {
    let estado = api::db::estado(Duration::from_secs(3)).await;

    if estado.base_de_datos {
        (StatusCode::OK, "ok".to_string())
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, estado.detalle)
    }
}
