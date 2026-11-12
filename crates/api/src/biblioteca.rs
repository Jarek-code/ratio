//! Catálogo de recetas publicadas para la pantalla Biblioteca (B2).

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db;

/// Capa tal como vive en la base, con su ingrediente y su unidad.
///
/// Se envia junto al resumen y a la ficha para que C2 pueda guardar la version sin
/// adivinar IDs y para que C7 pueda agrupar por ingrediente y separar los solidos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponenteFicha {
    /// Ingrediente del catalogo al que apunta la capa.
    pub ingrediente_id: String,
    /// Orden de la capa dentro de la receta (0 = fondo).
    pub posicion: i16,
    /// Nombre copiado dentro de esta receta.
    pub nombre: String,
    /// Unidad de la cantidad: "ml" o "mg".
    pub unidad: String,
    /// Cantidad en la unidad indicada.
    pub cantidad: f64,
    /// Alcohol por volumen (0-100).
    pub graduacion_pct: f64,
    /// Color de la capa en hex, si lo tiene.
    pub color: Option<String>,
    /// Si la capa es agua o hielo derretido.
    pub dilucion: bool,
}

/// Receta publicada con lo necesario para listarla y dibujar su miniatura.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaResumen {
    /// Identificador de la receta (UUID en texto).
    pub id: String,
    /// Pestaña a la que pertenece ("cafe" o "cocteleria").
    pub tipo: String,
    /// Familia de la bebida, para los filtros de la biblioteca.
    pub familia: String,
    /// Alias de quien creo la receta.
    pub autor: String,
    /// Vaso y capas listos para el motor de proporciones.
    pub receta: proporciones::Receta,
    /// Capas con ingrediente y unidad, para C2 y C7.
    pub componentes: Vec<ComponenteFicha>,
    /// Tiempo de preparación declarado (min).
    pub minutos: Option<i16>,
    /// Dificultad declarada (1-5).
    pub dificultad: Option<i16>,
    /// Intensidad declarada (1-5).
    pub intensidad: Option<i16>,
}

/// Fila de la consulta de recetas, con las medidas de su vaso (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
pub(crate) struct FilaReceta {
    /// Identificador de la receta (UUID en texto).
    id: String,
    /// Nombre visible de la bebida.
    nombre: String,
    /// Pestaña de la bebida.
    tipo: String,
    /// Familia de la bebida.
    familia: String,
    /// Alias de quien creo la receta.
    autor: String,
    /// Volumen servido (ml, el del vaso si la receta no fija uno).
    capacidad_ml: f64,
    /// Medidas del vaso, leídas de las columnas del vaso.
    #[sqlx(flatten)]
    vaso: crate::filas::MedidasVaso,
    /// Tiempo de preparación (min).
    minutos: Option<i16>,
    /// Dificultad (1-5).
    dificultad: Option<i16>,
    /// Intensidad (1-5).
    intensidad: Option<i16>,
}

/// Capa compartida con las demás consultas (solo servidor).
#[cfg(feature = "server")]
pub(crate) use crate::filas::FilaCapa;

#[cfg(feature = "server")]
impl FilaReceta {
    /// Convierte la fila y sus capas en el resumen que consume la lista.
    pub(crate) fn a_resumen(self, capas: &[FilaCapa]) -> RecetaResumen {
        let capas: Vec<&FilaCapa> = capas
            .iter()
            .filter(|capa| capa.receta_id == self.id)
            .collect();

        let ingredientes = capas.iter().map(|capa| capa.a_componente()).collect();
        let componentes = capas.iter().map(|capa| capa.a_ficha()).collect();

        RecetaResumen {
            id: self.id,
            tipo: self.tipo,
            familia: self.familia,
            autor: self.autor,
            receta: proporciones::Receta {
                nombre: self.nombre,
                vaso: self.vaso.a_vaso(),
                capacidad_ml: self.capacidad_ml,
                ingredientes,
            },
            componentes,
            minutos: self.minutos,
            dificultad: self.dificultad,
            intensidad: self.intensidad,
        }
    }
}

/// Trae todas las recetas publicadas, de la A a la Z, con su vaso y sus capas.
///
/// Sin una sesión que haya declarado mayoría de edad se omiten las recetas con alcohol.
///
/// La búsqueda y los filtros se resuelven en el cliente: el catálogo del piloto son unas
/// pocas decenas de recetas y así el buscador responde sin una petición por tecla.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si alguna consulta falla.
#[server(endpoint = "catalogo")]
pub async fn catalogo(token: Option<String>) -> Result<Vec<RecetaResumen>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    let recetas = sqlx::query_as::<_, FilaReceta>(
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
               r.intensidad
        FROM receta r
        JOIN vaso v ON v.id = r.vaso_id
        JOIN usuario u ON u.id = r.autor_id
        WHERE r.publicada_en IS NOT NULL
          AND NOT r.oculta
          AND ($1 OR NOT EXISTS (
                SELECT 1 FROM componente_receta alcohol
                WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
              ))
        ORDER BY r.nombre
        "#,
    )
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
        JOIN receta r ON r.id = c.receta_id
        WHERE r.publicada_en IS NOT NULL
          AND NOT r.oculta
          AND ($1 OR NOT EXISTS (
                SELECT 1 FROM componente_receta alcohol
                WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
              ))
        ORDER BY c.receta_id, c.posicion
        "#,
    )
    .bind(adulto)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    Ok(recetas
        .into_iter()
        .map(|fila| fila.a_resumen(&capas))
        .collect())
}
