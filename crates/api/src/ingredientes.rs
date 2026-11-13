//! Catálogo de ingredientes para añadir capas nuevas desde la pantalla Ajustar (C1).
//!
//! El editor necesita el nombre, la unidad, la graduación, el color y la dilución del
//! catálogo para que una capa añadida se dibuje o se mida igual que las de la receta.
//! Se devuelven tanto líquidos (ml) como sólidos (mg); de momento los mg se convierten
//! a volumen con la equivalencia temporal `1 g = 1 ml`.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta

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
    let pool = crate::base::pool()?;

    let filas = sqlx::query_as::<_, FilaIngrediente>(
        r#"
        SELECT id::text AS id,
               nombre,
               unidad,
               graduacion_pct,
               color,
               es_dilucion AS dilucion
        FROM ingrediente
        ORDER BY nombre
        "#,
    )
    .fetch_all(pool)
    .con_plazo("el catálogo de ingredientes")
    .await?;

    Ok(filas
        .into_iter()
        .map(|fila| IngredienteOpcion {
            id: fila.id,
            nombre: fila.nombre,
            unidad: fila.unidad,
            graduacion_pct: fila.graduacion_pct,
            color: fila.color,
            dilucion: fila.dilucion,
        })
        .collect())
}
