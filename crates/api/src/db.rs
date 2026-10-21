//! Acceso a PostgreSQL desde las server functions.
//!
//! El pool lo abre el binario (`crates/servidor`) y lo instala aquí al arrancar: así hay una
//! sola conexión gobernada por `DATABASE_URL`, y las server functions pueden consultar sin
//! arrastrar SQLx al paquete del teléfono (este módulo solo existe con la feature `server`).

use std::sync::OnceLock;
use std::time::Duration;

use sqlx::PgPool;

use crate::EstadoSalud;

static POOL: OnceLock<PgPool> = OnceLock::new();

/// Guarda el pool del proceso. Idempotente: el primero que llega manda.
pub fn instalar(pool: PgPool) {
    let _ = POOL.set(pool);
}

/// El pool, si el binario ya lo instaló.
pub fn pool() -> Option<&'static PgPool> {
    POOL.get()
}

/// Estado que se muestra en el cliente y en `/salud`.
pub async fn estado(plazo: Duration) -> EstadoSalud {
    let Some(pool) = pool() else {
        return EstadoSalud {
            base_de_datos: false,
            detalle: "el servidor no tiene DATABASE_URL o no pudo abrir el pool".to_string(),
        };
    };

    match tokio::time::timeout(plazo, sqlx::query("select 1").execute(pool)).await {
        Ok(Ok(_)) => EstadoSalud {
            base_de_datos: true,
            detalle: "la base de datos responde".to_string(),
        },
        Ok(Err(causa)) => EstadoSalud {
            base_de_datos: false,
            detalle: format!("la base de datos rechazó la consulta: {causa}"),
        },
        Err(_) => EstadoSalud {
            base_de_datos: false,
            detalle: format!("la base de datos no contestó en {} s", plazo.as_secs()),
        },
    }
}
