//! Conexión a PostgreSQL con SQLx.
//!
//! Aquí vive el pool y, más adelante, una consulta por módulo (`db/recetas.rs`, `db/foro.rs`…).
//! Las consultas se escriben con `sqlx::query` en ejecución, no con las macros `query!`, para
//! que la imagen de Render se construya sin una base de datos a mano.

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Cuántas conexiones mantiene el proceso. Supabase limita el total del proyecto, y el
/// backend es un solo servicio: cinco basta para el piloto y deja margen al pooler.
const MAX_CONEXIONES: u32 = 5;

/// Ajustes del pool. `idle_timeout` recicla antes de que Supabase cierre las conexiones
/// ociosas por su cuenta.
fn opciones() -> PgPoolOptions {
    PgPoolOptions::new()
        .max_connections(MAX_CONEXIONES)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
}

/// Abre el pool y comprueba que hay conexión real.
pub async fn conectar(url: &str) -> Result<PgPool, sqlx::Error> {
    opciones().connect(url).await
}

/// Pool que no conecta todavía: el servicio arranca y `/salud` informa del problema,
/// en lugar de entrar en un bucle de reinicios.
pub fn perezoso(url: &str) -> PgPool {
    opciones()
        .connect_lazy(url)
        .expect("la cadena de conexión ya se validó al leer el entorno")
}
