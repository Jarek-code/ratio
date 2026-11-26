//! Perfiles (E2 y E3): datos públicos de una cuenta y seguimiento, parte compartida.
//!
//! El perfil muestra el alias, las recetas publicadas y los contadores de seguidores. Las
//! recetas salen de la misma consulta que el muro de La Barra, así que respetan la misma
//! regla de edad y traen sus contadores.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

use crate::barra::TarjetaBarra; // Receta publicada tal como la pinta el muro

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta

// ESTRUCTURAS

/// Perfil público de una cuenta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfilPublico {
    /// Alias visible.
    pub alias: String,
    /// Si el perfil es el de la cuenta que pregunta.
    pub propio: bool,
    /// Recetas publicadas que la cuenta que pregunta puede ver.
    pub recetas: Vec<TarjetaBarra>,
    /// Cuentas que siguen a este perfil.
    pub seguidores: i64,
    /// Cuentas a las que sigue este perfil.
    pub siguiendo: i64,
    /// Si la cuenta que pregunta ya lo sigue.
    pub lo_sigo: bool,
}

/// Fila con la cuenta del alias y sus contadores (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaPerfil {
    /// Alias tal como se guardó.
    alias: String,
    /// Si es la cuenta que pregunta.
    propio: bool,
    /// Seguidores del perfil.
    seguidores: i64,
    /// Cuentas que sigue el perfil.
    siguiendo: i64,
    /// Si la cuenta que pregunta lo sigue.
    lo_sigo: bool,
}

// FUNCIONES

/// Trae el perfil público de un alias, sin distinguir mayúsculas (E2 y E3).
///
/// Devuelve `None` si el alias no existe. Sin sesión, `lo_sigo` y `propio` son `false`.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si alguna consulta falla.
#[server(endpoint = "perfil")]
pub async fn perfil(
    alias: String,
    token: Option<String>,
) -> Result<Option<PerfilPublico>, ServerFnError> {
    if alias.trim().is_empty() {
        return Ok(None); // Sin alias no hay nada que buscar
    }

    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(token.as_deref()).await;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Consultar la cuenta del alias con sus contadores de seguimiento
    let fila = sqlx::query_as::<_, FilaPerfil>(
        r#"
        SELECT u.alias,
               COALESCE(u.id = $2::uuid, false) AS propio,
               (SELECT count(*) FROM seguimiento s WHERE s.seguido_id = u.id) AS seguidores,
               (SELECT count(*) FROM seguimiento s WHERE s.seguidor_id = u.id) AS siguiendo,
               EXISTS (
                   SELECT 1 FROM seguimiento s
                   WHERE s.seguido_id = u.id AND s.seguidor_id = $2::uuid
               ) AS lo_sigo
        FROM usuario u
        WHERE lower(u.alias) = lower($1)
        "#,
    )
    .bind(alias.trim())
    .bind(&cuenta)
    .fetch_optional(pool)
    .con_plazo("el perfil")
    .await?;

    let Some(fila) = fila else {
        return Ok(None); // Alias inexistente
    };

    // Traer, con la regla de edad, las recetas publicadas por este alias
    let recetas = crate::barra::tarjetas_de(pool, &cuenta, adulto, &fila.alias).await?;

    Ok(Some(PerfilPublico {
        alias: fila.alias,
        propio: fila.propio,
        recetas,
        seguidores: fila.seguidores,
        siguiendo: fila.siguiendo,
        lo_sigo: fila.lo_sigo,
    }))
}

/// Empieza o deja de seguir a un alias (E3). Devuelve si queda siguiéndolo.
///
/// Repetir la misma acción no cambia nada, así que se puede reenviar desde la cola.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si el alias no existe o es el propio, o si la
/// consulta falla.
#[server(endpoint = "seguir")]
pub async fn seguir(token: String, alias: String, sigue: bool) -> Result<bool, ServerFnError> {
    if alias.trim().is_empty() {
        return Err(crate::fallos::datos("Elige un perfil para seguir."));
    }

    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| crate::fallos::sesion("Entra con tu cuenta para seguir a alguien."))?;

    // Buscar la cuenta del alias, que no puede ser la propia
    let seguido = sqlx::query_scalar::<_, String>(
        "SELECT id::text FROM usuario WHERE lower(alias) = lower($1) AND id <> $2::uuid",
    )
    .bind(alias.trim())
    .bind(&cuenta)
    .fetch_optional(pool)
    .con_plazo("el perfil")
    .await?
    .ok_or_else(|| crate::fallos::no_encontrado("Ese perfil no existe."))?;

    // Seguir: insertar si no existe; dejar de seguir: borrar si existe
    let consulta = if sigue {
        "INSERT INTO seguimiento (seguidor_id, seguido_id) VALUES ($1::uuid, $2::uuid) ON CONFLICT DO NOTHING"
    } else {
        "DELETE FROM seguimiento WHERE seguidor_id = $1::uuid AND seguido_id = $2::uuid"
    };
    sqlx::query(consulta)
        .bind(&cuenta)
        .bind(&seguido)
        .execute(pool)
        .await
        .map_err(|causa| {
            dioxus::logger::tracing::error!("no se pudo actualizar el seguimiento: {causa}");

            crate::fallos::reintento("No se pudo actualizar el seguimiento. Vuelve a intentarlo.")
        })?;

    Ok(sigue) // Estado final del seguimiento
}

/// Datos de la cuenta que la persona puede cambiar desde Ajustes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuentaPropia {
    /// Alias visible en La Barra.
    pub alias: String,
    /// Si declaró mayoría de edad (habilita el contenido con alcohol).
    pub mayor_de_edad: bool,
}

/// Cambia el alias y la declaración de edad de la cuenta.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si el alias no es válido o si ya lo usa otra
/// cuenta.
#[server(endpoint = "actualizar_cuenta")]
pub async fn actualizar_cuenta(
    token: String,
    datos: CuentaPropia,
) -> Result<CuentaPropia, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| crate::fallos::sesion("Inicia sesión para cambiar tu cuenta."))?;

    let alias = datos.alias.trim().trim_start_matches('@');
    if alias.chars().count() < 2 || alias.chars().count() > 32 {
        return Err(crate::fallos::datos(
            "El alias debe tener entre 2 y 32 caracteres.",
        ));
    }

    let ocupado = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM usuario WHERE lower(alias) = lower($1) AND id <> $2::uuid)",
    )
    .bind(alias)
    .bind(&cuenta)
    .fetch_one(pool)
    .con_plazo("el alias")
    .await?;

    if ocupado {
        return Err(crate::fallos::datos("Ese alias ya lo usa otra cuenta."));
    }

    let fila = sqlx::query_as::<_, (String, bool)>(
        r#"
        UPDATE usuario
        SET alias = $2, mayor_de_edad = $3
        WHERE id = $1::uuid
        RETURNING alias, mayor_de_edad
        "#,
    )
    .bind(&cuenta)
    .bind(alias)
    .bind(datos.mayor_de_edad)
    .fetch_optional(pool)
    .con_plazo("la cuenta")
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo actualizar la cuenta: {causa}");

        crate::fallos::reintento("No se pudo guardar la cuenta. Vuelve a intentarlo.")
    })?;

    let (alias, mayor_de_edad) =
        fila.ok_or_else(|| crate::fallos::no_encontrado("Tu cuenta ya no existe."))?;

    Ok(CuentaPropia {
        alias,
        mayor_de_edad,
    })
}
