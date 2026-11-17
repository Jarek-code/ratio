//! Notificaciones de La Barra (D6): avisos guardados en `notificacion`, parte compartida.
//!
//! Cada escritura que le importa a otra cuenta (un comentario, una respuesta, una valoración,
//! una versión publicada o la decisión de un reporte) deja una fila en `notificacion` dentro
//! de su misma transacción. Actividad lee esas filas y marca cuáles no se han visto.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta

// CONSTANTES Y VARIABLES GLOBALES

/// Novedades que trae Actividad.
#[cfg(feature = "server")]
const NOVEDADES_MAXIMAS: i64 = 50;

// ESTRUCTURAS

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
    /// Moderación resolvió un reporte propio o actuó sobre un contenido propio.
    Moderacion,
}

/// Novedad sobre las recetas, los comentarios o los reportes de la cuenta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evento
{
    /// Qué pasó.
    pub tipo: TipoEvento,
    /// Alias de quien lo hizo; «Moderación» en las decisiones de moderación.
    pub actor: String,
    /// Receta a la que lleva la novedad; `None` si la cuenta ya no puede abrirla.
    pub receta_id: Option<String>,
    /// Nombre de esa receta.
    pub receta_nombre: String,
    /// Detalle: el texto del comentario, «Le salió» o la decisión de moderación.
    pub detalle: String,
    /// Fecha legible, por ejemplo «12 nov 2026».
    pub fecha: String,
    /// Si la cuenta todavía no la había visto.
    pub nueva: bool,
}

/// Aviso que se guarda para otra cuenta (solo servidor).
#[cfg(feature = "server")]
pub(crate) struct Aviso<'a>
{
    /// Cuenta que recibe el aviso.
    pub(crate) para: &'a str,
    /// Cuenta que hizo la acción.
    pub(crate) actor: &'a str,
    /// Tipo del aviso, igual que el CHECK de la tabla.
    pub(crate) tipo: &'static str,
    /// Receta a la que lleva.
    pub(crate) receta_id: &'a str,
    /// Comentario del que trata, si trata de uno.
    pub(crate) comentario_id: Option<&'a str>,
}

/// Fila de una novedad de Actividad (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaEvento
{
    /// Tipo guardado en la tabla.
    tipo: String,
    /// Alias de quien la hizo.
    actor: String,
    /// Receta a la que lleva.
    receta_id: String,
    /// Nombre de la receta.
    receta_nombre: String,
    /// Si la cuenta todavía puede abrir la receta.
    receta_visible: bool,
    /// Texto del comentario, si la novedad trata de uno.
    comentario: Option<String>,
    /// Si ese comentario responde a otro.
    es_respuesta: bool,
    /// Última valoración de quien hizo la acción.
    me_salio: Option<bool>,
    /// Nombre de la receta original, en una derivada.
    origen_nombre: Option<String>,
    /// Acción de moderación a la que corresponde el aviso.
    accion: Option<String>,
    /// Nota que dejó moderación.
    nota: Option<String>,
    /// Si la cuenta es la autora del contenido moderado.
    soy_autor: bool,
    /// Si el aviso trata de un comentario y no de la receta.
    sobre_comentario: bool,
    /// Si no se ha visto.
    nueva: bool,
    /// Día de la novedad.
    dia: i32,
    /// Mes de la novedad (1-12).
    mes: i32,
    /// Año de la novedad.
    anio: i32,
}

// FUNCIONES

/// Cuenta que debe recibir el aviso, o `None` si quien actúa es el propio dueño.
///
/// Nadie se notifica a sí mismo: comentar la receta propia o responderse no avisa.
pub fn destinatario<'a>(dueno: &'a str, actor: &str) -> Option<&'a str>
{
    (!dueno.is_empty() && !dueno.eq_ignore_ascii_case(actor)).then_some(dueno)
}

/// Texto de una decisión de moderación para quien reportó o para quien publicó el contenido.
pub fn texto_moderacion(accion: Option<&str>, nota: Option<&str>, soy_autor: bool, sobre_comentario: bool) -> String
{
    let que = if sobre_comentario { "comentario" } else { "receta" };
    let base = match (accion, soy_autor)
    {
        (Some("ocultar"), true) => format!("Retiramos tu {que} de La Barra porque no cumple las normas."),
        (Some("restaurar"), true) => format!("Restauramos tu {que}: vuelve a estar visible."),
        (Some("avisar"), true) => format!("Recibimos un reporte sobre tu {que}. Revísalo, por favor."),
        (Some("ocultar"), false) => "Revisamos tu reporte y retiramos el contenido.".to_string(),
        (Some("avisar"), false) => "Revisamos tu reporte y avisamos a quien lo publicó.".to_string(),
        (Some("descartar"), false) => "Revisamos tu reporte: el contenido cumple las normas y se mantiene.".to_string(),
        _ => "Moderación revisó este contenido.".to_string(),
    };

    // La nota de moderación solo la lee quien publicó el contenido
    match nota.map(str::trim).filter(|texto| !texto.is_empty())
    {
        Some(texto) if soy_autor => format!("{base} Nota: {texto}"),
        _ => base,
    }
}

/// Condición que deja fuera las novedades que la cuenta ya no puede ver.
///
/// Usa los alias `n` (notificación), `r` (receta) y `c` (comentario), `$1` como la cuenta y
/// `$2` como la mayoría de edad. Una decisión de moderación se muestra aunque el contenido
/// esté retirado, porque de eso trata el aviso.
#[cfg(feature = "server")]
fn visible_sql() -> String
{
    format!(
        r#"
        ($2 OR {sin_alcohol})
        AND (
            n.tipo = 'reporte_resuelto'
            OR (
                NOT r.oculta
                AND (r.publicada_en IS NOT NULL OR r.autor_id = $1::uuid)
                AND c.oculto IS NOT TRUE
            )
        )
        "#,
        sin_alcohol = crate::foro::sin_alcohol("r"),
    )
}

/// Guarda un aviso para otra cuenta dentro de la transacción de la escritura que lo causa.
///
/// Un aviso sin comentario (valoración, versión o decisión sobre una receta) reemplaza al
/// anterior del mismo tipo, actor y receta, para que repetir la acción no llene Actividad.
///
/// # Errores
///
/// Devuelve `ServerFnError` si la base rechaza la escritura.
#[cfg(feature = "server")]
pub(crate) async fn notificar(conexion: &mut sqlx::PgConnection, aviso: Aviso<'_>) -> Result<(), ServerFnError>
{
    // Quitar el aviso anterior equivalente cuando no trata de un comentario concreto
    if aviso.comentario_id.is_none()
    {
        sqlx::query(
            r#"
            DELETE FROM notificacion
            WHERE usuario_id = $1::uuid
              AND actor_id = $2::uuid
              AND tipo = $3
              AND receta_id = $4::uuid
              AND comentario_id IS NULL
            "#,
        )
        .bind(aviso.para)
        .bind(aviso.actor)
        .bind(aviso.tipo)
        .bind(aviso.receta_id)
        .execute(&mut *conexion)
        .con_plazo("el aviso anterior")
        .await?;
    }

    // Insertar el aviso nuevo, sin leer
    sqlx::query(
        r#"
        INSERT INTO notificacion (usuario_id, actor_id, tipo, receta_id, comentario_id)
        VALUES ($1::uuid, $2::uuid, $3, $4::uuid, $5::uuid)
        "#,
    )
    .bind(aviso.para)
    .bind(aviso.actor)
    .bind(aviso.tipo)
    .bind(aviso.receta_id)
    .bind(aviso.comentario_id)
    .execute(&mut *conexion)
    .con_plazo("el aviso")
    .await?;

    Ok(())
}

/// Trae las últimas 50 novedades de la cuenta, de la más reciente a la más antigua (D6).
///
/// Lee la tabla `notificacion`. No aparecen las novedades de recetas o comentarios que la
/// cuenta ya no puede ver (retirados, despublicados o con alcohol sin mayoría de edad).
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si la consulta falla.
#[server(endpoint = "actividad")]
pub async fn actividad(token: String) -> Result<Vec<Evento>, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Consultar los avisos con lo necesario para explicar cada uno
    let consulta = format!(
        r#"
        SELECT n.tipo,
               COALESCE(u.alias, 'Anónimo') AS actor,
               r.id::text AS receta_id,
               r.nombre AS receta_nombre,
               (NOT r.oculta AND (r.publicada_en IS NOT NULL OR r.autor_id = $1::uuid)) AS receta_visible,
               c.texto AS comentario,
               (c.respuesta_a IS NOT NULL) AS es_respuesta,
               prep.me_salio,
               o.nombre AS origen_nombre,
               ma.accion,
               ma.nota,
               (COALESCE(c.autor_id, r.autor_id) = $1::uuid) AS soy_autor,
               (n.comentario_id IS NOT NULL) AS sobre_comentario,
               (n.leida_en IS NULL) AS nueva,
               EXTRACT(DAY FROM n.creada_en)::int AS dia,
               EXTRACT(MONTH FROM n.creada_en)::int AS mes,
               EXTRACT(YEAR FROM n.creada_en)::int AS anio
        FROM notificacion n
        JOIN receta r ON r.id = n.receta_id
        LEFT JOIN usuario u ON u.id = n.actor_id
        LEFT JOIN comentario c ON c.id = n.comentario_id
        LEFT JOIN receta o ON o.id = r.receta_origen_id
        LEFT JOIN LATERAL (
            SELECT p.me_salio
            FROM preparacion p
            WHERE n.tipo = 'valoracion' AND p.receta_id = n.receta_id AND p.usuario_id = n.actor_id
            ORDER BY p.creada_en DESC
            LIMIT 1
        ) prep ON true
        LEFT JOIN LATERAL (
            SELECT m.accion, m.nota
            FROM moderacion_accion m
            WHERE n.tipo = 'reporte_resuelto' AND m.objetivo_id = COALESCE(n.comentario_id, n.receta_id)
            ORDER BY abs(EXTRACT(EPOCH FROM m.creada_en - n.creada_en))
            LIMIT 1
        ) ma ON true
        WHERE n.usuario_id = $1::uuid
          AND {visible}
        ORDER BY n.creada_en DESC, n.id
        LIMIT $3
        "#,
        visible = visible_sql(),
    );
    let filas_vec = sqlx::query_as::<_, FilaEvento>(&consulta)
        .bind(&cuenta)
        .bind(adulto)
        .bind(NOVEDADES_MAXIMAS)
        .fetch_all(pool)
        .con_plazo("tu actividad")
        .await?;

    // Convertir cada fila en la novedad que pinta la app
    let eventos_vec = filas_vec
        .into_iter()
        .map(|fila|
        {
            let (tipo, detalle) = match fila.tipo.as_str()
            {
                "valoracion" => (
                    TipoEvento::Valoracion,
                    if fila.me_salio == Some(false) { "No le salió" } else { "Le salió" }.to_string(),
                ),
                "version_derivada" => (
                    TipoEvento::Derivada,
                    fila.origen_nombre.map(|nombre| format!("Versión de {nombre}")).unwrap_or_default(),
                ),
                "reporte_resuelto" => (
                    TipoEvento::Moderacion,
                    texto_moderacion(fila.accion.as_deref(), fila.nota.as_deref(), fila.soy_autor, fila.sobre_comentario),
                ),
                _ if fila.es_respuesta => (TipoEvento::Respuesta, fila.comentario.unwrap_or_default()),
                _ => (TipoEvento::Comentario, fila.comentario.unwrap_or_default()),
            };

            Evento
            {
                actor: if tipo == TipoEvento::Moderacion { "Moderación".to_string() } else { fila.actor },
                tipo,
                receta_id: fila.receta_visible.then_some(fila.receta_id),
                receta_nombre: fila.receta_nombre,
                detalle,
                fecha: crate::foro::fecha_legible(fila.dia, fila.mes, fila.anio),
                nueva: fila.nueva,
            }
        })
        .collect();

    Ok(eventos_vec)
}

/// Cuenta las novedades que la cuenta todavía no ha visto (D6).
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si la consulta falla.
#[server(endpoint = "no_leidas")]
pub async fn no_leidas(token: String) -> Result<u32, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Contar solo las novedades que Actividad mostraría
    let consulta = format!(
        r#"
        SELECT count(*)
        FROM notificacion n
        JOIN receta r ON r.id = n.receta_id
        LEFT JOIN comentario c ON c.id = n.comentario_id
        WHERE n.usuario_id = $1::uuid
          AND n.leida_en IS NULL
          AND {visible}
        "#,
        visible = visible_sql(),
    );
    let total = sqlx::query_scalar::<_, i64>(&consulta)
        .bind(&cuenta)
        .bind(adulto)
        .fetch_one(pool)
        .con_plazo("tus novedades")
        .await?;

    Ok(u32::try_from(total).unwrap_or(u32::MAX)) // Un recuento nunca es negativo
}

/// Marca como vistas todas las novedades de la cuenta (D6).
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si la escritura falla.
#[server(endpoint = "marcar_leidas")]
pub async fn marcar_leidas(token: String) -> Result<(), ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;

    sqlx::query("UPDATE notificacion SET leida_en = now() WHERE usuario_id = $1::uuid AND leida_en IS NULL")
        .bind(&cuenta)
        .execute(pool)
        .con_plazo("tus novedades")
        .await?;

    Ok(())
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas
{
    // LIBRERÍAS

    use super::destinatario; // Cuenta que recibe el aviso
    use super::texto_moderacion; // Texto de una decisión de moderación

    // FUNCIONES

    /// Comprueba que nadie se notifica a sí mismo y que el dueño ajeno sí recibe el aviso.
    #[test]
    fn nadie_se_notifica_a_si_mismo()
    {
        let ana = "11111111-1111-4111-8111-111111111111";
        let luis = "22222222-2222-4222-8222-222222222222";

        assert_eq!(destinatario(ana, ana), None);
        assert_eq!(destinatario(ana, &ana.to_uppercase()), None, "uuid case must not matter");
        assert_eq!(destinatario(ana, luis), Some(ana));
        assert_eq!(destinatario("", luis), None, "no owner, no notice");
    }

    /// Comprueba que la nota de moderación solo llega a quien publicó el contenido.
    #[test]
    fn nota_solo_para_el_autor()
    {
        let autor = texto_moderacion(Some("avisar"), Some("Cuida el tono"), true, true);
        let reportante = texto_moderacion(Some("avisar"), Some("Cuida el tono"), false, true);

        assert!(autor.contains("tu comentario") && autor.ends_with("Nota: Cuida el tono"));
        assert!(!reportante.contains("Cuida el tono"));
        assert!(texto_moderacion(Some("ocultar"), None, true, false).contains("tu receta"));
        assert!(texto_moderacion(Some("descartar"), None, false, false).contains("se mantiene"));
    }
}
