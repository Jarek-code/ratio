//! D7: reportar una receta o un comentario para que lo revise moderacion.
//!
//! El reporte no cambia el contenido: solo deja una fila en `reporte` para la cola
//! de D8. El reportante sale del token; el cliente nunca manda su id. Un mismo
//! usuario no puede reportar dos veces el mismo objetivo.
//!
//! Los comentarios de este modulo van sin acentos por pedido del equipo.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db;

/// Motivos que acepta la tabla `reporte`.
const MOTIVOS: [&str; 5] = ["spam", "ofensivo", "peligroso", "menores", "otro"];

/// Maximo de caracteres del detalle libre.
const DETALLE_MAXIMO: usize = 1000;

/// Datos del formulario de reporte.
///
/// Debe venir exactamente un objetivo: `receta_id` o `comentario_id`. El motivo
/// pertenece al conjunto fijo de la tabla y el detalle es opcional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReporteNuevo {
    /// Receta reportada; `None` si se reporta un comentario.
    pub receta_id: Option<String>,
    /// Comentario reportado; `None` si se reporta una receta.
    pub comentario_id: Option<String>,
    /// Causa elegida: `spam`, `ofensivo`, `peligroso`, `menores` u `otro`.
    pub motivo: String,
    /// Explicacion libre del reportante, hasta 1000 caracteres.
    pub detalle: Option<String>,
}

/// Guarda un reporte y devuelve su identificador.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el token no es valido, si no hay exactamente un
/// objetivo, si el motivo no esta permitido, si el detalle es demasiado largo,
/// si el objetivo no existe o no es visible para la cuenta, o si ya existe un
/// reporte de esa cuenta para el mismo contenido.
#[server(endpoint = "reportar")]
pub async fn reportar(token: String, datos: ReporteNuevo) -> Result<String, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let reportante = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| ServerFnError::new("La sesion caduco; vuelve a entrar"))?;

    // Exactamente un objetivo: la tabla tiene la misma restriccion.
    let objetivo = match (datos.receta_id.is_some(), datos.comentario_id.is_some()) {
        (true, false) => "receta",
        (false, true) => "comentario",
        _ => {
            return Err(ServerFnError::new("Elige una sola cosa para reportar"));
        }
    };

    if !MOTIVOS.contains(&datos.motivo.as_str()) {
        return Err(ServerFnError::new("Ese motivo no esta permitido"));
    }

    let detalle = datos
        .detalle
        .as_deref()
        .map(str::trim)
        .filter(|texto| !texto.is_empty());

    if detalle.is_some_and(|texto| texto.chars().count() > DETALLE_MAXIMO) {
        return Err(ServerFnError::new(
            "El detalle no puede pasar de 1000 caracteres",
        ));
    }

    let visible = match objetivo {
        "receta" => sqlx::query_scalar::<_, bool>(
            r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM receta r
                    WHERE r.id = $1::uuid
                      AND (
                            (r.publicada_en IS NOT NULL AND NOT r.oculta)
                            OR r.autor_id = $2::uuid
                          )
                )
                "#,
        )
        .bind(datos.receta_id.as_deref())
        .bind(&reportante)
        .fetch_one(pool)
        .await
        .map_err(|_| ServerFnError::new("Query error"))?,
        _ => sqlx::query_scalar::<_, bool>(
            r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM comentario c
                    JOIN receta r ON r.id = c.receta_id
                    WHERE c.id = $1::uuid
                      AND NOT c.oculto
                      AND (
                            (r.publicada_en IS NOT NULL AND NOT r.oculta)
                            OR r.autor_id = $2::uuid
                          )
                )
                "#,
        )
        .bind(datos.comentario_id.as_deref())
        .bind(&reportante)
        .fetch_one(pool)
        .await
        .map_err(|_| ServerFnError::new("Query error"))?,
    };

    if !visible {
        return Err(ServerFnError::new(
            "Ese contenido no existe o ya no esta disponible",
        ));
    }

    let repetido = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM reporte
            WHERE reportante_id = $1::uuid
              AND (
                    ($2::uuid IS NOT NULL AND receta_id = $2::uuid)
                    OR ($3::uuid IS NOT NULL AND comentario_id = $3::uuid)
                  )
        )
        "#,
    )
    .bind(&reportante)
    .bind(datos.receta_id.as_deref())
    .bind(datos.comentario_id.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    if repetido {
        return Err(ServerFnError::new("Ya reportaste este contenido"));
    }

    let reporte_id = sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO reporte (id, reportante_id, receta_id, comentario_id, motivo, detalle)
        VALUES (gen_random_uuid(), $1::uuid, $2::uuid, $3::uuid, $4, $5)
        RETURNING id::text
        "#,
    )
    .bind(&reportante)
    .bind(datos.receta_id.as_deref())
    .bind(datos.comentario_id.as_deref())
    .bind(&datos.motivo)
    .bind(detalle)
    .fetch_one(pool)
    .await
    .map_err(|_| ServerFnError::new("No se pudo guardar el reporte"))?;

    Ok(reporte_id)
}
