//! Sugerencias del día de la pantalla Descubrir (B1) del módulo Recetario (compartido y servidor)
//!
//! Los tipos viajan entre la app y el servidor; las consultas a Supabase solo compilan en el servidor

// LIBRERÍAS
use dioxus::prelude::*; // Macro server y tipo ServerFnError (única importación con comodín permitida)
use serde::Deserialize; // Derive para recibir la sugerencia en la app
use serde::Serialize; // Derive para enviar la sugerencia desde el servidor
#[cfg(feature = "server")]
use crate::db; // Pool de conexiones a PostgreSQL en Supabase (solo servidor)

// ESTRUCTURAS
/// Receta sugerida para la tarjeta del día, lista para dibujarse con el motor de proporciones
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaSugerida
{
    /// Identificador de la receta (UUID en texto)
    pub id: String,
    /// Tipo de bebida ("cafe" o "cocteleria")
    pub tipo: String,
    /// Nombre, vaso, capacidad y capas en el formato del motor de proporciones
    pub receta: proporciones::Receta,
}

/// Fila de una receta con las medidas de su vaso, tal como sale de la consulta (solo servidor)
#[cfg(feature = "server")]
struct FilaReceta
{
    /// Identificador de la receta (UUID en texto)
    id: String,
    /// Nombre visible de la bebida
    nombre: String,
    /// Tipo de bebida ("cafe" o "cocteleria")
    tipo: String,
    /// Volumen servido (ml, el del vaso si la receta no fija uno)
    capacidad_ml: f64,
    /// Volumen útil del vaso hasta el borde interior (ml)
    vaso_capacidad_ml: f64,
    /// Media anchura de la boca del vaso
    radio_boca: f64,
    /// Media anchura del fondo del vaso (0 = cono)
    radio_fondo: f64,
    /// Altura interior del vaso
    altura: f64,
    /// Grosor de la pared del vaso
    pared: f64,
    /// Labio sobre el nivel máximo del líquido
    borde: f64,
    /// Altura del tallo en copas (0 = sin tallo)
    tallo: f64,
    /// Tamaño del asa ("chica", "grande" o ninguna)
    asa: Option<String>,
    /// Si se dibuja plato bajo la taza
    plato: bool,
}

/// Fila de una capa líquida de receta, tal como sale de la consulta (solo servidor)
#[cfg(feature = "server")]
struct FilaCapa
{
    /// Receta a la que pertenece la capa (UUID en texto)
    receta_id: String,
    /// Nombre del ingrediente en esta receta
    nombre: String,
    /// Cantidad del ingrediente (ml)
    cantidad: f64,
    /// Alcohol por volumen (0-100)
    graduacion_pct: f64,
    /// Color de la capa en hex (ninguno = color de la paleta)
    color: Option<String>,
    /// Si es agua o hielo derretido
    dilucion: bool,
}

/// Conversión de las filas de la base al formato del motor de proporciones (solo servidor)
#[cfg(feature = "server")]
impl FilaReceta
{
    /// Convierte la fila y sus capas en una receta sugerida, con las capas de abajo hacia arriba
    fn a_sugerencia(self, capas_lista: &[FilaCapa]) -> RecetaSugerida
    {
        // Elegir el asa según el texto guardado en la base
        let asa_val = match self.asa.as_deref()
        {
            Some("chica") => Some(proporciones::Asa::Chica), // Asa chica
            Some("grande") => Some(proporciones::Asa::Grande), // Asa grande
            _ => None, // Vaso sin asa
        };

        // Construir el vaso con las mismas medidas que usa el motor
        let vaso_val = proporciones::Vaso
        {
            capacidad_ml: self.vaso_capacidad_ml,
            radio_boca: self.radio_boca,
            radio_fondo: self.radio_fondo,
            altura: self.altura,
            pared: self.pared,
            borde: self.borde,
            tallo: self.tallo,
            asa: asa_val,
            plato: self.plato,
        };

        // Convertir las capas de esta receta en componentes del motor (ya vienen ordenadas desde el fondo)
        let ingredientes_vec = capas_lista
            .iter()
            .filter(|capa_iter| capa_iter.receta_id == self.id) // Conservar solo las capas de esta receta
            .map(|capa_iter| proporciones::Componente
            {
                nombre: capa_iter.nombre.clone(),
                ml: capa_iter.cantidad,
                graduacion: capa_iter.graduacion_pct,
                dilucion: capa_iter.dilucion,
                color: capa_iter.color.clone(),
                superficie: None,
            })
            .collect();

        RecetaSugerida
        {
            id: self.id,
            tipo: self.tipo,
            receta: proporciones::Receta
            {
                nombre: self.nombre,
                vaso: vaso_val,
                capacidad_ml: self.capacidad_ml,
                ingredientes: ingredientes_vec,
            },
        } // Retornar sugerencia lista para dibujar
    }
}

// FUNCIONES
/// Consulta una receta publicada por cada tipo de bebida, rotando según el día en Tijuana
///
/// Omite borradores, recetas ocultas y, si la persona no declaró mayoría de edad, las recetas
/// con alguna capa con alcohol. Devuelve el vaso y las capas para que la app dibuje sin otra
/// petición. El café sale primero y la coctelería después.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool de conexiones o si alguna consulta falla
#[server(endpoint = "sugerencias_dia")]
pub async fn sugerencias_dia(token: Option<String>) -> Result<Vec<RecetaSugerida>, ServerFnError>
{
    // Obtener el pool que instaló el binario del servidor
    let pool_val = match db::pool()
    {
        Some(pool_val) => pool_val,
        None =>
        {
            // Sin pool: avisar que la base no está disponible
            return Err(ServerFnError::new("Database unavailable")); // Terminar sin consultar
        }
    };

    // La mayoría de edad sale del perfil que corresponde al token: sin sesión, se trata como menor
    // Consultar una receta visible por tipo con orden determinista por día, sin alcohol para menores
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    let recetas_vec = sqlx::query_as!(FilaReceta,
        r#"
        SELECT DISTINCT ON (r.tipo)
            r.id::text AS "id!",
            r.nombre AS "nombre!",
            r.tipo AS "tipo!",
            COALESCE(r.capacidad_ml, v.capacidad_ml) AS "capacidad_ml!",
            v.capacidad_ml AS "vaso_capacidad_ml!",
            v.radio_boca AS "radio_boca!",
            v.radio_fondo AS "radio_fondo!",
            v.altura AS "altura!",
            v.pared AS "pared!",
            v.borde AS "borde!",
            v.tallo AS "tallo!",
            v.asa,
            v.plato AS "plato!"
        FROM receta r
        JOIN vaso v ON v.id = r.vaso_id
        WHERE r.publicada_en IS NOT NULL
        AND NOT r.oculta
        AND ($1 OR NOT EXISTS (SELECT 1 FROM componente_receta c WHERE c.receta_id = r.id AND c.graduacion_pct > 0))
        ORDER BY r.tipo, md5(r.id::text || (now() AT TIME ZONE 'America/Tijuana')::date::text)
        "#,
        adulto // Mayoría de edad verificada contra Supabase Auth y el perfil
    )
        .fetch_all(pool_val) // Ejecutar consulta con el pool compartido
        .await
        .map_err(|_| ServerFnError::new("Query error"))?; // Propagar error de consulta

    // Reunir los identificadores de las recetas elegidas para pedir sus capas
    let ids_vec: Vec<String> = recetas_vec
        .iter()
        .map(|fila_iter| fila_iter.id.clone())
        .collect();

    // Consultar las capas líquidas de las recetas elegidas desde el fondo (los sólidos en mg no ocupan volumen en el vaso)
    let capas_vec = sqlx::query_as!(FilaCapa,
        r#"
        SELECT receta_id::text AS "receta_id!", nombre, cantidad, graduacion_pct, color, dilucion
        FROM componente_receta
        WHERE receta_id::text = ANY($1)
        AND unidad = 'ml'
        ORDER BY receta_id, posicion
        "#,
        &ids_vec[..] // Recetas elegidas (UUID en texto)
    )
        .fetch_all(pool_val) // Ejecutar consulta con el pool compartido
        .await
        .map_err(|_| ServerFnError::new("Query error"))?; // Propagar error de consulta

    // Convertir cada fila en una sugerencia con sus capas
    let sugerencias_vec = recetas_vec
        .into_iter()
        .map(|fila_iter| fila_iter.a_sugerencia(&capas_vec))
        .collect();

    Ok(sugerencias_vec) // Retornar sugerencias del día
}
