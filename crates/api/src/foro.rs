//! La Barra (D1-D7): publicar, hilo, comentar, valorar, actividad y reportes, parte compartida.
//!
//! Todas las server functions respetan la misma regla de visibilidad que la ficha: una
//! receta oculta, privada ajena o con alcohol para una cuenta sin mayoría de edad declarada
//! no tiene hilo ni valoraciones, y tampoco se puede comentar ni valorar.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta

// CONSTANTES Y VARIABLES GLOBALES

/// Largo máximo de un comentario, igual que el CHECK de la tabla.
#[cfg(feature = "server")]
const LARGO_MAXIMO: usize = 2000;

// ESTRUCTURAS

/// Parte de la receta a la que se ancla un comentario, por su posición (0 = primera).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Ancla
{
    /// Capa de la receta, en el orden de la ficha.
    Capa(i16),
    /// Paso de la preparación, en el orden de la ficha.
    Paso(i16),
}

/// Comentario tal como se pinta en el hilo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comentario
{
    /// Identificador del comentario (UUID en texto).
    pub id: String,
    /// Alias de quien comenta.
    pub autor: String,
    /// Texto del comentario.
    pub texto: String,
    /// Fecha legible, por ejemplo «12 nov 2026».
    pub fecha: String,
    /// Comentario al que responde, si es una respuesta.
    pub respuesta_a: Option<String>,
    /// Texto del ancla, por ejemplo «Capa · Leche» o «Paso 2 · Espumar».
    pub ancla: Option<String>,
    /// Cantidad propuesta con su unidad, por ejemplo «40 ml» o «30 s».
    pub propuesta: Option<String>,
    /// Si lo escribió la cuenta que pregunta.
    pub propio: bool,
}

/// Comentario nuevo que manda la app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComentarioNuevo
{
    /// UUID que genera la app: reenviarlo desde la cola no duplica el comentario.
    pub id: Option<String>,
    /// Receta comentada.
    pub receta_id: String,
    /// Texto del comentario (1-2000 caracteres).
    pub texto: String,
    /// Comentario al que responde, si es una respuesta.
    pub respuesta_a: Option<String>,
    /// Capa o paso al que se ancla, si se ancla.
    pub ancla: Option<Ancla>,
    /// Cantidad propuesta para el ancla: en la unidad de la capa, o segundos para un paso.
    pub propuesta: Option<f64>,
}

/// Recuento de «me salió / no me salió» de una receta.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Valoraciones
{
    /// Personas a las que les salió igual a la ficha.
    pub me_salio: i64,
    /// Personas a las que no les salió.
    pub no_me_salio: i64,
    /// Valoración de la cuenta que pregunta, si ya valoró.
    pub mia: Option<bool>,
}

/// Fila de un comentario del hilo (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaComentario
{
    /// Identificador del comentario.
    id: String,
    /// Alias del autor.
    autor: String,
    /// Texto del comentario.
    texto: String,
    /// Comentario al que responde.
    respuesta_a: Option<String>,
    /// Nombre de la capa anclada.
    capa: Option<String>,
    /// Unidad de la capa anclada.
    unidad: Option<String>,
    /// Posición del paso anclado.
    paso_posicion: Option<i16>,
    /// Título del paso anclado.
    paso_titulo: Option<String>,
    /// Cantidad propuesta.
    propuesta: Option<f64>,
    /// Si lo escribió la cuenta que pregunta.
    propio: bool,
    /// Día de publicación.
    dia: i32,
    /// Mes de publicación (1-12).
    mes: i32,
    /// Año de publicación.
    anio: i32,
}

/// Fila del recuento de valoraciones (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaValoraciones
{
    /// Valoraciones positivas.
    me_salio: i64,
    /// Valoraciones negativas.
    no_me_salio: i64,
    /// Valoración de la cuenta, si existe.
    mia: Option<bool>,
}

// FUNCIONES

/// Indica si la receta es visible para la cuenta, con la misma regla que la ficha.
#[cfg(feature = "server")]
async fn visible(
    pool: &sqlx::PgPool,
    receta_id: &str,
    cuenta: &Option<String>,
    adulto: bool,
) -> Result<bool, ServerFnError>
{
    // Un id roto no puede llegar a `$1::uuid`: la app avisa de que el enlace no sirve
    if !crate::fallos::es_uuid(receta_id)
    {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde el recetario.",
        ));
    }

    // Consultar si la receta existe, no está oculta, es pública o propia y respeta la edad
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM receta r
            WHERE r.id = $1::uuid
              AND NOT r.oculta
              AND (r.publicada_en IS NOT NULL OR r.autor_id = $2::uuid)
              AND ($3 OR NOT EXISTS (
                    SELECT 1 FROM componente_receta alcohol
                    WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
                  ))
        )
        "#,
    )
    .bind(receta_id)
    .bind(cuenta)
    .bind(adulto)
    .fetch_one(pool)
    .con_plazo("la visibilidad de la receta")
    .await
}

/// Cuenta del token o el error que pide iniciar sesión.
#[cfg(feature = "server")]
async fn cuenta_obligatoria(token: &str) -> Result<String, ServerFnError>
{
    crate::auth::cuenta(Some(token))
        .await
        .ok_or_else(|| crate::fallos::sesion("Entra con tu cuenta para hacer esto.")) // Token ausente o caducado
}

/// Recuento de valoraciones de una receta, contando la última valoración de cada persona.
#[cfg(feature = "server")]
async fn recuento(
    pool: &sqlx::PgPool,
    receta_id: &str,
    cuenta: &Option<String>,
) -> Result<Valoraciones, ServerFnError>
{
    // Contar la valoración más reciente de cada persona
    let fila = sqlx::query_as::<_, FilaValoraciones>(
        r#"
        SELECT count(*) FILTER (WHERE ultima.me_salio) AS me_salio,
               count(*) FILTER (WHERE NOT ultima.me_salio) AS no_me_salio,
               bool_or(ultima.me_salio) FILTER (WHERE ultima.usuario_id = $2::uuid) AS mia
        FROM (
            SELECT DISTINCT ON (usuario_id) usuario_id, me_salio
            FROM preparacion
            WHERE receta_id = $1::uuid
            ORDER BY usuario_id, creada_en DESC
        ) ultima
        "#,
    )
    .bind(receta_id)
    .bind(cuenta)
    .fetch_one(pool)
    .con_plazo("el recuento de valoraciones")
    .await?;

    Ok(Valoraciones
    {
        me_salio: fila.me_salio,
        no_me_salio: fila.no_me_salio,
        mia: fila.mia,
    })
}

/// Publica una receta propia en La Barra o la retira (D1).
///
/// Devuelve si la receta quedó publicada. Una receta retirada por moderación no cambia.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la receta no es de la cuenta o si la
/// consulta falla.
#[server(endpoint = "publicar")]
pub async fn publicar(token: String, receta_id: String, publica: bool) -> Result<bool, ServerFnError>
{
    if !crate::fallos::es_uuid(&receta_id)
    {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde Mi recetario.",
        ));
    }

    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;

    // Cambiar la fecha de publicación solo si la receta es de la cuenta y no está oculta
    let publicada = sqlx::query_scalar::<_, bool>(
        r#"
        UPDATE receta
        SET publicada_en = CASE WHEN $3 THEN COALESCE(publicada_en, now()) ELSE NULL END
        WHERE id = $1::uuid
          AND autor_id = $2::uuid
          AND NOT oculta
        RETURNING publicada_en IS NOT NULL
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .bind(publica)
    .fetch_optional(pool)
    .con_plazo("la publicación")
    .await?;

    publicada.ok_or_else(|| crate::fallos::no_encontrado("Esa receta no existe o no es tuya.")) // No es suya o está oculta
}

/// Trae el hilo de comentarios de una receta, del más antiguo al más reciente (D2).
///
/// Sin permiso para ver la receta devuelve un hilo vacío. Los comentarios retirados por
/// moderación no se devuelven.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si la consulta falla.
#[server(endpoint = "hilo")]
pub async fn hilo(receta_id: String, token: Option<String>) -> Result<Vec<Comentario>, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(token.as_deref()).await;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Receta no visible: el hilo tampoco lo es
    if !visible(pool, &receta_id, &cuenta, adulto).await?
    {
        return Ok(Vec::new());
    }

    // Consultar los comentarios visibles con su autor y su ancla
    let filas_vec = sqlx::query_as::<_, FilaComentario>(
        r#"
        SELECT c.id::text AS id,
               COALESCE(u.alias, 'Anónimo') AS autor,
               c.texto,
               c.respuesta_a::text AS respuesta_a,
               capa.nombre AS capa,
               capa.unidad AS unidad,
               p.posicion AS paso_posicion,
               p.titulo AS paso_titulo,
               c.propuesta,
               COALESCE(c.autor_id = $2::uuid, false) AS propio,
               EXTRACT(DAY FROM c.creado_en)::int AS dia,
               EXTRACT(MONTH FROM c.creado_en)::int AS mes,
               EXTRACT(YEAR FROM c.creado_en)::int AS anio
        FROM comentario c
        LEFT JOIN usuario u ON u.id = c.autor_id
        LEFT JOIN componente_receta capa ON capa.id = c.componente_id
        LEFT JOIN paso p ON p.id = c.paso_id
        WHERE c.receta_id = $1::uuid
          AND NOT c.oculto
        ORDER BY c.creado_en
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .fetch_all(pool)
    .con_plazo("el hilo")
    .await?;

    // Convertir cada fila en el comentario que pinta la app
    let comentarios_vec = filas_vec
        .into_iter()
        .map(|fila|
        {
            let ancla = match (&fila.capa, fila.paso_posicion)
            {
                (Some(capa), _) => Some(format!("Capa · {capa}")), // Anclado a una capa
                (None, Some(posicion)) => Some(format!(
                    "Paso {} · {}",
                    posicion + 1,
                    fila.paso_titulo.clone().unwrap_or_default()
                )), // Anclado a un paso
                (None, None) => None, // Comentario general
            };
            let unidad = if fila.capa.is_some() { fila.unidad.clone().unwrap_or_default() } else { "s".to_string() };

            Comentario
            {
                id: fila.id,
                autor: fila.autor,
                texto: fila.texto,
                fecha: format!(
                    "{} {} {}",
                    fila.dia,
                    crate::historial::MESES[(fila.mes.clamp(1, 12) - 1) as usize],
                    fila.anio
                ),
                respuesta_a: fila.respuesta_a,
                propuesta: fila.propuesta.filter(|_| ancla.is_some()).map(|cantidad| format!("{cantidad:.0} {unidad}")),
                ancla,
                propio: fila.propio,
            }
        })
        .collect();

    Ok(comentarios_vec)
}

/// Publica un comentario en el hilo de una receta, anclado o no (D3).
///
/// Devuelve el identificador del comentario. Si la app reenvía el mismo `id` desde la cola,
/// no se crea otro.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si el texto está vacío o es demasiado largo,
/// si la receta no es visible para la cuenta, si el ancla no existe o si la consulta falla.
#[server(endpoint = "comentar")]
pub async fn comentar(token: String, nuevo: ComentarioNuevo) -> Result<String, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Validar el texto con el mismo límite que la tabla
    let texto = nuevo.texto.trim();
    if texto.is_empty() || texto.chars().count() > LARGO_MAXIMO
    {
        return Err(crate::fallos::datos(
            "El comentario necesita entre 1 y 2000 caracteres.",
        ));
    }

    // Validar la propuesta: positiva y solo con ancla
    if let Some(cantidad) = nuevo.propuesta
        && (!cantidad.is_finite() || cantidad <= 0.0 || nuevo.ancla.is_none())
    {
        return Err(crate::fallos::datos(
            "La cantidad propuesta tiene que ser un número mayor que cero.",
        ));
    }

    if !visible(pool, &nuevo.receta_id, &Some(cuenta.clone()), adulto).await?
    {
        return Err(crate::fallos::no_encontrado("Esa receta no existe o ya no está disponible."));
    }

    // Resolver el ancla por su posición dentro de la receta
    let (componente_id, paso_id) = match nuevo.ancla
    {
        Some(Ancla::Capa(posicion)) =>
        {
            let id = sqlx::query_scalar::<_, String>(
                "SELECT id::text FROM componente_receta WHERE receta_id = $1::uuid AND posicion = $2",
            )
            .bind(&nuevo.receta_id)
            .bind(posicion)
            .fetch_optional(pool)
            .con_plazo("la capa del comentario")
            .await?
            .ok_or_else(|| crate::fallos::datos("Esa capa ya no está en la receta: elige otra."))?;
            (Some(id), None) // Anclado a una capa
        }
        Some(Ancla::Paso(posicion)) =>
        {
            let id = sqlx::query_scalar::<_, String>(
                "SELECT id::text FROM paso WHERE receta_id = $1::uuid AND posicion = $2",
            )
            .bind(&nuevo.receta_id)
            .bind(posicion)
            .fetch_optional(pool)
            .con_plazo("el paso del comentario")
            .await?
            .ok_or_else(|| crate::fallos::datos("Ese paso ya no está en la receta: elige otro."))?;
            (None, Some(id)) // Anclado a un paso
        }
        None => (None, None), // Comentario general
    };

    // Insertar el comentario; un reenvío con el mismo id no hace nada
    let creado = sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO comentario (id, receta_id, autor_id, respuesta_a, texto, componente_id, paso_id, propuesta)
        VALUES (COALESCE($1::uuid, gen_random_uuid()), $2::uuid, $3::uuid, $4::uuid, $5, $6::uuid, $7::uuid, $8)
        ON CONFLICT (id) DO NOTHING
        RETURNING id::text
        "#,
    )
    .bind(&nuevo.id)
    .bind(&nuevo.receta_id)
    .bind(&cuenta)
    .bind(&nuevo.respuesta_a)
    .bind(texto)
    .bind(&componente_id)
    .bind(&paso_id)
    .bind(nuevo.propuesta)
    .fetch_optional(pool)
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo guardar el comentario: {causa}");

        crate::fallos::reintento("No se pudo guardar el comentario. Vuelve a intentarlo.")
    })?;

    creado
        .or(nuevo.id)
        .ok_or_else(|| crate::fallos::reintento("No se pudo guardar el comentario. Vuelve a intentarlo.")) // Id nuevo o el del reenvío
}

/// Trae el recuento de «me salió / no me salió» de una receta (D4).
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si la consulta falla.
#[server(endpoint = "valoraciones")]
pub async fn valoraciones(receta_id: String, token: Option<String>) -> Result<Valoraciones, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(token.as_deref()).await;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Receta no visible: sin valoraciones
    if !visible(pool, &receta_id, &cuenta, adulto).await?
    {
        return Ok(Valoraciones::default());
    }

    recuento(pool, &receta_id, &cuenta).await
}

/// Registra si a la cuenta le salió la receta; una nueva valoración reemplaza la anterior (D4).
///
/// Devuelve el recuento actualizado. Quien publicó la receta no puede valorarla.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la receta no es visible, si es propia o si
/// la consulta falla.
#[server(endpoint = "valorar")]
pub async fn valorar(token: String, receta_id: String, me_salio: bool) -> Result<Valoraciones, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    if !visible(pool, &receta_id, &Some(cuenta.clone()), adulto).await?
    {
        return Err(crate::fallos::no_encontrado("Esa receta no existe o ya no está disponible."));
    }

    // Rechazar la valoración de la propia receta
    let propia = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM receta WHERE id = $1::uuid AND autor_id = $2::uuid)",
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .fetch_one(pool)
    .con_plazo("la receta del comentario")
    .await?;
    if propia
    {
        return Err(crate::fallos::datos("No puedes valorar tu propia receta."));
    }

    // Actualizar la última valoración de la cuenta o crear la primera
    sqlx::query(
        r#"
        WITH previa AS (
            UPDATE preparacion
            SET me_salio = $3, creada_en = now()
            WHERE id = (
                SELECT id FROM preparacion
                WHERE usuario_id = $2::uuid AND receta_id = $1::uuid
                ORDER BY creada_en DESC
                LIMIT 1
            )
            RETURNING id
        )
        INSERT INTO preparacion (usuario_id, receta_id, me_salio)
        SELECT $2::uuid, $1::uuid, $3
        WHERE NOT EXISTS (SELECT 1 FROM previa)
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .bind(me_salio)
    .execute(pool)
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo guardar la valoración: {causa}");

        crate::fallos::reintento("No se pudo guardar tu valoración. Vuelve a intentarlo.")
    })?;

    recuento(pool, &receta_id, &Some(cuenta)).await
}

// ACTIVIDAD Y REPORTES (D6 y D7)

/// Tipo de novedad que aparece en Actividad.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TipoEvento
{
    /// Alguien comentó una receta propia.
    Comentario,
    /// Alguien respondió a un comentario propio.
    Respuesta,
    /// Alguien valoró una receta propia.
    Valoracion,
    /// Alguien publicó una versión derivada de una receta propia.
    Derivada,
}

/// Novedad sobre las recetas o los comentarios de la cuenta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evento
{
    /// Qué pasó.
    pub tipo: TipoEvento,
    /// Alias de quien lo hizo.
    pub actor: String,
    /// Receta a la que lleva la novedad (la derivada, en una derivación).
    pub receta_id: String,
    /// Nombre de esa receta.
    pub receta_nombre: String,
    /// Detalle: el texto del comentario o «me salió / no me salió».
    pub detalle: String,
    /// Fecha legible, por ejemplo «12 nov 2026».
    pub fecha: String,
}

/// Motivo de un reporte, igual que el CHECK de la tabla.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Motivo
{
    /// Publicidad o contenido repetido.
    Spam,
    /// Insultos o acoso.
    Ofensivo,
    /// Una preparación que pone en riesgo a alguien.
    Peligroso,
    /// Alcohol dirigido a menores de edad.
    Menores,
    /// Cualquier otra causa, explicada en el detalle.
    Otro,
}

impl Motivo
{
    /// Los motivos en el orden en que se ofrecen.
    pub const TODOS: [Motivo; 5] = [Motivo::Spam, Motivo::Ofensivo, Motivo::Peligroso, Motivo::Menores, Motivo::Otro];

    /// Clave del motivo en la base.
    pub fn clave(self) -> &'static str
    {
        match self
        {
            Motivo::Spam => "spam",
            Motivo::Ofensivo => "ofensivo",
            Motivo::Peligroso => "peligroso",
            Motivo::Menores => "menores",
            Motivo::Otro => "otro",
        }
    }

    /// Texto visible del motivo.
    pub fn etiqueta(self) -> &'static str
    {
        match self
        {
            Motivo::Spam => "Spam o publicidad",
            Motivo::Ofensivo => "Ofensivo",
            Motivo::Peligroso => "Peligroso",
            Motivo::Menores => "Alcohol para menores",
            Motivo::Otro => "Otro motivo",
        }
    }
}

/// Reporte de una receta o de un comentario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReporteNuevo
{
    /// Receta reportada, o `None` si se reporta un comentario.
    pub receta_id: Option<String>,
    /// Comentario reportado, o `None` si se reporta una receta.
    pub comentario_id: Option<String>,
    /// Causa elegida.
    pub motivo: Motivo,
    /// Explicación libre, opcional.
    pub detalle: Option<String>,
}

/// Fila de una novedad de Actividad (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaEvento
{
    /// Clave del tipo de novedad.
    tipo: String,
    /// Alias de quien la hizo.
    actor: String,
    /// Receta a la que lleva.
    receta_id: String,
    /// Nombre de la receta.
    receta_nombre: String,
    /// Texto del detalle.
    detalle: String,
    /// Día de la novedad.
    dia: i32,
    /// Mes de la novedad (1-12).
    mes: i32,
    /// Año de la novedad.
    anio: i32,
}

/// Trae las últimas 50 novedades sobre las recetas y los comentarios de la cuenta (D6).
///
/// Junta comentarios y valoraciones de otras personas en recetas propias, respuestas a
/// comentarios propios y versiones publicadas que derivan de recetas propias. Una derivada
/// con alcohol no aparece si la cuenta no declaró mayoría de edad.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si la consulta falla.
#[server(endpoint = "actividad")]
pub async fn actividad(token: String) -> Result<Vec<Evento>, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Juntar las cuatro clases de novedad y quedarse con las más recientes
    let filas_vec = sqlx::query_as::<_, FilaEvento>(
        r#"
        WITH eventos AS (
            SELECT 'comentario' AS tipo, c.autor_id AS actor_id, r.id AS receta_id, r.nombre AS receta_nombre,
                   c.texto AS detalle, c.creado_en AS fecha
            FROM comentario c
            JOIN receta r ON r.id = c.receta_id
            WHERE r.autor_id = $1::uuid AND c.autor_id <> $1::uuid AND c.respuesta_a IS NULL AND NOT c.oculto
            UNION ALL
            SELECT 'respuesta', c.autor_id, r.id, r.nombre, c.texto, c.creado_en
            FROM comentario c
            JOIN comentario padre ON padre.id = c.respuesta_a
            JOIN receta r ON r.id = c.receta_id
            WHERE padre.autor_id = $1::uuid AND c.autor_id <> $1::uuid AND NOT c.oculto
            UNION ALL
            SELECT 'valoracion', p.usuario_id, r.id, r.nombre,
                   CASE WHEN p.me_salio THEN 'Le salió' ELSE 'No le salió' END, p.creada_en
            FROM preparacion p
            JOIN receta r ON r.id = p.receta_id
            WHERE r.autor_id = $1::uuid AND p.usuario_id <> $1::uuid
            UNION ALL
            SELECT 'derivada', d.autor_id, d.id, d.nombre, 'Versión de ' || o.nombre, d.creada_en
            FROM receta d
            JOIN receta o ON o.id = d.receta_origen_id
            WHERE o.autor_id = $1::uuid AND d.autor_id <> $1::uuid
              AND d.publicada_en IS NOT NULL AND NOT d.oculta
              AND ($2 OR NOT EXISTS (
                    SELECT 1 FROM componente_receta alcohol
                    WHERE alcohol.receta_id = d.id AND alcohol.graduacion_pct > 0
                  ))
        )
        SELECT e.tipo,
               COALESCE(u.alias, 'Anónimo') AS actor,
               e.receta_id::text AS receta_id,
               e.receta_nombre,
               e.detalle,
               EXTRACT(DAY FROM e.fecha)::int AS dia,
               EXTRACT(MONTH FROM e.fecha)::int AS mes,
               EXTRACT(YEAR FROM e.fecha)::int AS anio
        FROM eventos e
        LEFT JOIN usuario u ON u.id = e.actor_id
        ORDER BY e.fecha DESC
        LIMIT 50
        "#,
    )
    .bind(&cuenta)
    .bind(adulto)
    .fetch_all(pool)
    .con_plazo("la respuesta del comentario")
    .await?;

    // Convertir cada fila en la novedad que pinta la app
    let eventos_vec = filas_vec
        .into_iter()
        .map(|fila| Evento
        {
            tipo: match fila.tipo.as_str()
            {
                "respuesta" => TipoEvento::Respuesta,
                "valoracion" => TipoEvento::Valoracion,
                "derivada" => TipoEvento::Derivada,
                _ => TipoEvento::Comentario,
            },
            actor: fila.actor,
            receta_id: fila.receta_id,
            receta_nombre: fila.receta_nombre,
            detalle: fila.detalle,
            fecha: format!(
                "{} {} {}",
                fila.dia,
                crate::historial::MESES[(fila.mes.clamp(1, 12) - 1) as usize],
                fila.anio
            ),
        })
        .collect();

    Ok(eventos_vec)
}

/// Reporta una receta o un comentario para la cola de moderación (D7).
///
/// Cada cuenta reporta una sola vez cada contenido: un segundo reporte no crea otro.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si no se indica exactamente un contenido, si
/// ese contenido no es visible para la cuenta o si la consulta falla.
#[server(endpoint = "reportar")]
pub async fn reportar(token: String, reporte: ReporteNuevo) -> Result<(), ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Averiguar la receta del contenido para comprobar que la cuenta puede verlo
    let receta_id = match (&reporte.receta_id, &reporte.comentario_id)
    {
        (Some(receta_id), None) => receta_id.clone(), // Se reporta una receta
        (None, Some(comentario_id)) if !crate::fallos::es_uuid(comentario_id) =>
        {
            return Err(crate::fallos::datos(
                "Ese comentario no es válido: vuelve a abrir la receta.",
            ));
        }
        (None, Some(comentario_id)) =>
        {
            sqlx::query_scalar::<_, String>(
                "SELECT receta_id::text FROM comentario WHERE id = $1::uuid AND NOT oculto",
            )
            .bind(comentario_id)
            .fetch_optional(pool)
            .con_plazo("el comentario reportado")
            .await?
            .ok_or_else(|| crate::fallos::no_encontrado("Ese comentario ya no está disponible."))?
        }
        _ => {
            return Err(crate::fallos::datos(
                "Reporta una sola cosa: la receta o un comentario.",
            ));
        }
    };

    if !visible(pool, &receta_id, &Some(cuenta.clone()), adulto).await?
    {
        return Err(crate::fallos::no_encontrado("Esa receta no existe o ya no está disponible."));
    }

    let detalle = reporte
        .detalle
        .as_deref()
        .map(str::trim)
        .filter(|texto| !texto.is_empty())
        .map(|texto| texto.chars().take(LARGO_MAXIMO).collect::<String>()); // Detalle limpio y acotado

    // Insertar el reporte salvo que la cuenta ya haya reportado ese contenido
    sqlx::query(
        r#"
        INSERT INTO reporte (reportante_id, receta_id, comentario_id, motivo, detalle)
        SELECT $1::uuid, $2::uuid, $3::uuid, $4, $5
        WHERE NOT EXISTS (
            SELECT 1 FROM reporte
            WHERE reportante_id = $1::uuid
              AND receta_id IS NOT DISTINCT FROM $2::uuid
              AND comentario_id IS NOT DISTINCT FROM $3::uuid
        )
        "#,
    )
    .bind(&cuenta)
    .bind(&reporte.receta_id)
    .bind(&reporte.comentario_id)
    .bind(reporte.motivo.clave())
    .bind(&detalle)
    .execute(pool)
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo guardar el reporte: {causa}");

        crate::fallos::reintento("No se pudo guardar el reporte. Vuelve a intentarlo.")
    })?;

    Ok(())
}

// FOTOS (D1)

/// Fotos que admite una receta, igual que el CHECK de la tabla.
pub const FOTOS_MAXIMAS: usize = 5;

/// Registra en la receta la ruta de una foto que la app ya subió a Storage (D1).
///
/// La foto debe estar en la carpeta `{cuenta}/{receta}/` del bucket, que es la única donde la
/// política de Storage deja subir a la cuenta. Devuelve las rutas de la receta; registrar dos
/// veces la misma ruta no la duplica.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la ruta no es de la cuenta y la receta, si la
/// receta no es suya, si ya tiene cinco fotos o si la consulta falla.
#[server(endpoint = "agregar_foto")]
pub async fn agregar_foto(token: String, receta_id: String, ruta: String) -> Result<Vec<String>, ServerFnError>
{
    if !crate::fallos::es_uuid(&receta_id)
    {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde Mi recetario.",
        ));
    }

    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;

    // Aceptar solo una ruta dentro de la carpeta de la cuenta y la receta
    let prefijo = format!("{cuenta}/{receta_id}/");
    let nombre = ruta.strip_prefix(&prefijo).unwrap_or_default();
    let nombre_valido = !nombre.is_empty()
        && nombre.len() <= 80
        && nombre.chars().all(|letra| letra.is_ascii_alphanumeric() || letra == '-' || letra == '.')
        && !nombre.contains("..");
    if !nombre_valido
    {
        return Err(crate::fallos::datos(
            "La ruta de la foto no es válida: tiene que estar dentro de tu carpeta.",
        ));
    }

    // Agregar la ruta si la receta es de la cuenta, no la tiene ya y no llegó al máximo
    let rutas = sqlx::query_scalar::<_, Vec<String>>(
        r#"
        UPDATE receta
        SET fotos_rutas = CASE
                WHEN $3 = ANY(fotos_rutas) THEN fotos_rutas
                ELSE array_append(fotos_rutas, $3)
            END
        WHERE id = $1::uuid
          AND autor_id = $2::uuid
          AND ($3 = ANY(fotos_rutas) OR cardinality(fotos_rutas) < $4)
        RETURNING fotos_rutas
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .bind(&ruta)
    .bind(FOTOS_MAXIMAS as i32)
    .fetch_optional(pool)
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo registrar la foto: {causa}");

        crate::fallos::reintento("No se pudo registrar la foto en la receta. Vuelve a intentarlo.")
    })?;

    rutas.ok_or_else(|| crate::fallos::datos("La receta no es tuya o ya tiene cinco fotos.")) // No es suya o ya tiene cinco
}
