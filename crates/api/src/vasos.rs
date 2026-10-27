//! Catálogo de vasos que puede elegir la pantalla Ajustar proporciones (C1).
//!
//! El motor de proporciones necesita la silueta completa, no solo el nombre: por eso el
//! contrato devuelve `proporciones::Vaso` listo para recalcular el dibujo y las capas.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Un vaso del catálogo, con lo necesario para dibujarlo y medir su contenido.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VasoOpcion {
    /// Identificador del vaso en la base.
    pub id: String,
    /// Nombre visible en la pantalla.
    pub nombre: String,
    /// Silueta y capacidad que consume el motor.
    pub vaso: proporciones::Vaso,
}

/// Fila de la consulta de vasos (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaVaso {
    /// Identificador del vaso (UUID en texto).
    id: String,
    /// Nombre visible.
    nombre: String,
    /// Capacidad útil hasta el borde interior (ml).
    capacidad_ml: f64,
    /// Media anchura de la boca.
    radio_boca: f64,
    /// Media anchura del fondo.
    radio_fondo: f64,
    /// Altura interior del vaso.
    altura: f64,
    /// Grosor de la pared.
    pared: f64,
    /// Labio sobre el nivel máximo del líquido.
    borde: f64,
    /// Altura del tallo.
    tallo: f64,
    /// Tamaño del asa, si tiene.
    asa: Option<String>,
    /// Si el vaso lleva plato.
    plato: bool,
}

#[cfg(feature = "server")]
impl FilaVaso {
    /// Convierte la fila en la opción que entiende la pantalla.
    fn a_opcion(self) -> VasoOpcion {
        let asa = match self.asa.as_deref() {
            Some("chica") => Some(proporciones::Asa::Chica),
            Some("grande") => Some(proporciones::Asa::Grande),
            _ => None,
        };

        VasoOpcion {
            id: self.id,
            nombre: self.nombre,
            vaso: proporciones::Vaso {
                capacidad_ml: self.capacidad_ml,
                radio_boca: self.radio_boca,
                radio_fondo: self.radio_fondo,
                altura: self.altura,
                pared: self.pared,
                borde: self.borde,
                tallo: self.tallo,
                asa,
                plato: self.plato,
            },
        }
    }
}

/// Trae el catálogo de vasos ordenado de menor a mayor capacidad.
///
/// La pantalla Ajustar lo usa para cambiar de recipiente sin perder las cantidades.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si la consulta falla.
#[server(endpoint = "vasos")]
pub async fn vasos() -> Result<Vec<VasoOpcion>, ServerFnError> {
    let Some(pool) = crate::db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let filas = sqlx::query_as::<_, FilaVaso>(
        r#"
        SELECT id::text AS id,
               nombre,
               capacidad_ml,
               radio_boca,
               radio_fondo,
               altura,
               pared,
               borde,
               tallo,
               asa,
               plato
        FROM vaso
        ORDER BY capacidad_ml, nombre
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    Ok(filas
        .into_iter()
        .map(FilaVaso::a_opcion)
        .collect::<Vec<_>>())
}
