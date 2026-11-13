//! Guardados (C5): las recetas que la cuenta quiere volver a preparar.
//!
//! La lista muestra las recetas públicas visibles y también las privadas propias, ordenadas
//! por la fecha en que se guardaron. El detalle de cada receta se reutiliza de
//! `RecetaResumen`, igual que en la biblioteca y en el recetario propio.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biblioteca::RecetaResumen;

#[cfg(feature = "server")]
use crate::biblioteca::{FilaCapa, FilaReceta};
#[cfg(feature = "server")]
use crate::db;

/// Receta guardada con la fecha en que se añadió a la lista.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaGuardada {
    /// Resumen con vaso, capas y datos de la receta.
    pub receta: RecetaResumen,
    /// Fecha en que se guardó, en formato legible.
    pub guardado_en: String,
    /// Fecha de la última preparación propia, si existe.
    pub ultima_preparacion: Option<String>,
}

/// Fila de la consulta de guardados (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaGuardada {
    /// Campos compartidos con el catálogo público.
    #[sqlx(flatten)]
    cabecera: FilaReceta,
    /// Fecha en que se guardó la receta.
    guardado_en: String,
    /// Última preparación propia de esa receta.
    ultima_preparacion: Option<String>,
}

#[cfg(feature = "server")]
impl FilaGuardada {
    /// Convierte la fila y sus capas en la receta que consume C5.
    fn a_guardada(self, capas: &[FilaCapa]) -> RecetaGuardada {
        RecetaGuardada {
            receta: self.cabecera.a_resumen(capas),
            guardado_en: self.guardado_en,
            ultima_preparacion: self.ultima_preparacion,
        }
    }
}

/// Trae las recetas guardadas por la cuenta, de la más reciente a la más antigua.
///
/// Aparecen las recetas públicas no ocultas y las privadas del autor; si una receta
/// pública se retira, deja de mostrarse aunque la fila de `guardado` siga existiendo.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el token no es válido, si el servidor no tiene pool
/// o si alguna consulta falla.
#[server(endpoint = "guardados")]
pub async fn guardados(token: String) -> Result<Vec<RecetaGuardada>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| ServerFnError::new("La sesión caducó; vuelve a entrar"))?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    let filas = sqlx::query_as::<_, FilaGuardada>(
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
               to_char(g.creado_en AT TIME ZONE 'UTC', 'DD/MM/YYYY') AS guardado_en,
               to_char(MAX(p.creada_en) AT TIME ZONE 'UTC', 'DD/MM/YYYY') AS ultima_preparacion
        FROM guardado g
        JOIN receta r ON r.id = g.receta_id
        JOIN vaso v ON v.id = r.vaso_id
        JOIN usuario u ON u.id = r.autor_id
        LEFT JOIN preparacion p
               ON p.receta_id = r.id
              AND p.usuario_id = g.usuario_id
        WHERE g.usuario_id = $1::uuid
          AND (
                (r.publicada_en IS NOT NULL AND NOT r.oculta)
                OR r.autor_id = $1::uuid
              )
          AND ($2 OR NOT EXISTS (
                SELECT 1 FROM componente_receta alcohol
                WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
              ))
        GROUP BY g.creado_en, r.id, u.alias, v.id
        ORDER BY g.creado_en DESC
        "#,
    )
    .bind(&cuenta)
    .bind(adulto)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let capas = sqlx::query_as::<_, FilaCapa>(
        r#"
        SELECT c.receta_id::text AS receta_id,
               c.ingrediente_id::text AS ingrediente_id,
               c.posicion,
               c.nombre,
               c.unidad,
               c.cantidad,
               c.graduacion_pct,
               c.color,
               c.dilucion
        FROM componente_receta c
        JOIN guardado g ON g.receta_id = c.receta_id
        JOIN receta r ON r.id = c.receta_id
        WHERE g.usuario_id = $1::uuid
          AND (
                (r.publicada_en IS NOT NULL AND NOT r.oculta)
                OR r.autor_id = $1::uuid
              )
          AND ($2 OR NOT EXISTS (
                SELECT 1 FROM componente_receta alcohol
                WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
              ))
        ORDER BY c.receta_id, c.posicion
        "#,
    )
    .bind(&cuenta)
    .bind(adulto)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    Ok(filas
        .into_iter()
        .map(|fila| fila.a_guardada(&capas))
        .collect())
}

/// Añade o quita una receta de la lista de guardados de la cuenta.
///
/// Solo se pueden guardar recetas visibles: publicadas y no ocultas, o propias.
/// Devuelve el estado final (`true` si quedó guardada).
///
/// # Errores
///
/// Devuelve `ServerFnError` si el token no es válido, si la receta no existe o no
/// está disponible, si el servidor no tiene pool o si la escritura falla.
#[server(endpoint = "alternar_guardado")]
pub async fn alternar_guardado(
    token: String,
    receta_id: String,
    guardar: bool,
) -> Result<bool, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| ServerFnError::new("La sesión caducó; vuelve a entrar"))?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    let visible = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM receta r
            WHERE r.id = $1::uuid
              AND (
                    (r.publicada_en IS NOT NULL AND NOT r.oculta)
                    OR r.autor_id = $2::uuid
                  )
              AND ($3 OR NOT EXISTS (
                    SELECT 1 FROM componente_receta alcohol
                    WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
                  ))
        )
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .bind(adulto)
    .fetch_one(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    if !visible {
        return Err(ServerFnError::new(
            "La receta no existe o no está disponible",
        ));
    }

    if guardar {
        sqlx::query(
            r#"
            INSERT INTO guardado (usuario_id, receta_id)
            VALUES ($1::uuid, $2::uuid)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&cuenta)
        .bind(&receta_id)
        .execute(pool)
        .await
        .map_err(|_| ServerFnError::new("No se pudo guardar la receta"))?;

        Ok(true)
    } else {
        sqlx::query(
            r#"
            DELETE FROM guardado
            WHERE usuario_id = $1::uuid
              AND receta_id = $2::uuid
            "#,
        )
        .bind(&cuenta)
        .bind(&receta_id)
        .execute(pool)
        .await
        .map_err(|_| ServerFnError::new("No se pudo quitar la receta"))?;

        Ok(false)
    }
}
