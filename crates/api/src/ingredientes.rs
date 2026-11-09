//! Catálogo de ingredientes para añadir capas nuevas (C1) y para el glosario (B5).
//!
//! El editor necesita el nombre, la unidad, la graduación, el color y la dilución del
//! catálogo para que una capa añadida se dibuje o se mida igual que las de la receta.
//! B5 además usa `tipo` para agrupar el glosario por familia. Se devuelven tanto
//! líquidos (ml) como sólidos (mg); de momento los mg se convierten a volumen con la
//! equivalencia temporal `1 g = 1 ml`.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Una opción del catálogo cerrado de ingredientes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngredienteOpcion {
    /// Identificador del ingrediente en la base.
    pub id: String,
    /// Nombre visible en el menú desplegable.
    pub nombre: String,
    /// Unidad de la cantidad: "ml" o "mg".
    pub unidad: String,
    /// Alcohol por volumen (0-100).
    pub graduacion_pct: f64,
    /// Color de la capa en hex, si el catálogo lo define.
    pub color: Option<String>,
    /// Si el ingrediente diluye la bebida (agua o hielo derretido).
    pub dilucion: bool,
    /// Familia del ingrediente para agrupar el glosario de B5 (NULL = sin clasificar).
    pub tipo: Option<String>,
}

/// Fila de la consulta del catálogo (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaIngrediente {
    id: String,
    nombre: String,
    unidad: String,
    graduacion_pct: f64,
    color: Option<String>,
    dilucion: bool,
    tipo: Option<String>,
}

/// Trae los ingredientes del catálogo que C1 puede añadir a una capa.
///
/// La pantalla ya conoce las capas de la receta y filtra las que están en uso; el
/// servidor devuelve líquidos y sólidos ordenados para que el desplegable sea estable.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si la consulta falla.
#[server(endpoint = "ingredientes")]
pub async fn catalogo() -> Result<Vec<IngredienteOpcion>, ServerFnError> {
    let Some(pool) = crate::db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let filas = sqlx::query_as::<_, FilaIngrediente>(
        r#"
        SELECT id::text AS id,
               nombre,
               unidad,
               graduacion_pct,
               color,
               es_dilucion AS dilucion,
               tipo
        FROM ingrediente
        ORDER BY tipo NULLS LAST, nombre
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    Ok(filas
        .into_iter()
        .map(|fila| IngredienteOpcion {
            id: fila.id,
            nombre: fila.nombre,
            unidad: fila.unidad,
            graduacion_pct: fila.graduacion_pct,
            color: fila.color,
            dilucion: fila.dilucion,
            tipo: fila.tipo,
        })
        .collect())
}
