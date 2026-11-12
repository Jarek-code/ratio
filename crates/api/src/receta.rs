//! Ficha de una receta (B3): lo que la app necesita para dibujarla y explicarla.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biblioteca::ComponenteFicha;

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
    /// Posición del paso en la base; ancla los comentarios de La Barra.
    #[serde(default)]
    pub posicion: i16,
}

/// Receta de la que deriva una versión, para dar crédito al original (D5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrigenReceta {
    /// Identificador de la receta original (UUID en texto).
    pub id: String,
    /// Nombre de la receta original.
    pub nombre: String,
    /// Alias de quien la creó.
    pub autor: String,
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
    /// Identificador del vaso en el catálogo.
    pub vaso_id: String,
    /// Nombre del vaso en el catálogo.
    pub vaso_nombre: String,
    /// Vaso y capas listos para el motor de proporciones.
    pub receta: proporciones::Receta,
    /// Capas con ingrediente y unidad, para guardar una version desde C2.
    pub componentes: Vec<ComponenteFicha>,
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
    /// Receta de la que deriva, si es una versión y el original es visible para la cuenta.
    #[serde(default)]
    pub origen: Option<OrigenReceta>,
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
    /// Identificador del vaso.
    vaso_id: String,
    /// Nombre del vaso.
    vaso_nombre: String,
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
    /// Notas de sabor.
    notas: Vec<String>,
    /// Receta original, si es visible para la cuenta.
    origen_id: Option<String>,
    /// Nombre de la receta original.
    origen_nombre: Option<String>,
    /// Alias de quien creó el original.
    origen_autor: Option<String>,
}

/// Capa compartida con las demás consultas (solo servidor).
#[cfg(feature = "server")]
use crate::filas::FilaCapa;

/// Fila de un paso de preparación (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaPaso {
    /// Posición del paso en la receta.
    posicion: i16,
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
        let ingredientes = capas.iter().map(|capa| capa.a_componente()).collect();
        let componentes = capas.iter().map(|capa| capa.a_ficha()).collect();

        RecetaCompleta {
            id: self.id,
            tipo: self.tipo,
            familia: self.familia,
            autor: self.autor,
            vaso_id: self.vaso_id,
            vaso_nombre: self.vaso_nombre,
            receta: proporciones::Receta {
                nombre: self.nombre,
                vaso: self.vaso.a_vaso(),
                capacidad_ml: self.capacidad_ml,
                ingredientes,
            },
            componentes,
            pasos,
            minutos: self.minutos,
            dificultad: self.dificultad,
            intensidad: self.intensidad,
            notas: self.notas,
            origen: match (self.origen_id, self.origen_nombre) {
                (Some(id), Some(nombre)) => Some(OrigenReceta {
                    id,
                    nombre,
                    autor: self.origen_autor.unwrap_or_else(|| "Anónimo".to_string()),
                }),
                _ => None,
            },
        }
    }
}

/// Trae una receta con su vaso, sus capas y sus pasos.
///
/// Devuelve la receta si esta publicada o si pertenece a la cuenta del token. Los
/// borradores y las versiones privadas ajenas se tratan como inexistentes; una receta
/// oculta no se devuelve ni a su autor. Sin mayoría de edad declarada, una receta con
/// alcohol también se trata como inexistente.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si alguna consulta falla.
#[server(endpoint = "receta")]
pub async fn completa(
    id: String,
    token: Option<String>,
) -> Result<Option<RecetaCompleta>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cuenta = crate::auth::cuenta(token.as_deref()).await;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    let cabecera = sqlx::query_as::<_, FilaReceta>(
        r#"
        SELECT r.id::text AS id,
               r.nombre,
               r.tipo,
               r.familia,
               u.alias AS autor,
               v.id::text AS vaso_id,
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
               r.notas,
               o.id::text AS origen_id,
               o.nombre AS origen_nombre,
               ou.alias AS origen_autor
        FROM receta r
        JOIN vaso v ON v.id = r.vaso_id
        JOIN usuario u ON u.id = r.autor_id
        LEFT JOIN receta o ON o.id = r.receta_origen_id
          AND NOT o.oculta
          AND (o.publicada_en IS NOT NULL OR o.autor_id = $2::uuid)
          AND ($3 OR NOT EXISTS (
                SELECT 1 FROM componente_receta alcohol_origen
                WHERE alcohol_origen.receta_id = o.id AND alcohol_origen.graduacion_pct > 0
              ))
        LEFT JOIN usuario ou ON ou.id = o.autor_id
        WHERE r.id = $1::uuid
          AND NOT r.oculta
          AND (r.publicada_en IS NOT NULL OR r.autor_id = $2::uuid)
          AND ($3 OR NOT EXISTS (
                SELECT 1 FROM componente_receta alcohol
                WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
              ))
        "#,
    )
    .bind(&id)
    .bind(&cuenta)
    .bind(adulto)
    .fetch_optional(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let Some(cabecera) = cabecera else {
        return Ok(None);
    };

    let capas = sqlx::query_as::<_, FilaCapa>(
        r#"
        SELECT receta_id::text AS receta_id,
               ingrediente_id::text AS ingrediente_id,
               posicion,
               nombre,
               unidad,
               cantidad,
               graduacion_pct,
               color,
               dilucion
        FROM componente_receta
        WHERE receta_id = $1::uuid
        ORDER BY posicion
        "#,
    )
    .bind(&id)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let pasos = sqlx::query_as::<_, FilaPaso>(
        r#"
        SELECT posicion, titulo, instruccion, temporizador_s
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
            posicion: paso.posicion,
        })
        .collect();

    Ok(Some(cabecera.a_completa(capas, pasos)))
}
