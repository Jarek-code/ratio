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
        Ok(config) => Some(config),
        Err(motivo) => {
            error!("{motivo}");
            None
        }
    };

    dioxus::serve(move || {
        let config = config.clone();
        async move { arrancar(config).await }
    });
}

/// Fuerza el enlace de las server functions: Dioxus registra solo las que el binario referencia.
fn enlazar_contratos() {
    let _ = (
        api::analizar,
        api::salud,
        api::auth::entrar,
        api::auth::registrar,
        api::auth::sesion,
        api::biblioteca::catalogo,
        api::descubrir::sugerencias_dia,
        api::receta::completa,
    );
}

/// Prepara la base y el router. Si falta configuración arranca igual: el proceso queda en pie
/// y `/salud` dice qué falta, en vez de morir en cada despliegue sin dejar diagnóstico.
async fn arrancar(config: Option<config::Config>) -> Result<axum::Router, Error> {
    match &config {
        Some(config) => {
            abrir_pool(config).await;

            if config.migrar {
                aplicar_migraciones().await;
            }
        }
        None => warn!("sin DATABASE_URL: se arranca en modo degradado"),
    }

    Ok(axum::Router::new()
        .route("/salud", get(salud))
        .register_server_functions()
        .layer(axum::middleware::from_fn(registrar_peticion))
        .with_state(FullstackState::headless()))
}

/// Anota método, ruta y estado de cada petición. Se ve con `RUST_LOG=debug`.
async fn registrar_peticion(
    peticion: axum::extract::Request,
    siguiente: axum::middleware::Next,
) -> axum::response::Response {
    let metodo = peticion.method().clone();
    let ruta = peticion.uri().path().to_string();
    let respuesta = siguiente.run(peticion).await;

    tracing::debug!(%metodo, %ruta, estado = respuesta.status().as_u16(), "petición");

    respuesta
}

/// Abre el pool y lo publica para las server functions; si no conecta, deja uno perezoso
/// que reintenta en cada consulta.
async fn abrir_pool(config: &config::Config) {
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

    api::db::instalar(pool);

    match (&config.supabase_url, &config.supabase_anon) {
        (Some(url), Some(anon)) => api::auth::configurar(url.clone(), anon.clone()),
        _ => warn!("sin SUPABASE_URL o SUPABASE_ANON_KEY: el acceso no podrá validar credenciales"),
    }
}

/// Aplica `migrations/` con el pool ya instalado.
async fn aplicar_migraciones() {
    let Some(pool) = api::db::pool() else {
        return;
    };

    let mut migrador = sqlx::migrate!("../../migrations");
    migrador.set_ignore_missing(true);

    match migrador.run(pool).await {
        Ok(()) => info!("migraciones aplicadas"),
        Err(causa) => error!(error = %causa, "las migraciones fallaron; se arranca sin aplicarlas"),
    }
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
