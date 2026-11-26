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

/// Ingrediente nuevo propuesto por una cuenta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngredienteNuevo {
    /// Nombre visible, de 2 a 60 caracteres.
    pub nombre: String,
    /// Categoría libre y corta: café, licor, zumo, especia…
    pub tipo: Option<String>,
    /// Unidad de medida: `ml` o `mg`.
    pub unidad: String,
    /// Alcohol por volumen (0-100).
    pub graduacion_pct: f64,
    /// Color de la capa en hex `#RRGGBB`; se usa el de reserva si falta.
    pub color: Option<String>,
    /// Si diluye la bebida (agua o hielo derretido).
    pub dilucion: bool,
}

/// Agrega un ingrediente al catálogo y lo devuelve listo para usar.
///
/// El catálogo es común: lo que agrega una cuenta lo pueden usar todas. Se acepta cualquier
/// cuenta con sesión, y el nombre repetido devuelve el ingrediente que ya existe para no
/// llenar el catálogo de duplicados.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si los datos no son válidos.
#[server(endpoint = "crear_ingrediente")]
pub async fn crear_ingrediente(
    token: String,
    datos: IngredienteNuevo,
) -> Result<IngredienteOpcion, ServerFnError> {
    let pool = crate::base::pool()?;

    crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| crate::fallos::sesion("Inicia sesión para agregar un ingrediente."))?;

    let nombre = datos.nombre.trim();
    if nombre.chars().count() < 2 || nombre.chars().count() > 60 {
        return Err(crate::fallos::datos(
            "El nombre del ingrediente debe tener entre 2 y 60 caracteres.",
        ));
    }

    if !matches!(datos.unidad.as_str(), "ml" | "mg") {
        return Err(crate::fallos::datos("La unidad debe ser ml o mg."));
    }

    if !(0.0..=100.0).contains(&datos.graduacion_pct) || !datos.graduacion_pct.is_finite() {
        return Err(crate::fallos::datos(
            "La graduación tiene que estar entre 0 y 100.",
        ));
    }

    let color = match datos.color.as_deref().map(str::trim) {
        Some(valor) if valor.is_empty() => None,
        Some(valor) if crate::fallos::es_color(valor) => Some(valor.to_uppercase()),
        Some(_) => {
            return Err(crate::fallos::datos(
                "El color tiene que ser un valor como #7A4A2B.",
            ));
        }
        None => None,
    };

    let tipo = datos
        .tipo
        .as_deref()
        .map(str::trim)
        .filter(|valor| !valor.is_empty() && valor.chars().count() <= 30)
        .map(str::to_string);

    // Si ya existe por nombre, se devuelve ese: el catálogo no debe repetirse
    let existente = sqlx::query_as::<_, FilaIngrediente>(
        r#"
        SELECT id::text AS id, nombre, unidad, graduacion_pct, color, es_dilucion AS dilucion
        FROM ingrediente
        WHERE lower(nombre) = lower($1)
        LIMIT 1
        "#,
    )
    .bind(nombre)
    .fetch_optional(pool)
    .con_plazo("el catálogo de ingredientes")
    .await?;

    if let Some(fila) = existente {
        return Ok(IngredienteOpcion {
            id: fila.id,
            nombre: fila.nombre,
            unidad: fila.unidad,
            graduacion_pct: fila.graduacion_pct,
            color: fila.color,
            dilucion: fila.dilucion,
        });
    }

    let fila = sqlx::query_as::<_, FilaIngrediente>(
        r#"
        INSERT INTO ingrediente (nombre, tipo, unidad, graduacion_pct, color, es_dilucion)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id::text AS id, nombre, unidad, graduacion_pct, color, es_dilucion AS dilucion
        "#,
    )
    .bind(nombre)
    .bind(&tipo)
    .bind(&datos.unidad)
    .bind(datos.graduacion_pct)
    .bind(&color)
    .bind(datos.dilucion)
    .fetch_one(pool)
    .con_plazo("el alta del ingrediente")
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo crear el ingrediente: {causa}");

        crate::fallos::reintento("No se pudo agregar el ingrediente. Vuelve a intentarlo.")
    })?;

    Ok(IngredienteOpcion {
        id: fila.id,
        nombre: fila.nombre,
        unidad: fila.unidad,
        graduacion_pct: fila.graduacion_pct,
        color: fila.color,
        dilucion: fila.dilucion,
    })
}
