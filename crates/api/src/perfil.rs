//! Perfiles (E2 y E3): datos públicos de una cuenta y seguimiento, parte compartida.
//!
//! El perfil muestra el alias, las recetas publicadas y los contadores de seguidores. Las
//! recetas salen del mismo catálogo que Biblioteca, así que respetan la misma regla de edad.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

use crate::biblioteca::RecetaResumen; // Receta publicada tal como la lista Biblioteca

#[cfg(feature = "server")]
use crate::db; // Pool compartido de la base

// ESTRUCTURAS

/// Perfil público de una cuenta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfilPublico
{
    /// Alias visible.
    pub alias: String,
    /// Si el perfil es el de la cuenta que pregunta.
    pub propio: bool,
    /// Recetas publicadas que la cuenta que pregunta puede ver.
    pub recetas: Vec<RecetaResumen>,
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
struct FilaPerfil
{
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
pub async fn perfil(alias: String, token: Option<String>) -> Result<Option<PerfilPublico>, ServerFnError>
{
    let Some(pool) = db::pool() else
    {
        return Err(ServerFnError::new("Database unavailable"));
    };
    let cuenta = crate::auth::cuenta(token.as_deref()).await;

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
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let Some(fila) = fila else
    {
        return Ok(None); // Alias inexistente
    };

    // Tomar del catálogo, con su regla de edad, las recetas publicadas por este alias
    let alias_min = fila.alias.to_lowercase();
    let recetas = crate::biblioteca::catalogo(token)
        .await?
        .into_iter()
        .filter(|receta| receta.autor.to_lowercase() == alias_min)
        .collect();

    Ok(Some(PerfilPublico
    {
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
pub async fn seguir(token: String, alias: String, sigue: bool) -> Result<bool, ServerFnError>
{
    let Some(pool) = db::pool() else
    {
        return Err(ServerFnError::new("Database unavailable"));
    };
    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| ServerFnError::new("Sign-in required"))?;

    // Buscar la cuenta del alias, que no puede ser la propia
    let seguido = sqlx::query_scalar::<_, String>(
        "SELECT id::text FROM usuario WHERE lower(alias) = lower($1) AND id <> $2::uuid",
    )
    .bind(alias.trim())
    .bind(&cuenta)
    .fetch_optional(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?
    .ok_or_else(|| ServerFnError::new("Profile not found"))?;

    // Seguir: insertar si no existe; dejar de seguir: borrar si existe
    let consulta = if sigue
    {
        "INSERT INTO seguimiento (seguidor_id, seguido_id) VALUES ($1::uuid, $2::uuid) ON CONFLICT DO NOTHING"
    }
    else
    {
        "DELETE FROM seguimiento WHERE seguidor_id = $1::uuid AND seguido_id = $2::uuid"
    };
    sqlx::query(consulta)
        .bind(&cuenta)
        .bind(&seguido)
        .execute(pool)
        .await
        .map_err(|_| ServerFnError::new("Could not update follow"))?;

    Ok(sigue) // Estado final del seguimiento
}
