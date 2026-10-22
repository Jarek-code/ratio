//! Catálogo de recetas publicadas para la pantalla Biblioteca (B2).

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db;

/// Receta publicada con lo necesario para listarla y dibujar su miniatura.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaResumen {
    /// Identificador de la receta (UUID en texto).
    pub id: String,
    /// Pestaña a la que pertenece ("cafe" o "cocteleria").
    pub tipo: String,
    /// Familia de la bebida, para los filtros de la biblioteca.
    pub familia: String,
    /// Vaso y capas listos para el motor de proporciones.
    pub receta: proporciones::Receta,
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
struct FilaReceta {
    /// Identificador de la receta (UUID en texto).
    id: String,
    /// Nombre visible de la bebida.
    nombre: String,
    /// Pestaña de la bebida.
    tipo: String,
    /// Familia de la bebida.
    familia: String,
    /// Volumen servido (ml, el del vaso si la receta no fija uno).
    capacidad_ml: f64,
    /// Volumen útil del vaso (ml).
    vaso_capacidad_ml: f64,
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
    /// Tiempo de preparación (min).
    minutos: Option<i16>,
    /// Dificultad (1-5).
    dificultad: Option<i16>,
    /// Intensidad (1-5).
    intensidad: Option<i16>,
}

/// Fila de una capa líquida, con la receta a la que pertenece (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaCapa {
    /// Receta a la que pertenece la capa (UUID en texto).
    receta_id: String,
    /// Nombre del ingrediente en esta receta.
    nombre: String,
    /// Cantidad en mililitros.
    cantidad: f64,
    /// Alcohol por volumen (0-100).
    graduacion_pct: f64,
    /// Color de la capa en hex, si lo tiene.
    color: Option<String>,
    /// Si la capa es agua o hielo derretido.
    dilucion: bool,
}

#[cfg(feature = "server")]
impl FilaReceta {
    /// Convierte la fila y sus capas en el resumen que consume la lista.
    fn a_resumen(self, capas: &[FilaCapa]) -> RecetaResumen {
        let asa = match self.asa.as_deref() {
            Some("chica") => Some(proporciones::Asa::Chica),
            Some("grande") => Some(proporciones::Asa::Grande),
            _ => None,
        };

        let vaso = proporciones::Vaso {
            capacidad_ml: self.vaso_capacidad_ml,
            radio_boca: self.radio_boca,
            radio_fondo: self.radio_fondo,
            altura: self.altura,
            pared: self.pared,
            borde: self.borde,
            tallo: self.tallo,
            asa,
            plato: self.plato,
        };

        let ingredientes = capas
            .iter()
            .filter(|capa| capa.receta_id == self.id)
            .map(|capa| proporciones::Componente {
                nombre: capa.nombre.clone(),
                ml: capa.cantidad,
                graduacion: capa.graduacion_pct,
                dilucion: capa.dilucion,
                color: capa.color.clone(),
                superficie: None,
            })
            .collect();

        RecetaResumen {
            id: self.id,
            tipo: self.tipo,
            familia: self.familia,
            receta: proporciones::Receta {
                nombre: self.nombre,
                vaso,
                capacidad_ml: self.capacidad_ml,
                ingredientes,
            },
            minutos: self.minutos,
            dificultad: self.dificultad,
            intensidad: self.intensidad,
        }
    }
}

/// Trae todas las recetas publicadas, de la A a la Z, con su vaso y sus capas.
///
/// La búsqueda y los filtros se resuelven en el cliente: el catálogo del piloto son unas
/// pocas decenas de recetas y así el buscador responde sin una petición por tecla.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si alguna consulta falla.
#[server(endpoint = "catalogo")]
pub async fn catalogo() -> Result<Vec<RecetaResumen>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let recetas = sqlx::query_as::<_, FilaReceta>(
        r#"
        SELECT r.id::text AS id,
               r.nombre,
               r.tipo,
               r.familia,
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
        WHERE r.publicada_en IS NOT NULL
          AND NOT r.oculta
        ORDER BY r.nombre
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let capas = sqlx::query_as::<_, FilaCapa>(
        r#"
        SELECT c.receta_id::text AS receta_id, c.nombre, c.cantidad, c.graduacion_pct, c.color, c.dilucion
        FROM componente_receta c
        JOIN receta r ON r.id = c.receta_id
        WHERE c.unidad = 'ml'
          AND r.publicada_en IS NOT NULL
          AND NOT r.oculta
        ORDER BY c.receta_id, c.posicion
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    Ok(recetas
        .into_iter()
        .map(|fila| fila.a_resumen(&capas))
        .collect())
}
