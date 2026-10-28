//! Recetario propio (C4): versiones, publicaciones y borradores de la cuenta.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biblioteca::RecetaResumen;

#[cfg(feature = "server")]
use crate::biblioteca::{FilaCapa, FilaReceta};

#[cfg(feature = "server")]
use crate::db;

/// Estado visible de una receta dentro del recetario propio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoReceta {
    /// Receta original que aun no se publica.
    Borrador,
    /// Version derivada que aun no se publica.
    Privada,
    /// Receta visible en el recetario publico.
    Publicada,
    /// Receta retirada por moderacion.
    Oculta,
}

/// Receta propia con lo necesario para listarla y explicar su estado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaPropia {
    /// Resumen con vaso, capas y datos de la receta.
    pub receta: RecetaResumen,
    /// Estado actual dentro del recetario.
    pub estado: EstadoReceta,
    /// Receta de la que deriva, si es una version.
    pub origen_id: Option<String>,
    /// Nombre de la receta de la que deriva, si existe.
    pub origen_nombre: Option<String>,
    /// Fecha de creacion en formato legible.
    pub creada_en: String,
    /// Fecha de la ultima edicion en formato legible.
    pub actualizada_en: String,
}

/// Fila de la consulta del recetario (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaPropia {
    /// Campos compartidos con el catalogo publico.
    #[sqlx(flatten)]
    cabecera: FilaReceta,
    /// Receta de la que deriva.
    origen_id: Option<String>,
    /// Nombre de la receta de la que deriva.
    origen_nombre: Option<String>,
    /// Si tiene fecha de publicacion.
    publicada: bool,
    /// Si fue retirada por moderacion.
    oculta: bool,
    /// Fecha de creacion legible.
    creada_en: String,
    /// Fecha de la ultima edicion legible.
    actualizada_en: String,
}

#[cfg(feature = "server")]
impl FilaPropia {
    /// Convierte la fila y sus capas en la receta que consume C4.
    fn a_propia(self, capas: &[FilaCapa]) -> RecetaPropia {
        let estado = if self.oculta {
            EstadoReceta::Oculta
        } else if self.publicada {
            EstadoReceta::Publicada
        } else if self.origen_id.is_some() {
            EstadoReceta::Privada
        } else {
            EstadoReceta::Borrador
        };

        RecetaPropia {
            receta: self.cabecera.a_resumen(capas),
            estado,
            origen_id: self.origen_id,
            origen_nombre: self.origen_nombre,
            creada_en: self.creada_en,
            actualizada_en: self.actualizada_en,
        }
    }
}

/// Trae todas las recetas de la cuenta, de la mas reciente a la mas antigua.
///
/// Incluye borradores, versiones privadas, publicaciones y recetas ocultas: C4 es el
/// unico lugar donde el autor ve todo lo suyo en una sola lista.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el token no es valido, si el servidor no tiene pool
/// o si alguna consulta falla.
#[server(endpoint = "mis_recetas")]
pub async fn mis_recetas(token: String) -> Result<Vec<RecetaPropia>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| ServerFnError::new("La sesion caduco; vuelve a entrar"))?;

    let filas = sqlx::query_as::<_, FilaPropia>(
        r#"
        SELECT r.id::text AS id,
               r.nombre,
               r.tipo,
               r.familia,
               u.alias AS autor,
               COALESCE(r.capacidad_ml, v.capacidad_ml) AS capacidad_ml,
               v.capacidad_ml AS vaso_capacidad_ml,
               v.radio_boca,
               v.radio_fondo,
               v.altura,
               v.pared,
               v.borde,
               v.tallo,
               v.asa,
               v.plato,
               r.minutos,
               r.dificultad,
               r.intensidad,
               r.receta_origen_id::text AS origen_id,
               o.nombre AS origen_nombre,
               (r.publicada_en IS NOT NULL) AS publicada,
               r.oculta,
               to_char(r.creada_en AT TIME ZONE 'UTC', 'DD/MM/YYYY') AS creada_en,
               to_char(r.actualizado_en AT TIME ZONE 'UTC', 'DD/MM/YYYY') AS actualizada_en
        FROM receta r
        JOIN vaso v ON v.id = r.vaso_id
        JOIN usuario u ON u.id = r.autor_id
        LEFT JOIN receta o ON o.id = r.receta_origen_id
        WHERE r.autor_id = $1::uuid
        ORDER BY r.creada_en DESC
        "#,
    )
    .bind(&cuenta)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let capas = sqlx::query_as::<_, FilaCapa>(
        r#"
        SELECT c.receta_id::text AS receta_id, c.nombre, c.cantidad, c.graduacion_pct, c.color, c.dilucion
        FROM componente_receta c
        JOIN receta r ON r.id = c.receta_id
        WHERE c.unidad = 'ml'
          AND r.autor_id = $1::uuid
        ORDER BY c.receta_id, c.posicion
        "#,
    )
    .bind(&cuenta)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    Ok(filas
        .into_iter()
        .map(|fila| fila.a_propia(&capas))
        .collect())
}
