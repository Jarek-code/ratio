//! Ficha de una receta (B3): lo que la app necesita para dibujarla y explicarla.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db;

/// Paso de preparación con su temporizador opcional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasoFicha {
    /// Encabezado corto del paso.
    pub titulo: String,
    /// Texto completo del paso.
    pub instruccion: String,
    /// Duración del temporizador en segundos, si el paso lo necesita.
    pub temporizador_s: Option<i32>,
}

/// Receta completa para la ficha: el vaso y las capas en el formato del motor,
/// más los datos que la pantalla muestra alrededor del dibujo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaCompleta {
    /// Identificador de la receta (UUID en texto).
    pub id: String,
    /// Pestaña a la que pertenece ("cafe" o "cocteleria").
    pub tipo: String,
    /// Familia de la bebida (B1, B2).
    pub familia: String,
    /// Alias de quien la publicó.
    pub autor: String,
    /// Nombre del vaso en el catálogo.
    pub vaso_nombre: String,
    /// Vaso y capas listos para el motor de proporciones.
    pub receta: proporciones::Receta,
    /// Pasos en orden, del primero al último.
    pub pasos: Vec<PasoFicha>,
    /// Tiempo de preparación declarado (min).
    pub minutos: Option<i16>,
    /// Dificultad declarada (1-5).
    pub dificultad: Option<i16>,
    /// Intensidad declarada (1-5).
    pub intensidad: Option<i16>,
    /// Notas de sabor de la ficha.
    pub notas: Vec<String>,
}

/// Fila de la consulta de cabecera, con las medidas del vaso (solo servidor).
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
    /// Alias del autor.
    autor: String,
    /// Nombre del vaso.
    vaso_nombre: String,
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
    /// Notas de sabor.
    notas: Vec<String>,
}

/// Fila de una capa líquida (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaCapa {
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

/// Fila de un paso de preparación (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaPaso {
    /// Encabezado corto.
    titulo: Option<String>,
    /// Texto del paso.
    instruccion: String,
    /// Temporizador en segundos.
    temporizador_s: Option<i32>,
}

#[cfg(feature = "server")]
impl FilaReceta {
    /// Convierte la cabecera y sus capas en la receta que consume el motor.
    fn a_completa(self, capas: Vec<FilaCapa>, pasos: Vec<PasoFicha>) -> RecetaCompleta {
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
            .into_iter()
            .map(|capa| proporciones::Componente {
                nombre: capa.nombre,
                ml: capa.cantidad,
                graduacion: capa.graduacion_pct,
                dilucion: capa.dilucion,
                color: capa.color,
                superficie: None,
            })
            .collect();

        RecetaCompleta {
            id: self.id,
            tipo: self.tipo,
            familia: self.familia,
            autor: self.autor,
            vaso_nombre: self.vaso_nombre,
            receta: proporciones::Receta {
                nombre: self.nombre,
                vaso,
                capacidad_ml: self.capacidad_ml,
                ingredientes,
            },
            pasos,
            minutos: self.minutos,
            dificultad: self.dificultad,
            intensidad: self.intensidad,
            notas: self.notas,
        }
    }
}

/// Trae una receta publicada con su vaso, sus capas y sus pasos.
///
/// Devuelve `None` si no existe, si es un borrador o si está oculta: la ficha solo
/// enseña lo que se puede ver en La Barra.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si alguna consulta falla.
#[server(endpoint = "receta")]
pub async fn completa(id: String) -> Result<Option<RecetaCompleta>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cabecera = sqlx::query_as::<_, FilaReceta>(
        r#"
        SELECT r.id::text AS id,
               r.nombre,
               r.tipo,
               r.familia,
               u.alias AS autor,
               v.nombre AS vaso_nombre,
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
               r.notas
        FROM receta r
        JOIN vaso v ON v.id = r.vaso_id
        JOIN usuario u ON u.id = r.autor_id
        WHERE r.id = $1::uuid
          AND r.publicada_en IS NOT NULL
          AND NOT r.oculta
        "#,
    )
    .bind(&id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let Some(cabecera) = cabecera else {
        return Ok(None);
    };

    let capas = sqlx::query_as::<_, FilaCapa>(
        r#"
        SELECT nombre, cantidad, graduacion_pct, color, dilucion
        FROM componente_receta
        WHERE receta_id = $1::uuid
          AND unidad = 'ml'
        ORDER BY posicion
        "#,
    )
    .bind(&id)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let pasos = sqlx::query_as::<_, FilaPaso>(
        r#"
        SELECT titulo, instruccion, temporizador_s
        FROM paso
        WHERE receta_id = $1::uuid
        ORDER BY posicion
        "#,
    )
    .bind(&id)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let pasos = pasos
        .into_iter()
        .map(|paso| PasoFicha {
            titulo: paso.titulo.unwrap_or_default(),
            instruccion: paso.instruccion,
            temporizador_s: paso.temporizador_s,
        })
        .collect();

    Ok(Some(cabecera.a_completa(capas, pasos)))
}
