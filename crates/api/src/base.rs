//! Base de datos del servidor: pool compartido y plazo para cada consulta.
//!
//! Ninguna consulta se queda esperando para siempre: si la base no contesta dentro del
//! plazo, la server function devuelve un fallo temporal (503) que la app puede reintentar,
//! y el motivo queda en el log del servidor.

use std::future::Future;
use std::time::Duration;

use dioxus::prelude::*;
use sqlx::PgPool;

use crate::fallos;

/// Segundos que se espera una consulta antes de darla por perdida.
pub const ESPERA: Duration = Duration::from_secs(8);

/// El pool que instaló el binario al arrancar, o el fallo que se le cuenta al cliente.
pub fn pool() -> Result<&'static PgPool, ServerFnError> {
    crate::db::pool().ok_or_else(|| {
        fallos::reintento("El servidor no tiene la base de datos conectada. Vuelve a intentarlo en un momento.")
    })
}

/// Plazo para las consultas de SQLx: se encadena donde termina la consulta.
///
/// ```ignore
/// let filas = sqlx::query_as::<_, Fila>(...).fetch_all(pool).con_plazo("el catálogo").await?;
/// ```
pub trait ConPlazo<T> {
    /// Espera a la consulta como mucho [`ESPERA`] y traduce el fallo a algo legible.
    fn con_plazo(self, que: &'static str) -> impl Future<Output = Result<T, ServerFnError>>;
}

impl<T, F> ConPlazo<T> for F
where
    F: Future<Output = Result<T, sqlx::Error>>,
{
    async fn con_plazo(self, que: &'static str) -> Result<T, ServerFnError> {
        match tokio::time::timeout(ESPERA, self).await {
            Ok(Ok(valor)) => Ok(valor),
            Ok(Err(causa)) => {
                dioxus::logger::tracing::error!("base de datos al leer {que}: {causa}");

                Err(fallos::reintento(format!(
                    "La base de datos rechazó la consulta de {que}. Vuelve a intentarlo en un momento."
                )))
            }
            Err(_) => {
                dioxus::logger::tracing::error!(
                    "base de datos al leer {que}: sin respuesta en {} s",
                    ESPERA.as_secs()
                );

                Err(fallos::reintento(format!(
                    "La base de datos no respondió en {} s al leer {que}. Revisa la conexión y reintenta.",
                    ESPERA.as_secs()
                )))
            }
        }
    }
}

/// Elige el plazo de una llamada a Supabase Auth y traduce el fallo de red.
#[cfg(feature = "server")]
pub fn cliente_http() -> Result<reqwest::Client, ServerFnError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|causa| {
            dioxus::logger::tracing::error!("no se pudo crear el cliente HTTP: {causa}");

            fallos::reintento("El servidor no pudo preparar la conexión con Supabase.")
        })
}
