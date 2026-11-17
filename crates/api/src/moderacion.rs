//! Moderación de La Barra (D8): cola de reportes, decisiones e historial, parte compartida.
//!
//! Solo una cuenta con rol de moderación puede usar estas server functions; el rol se lee de
//! `usuario` en el servidor en cada llamada. Cada decisión se aplica en una transacción:
//! cambia la visibilidad del contenido, cierra sus reportes pendientes, queda registrada en
//! `moderacion_accion` y avisa a quien reportó y a quien publicó.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

use crate::Rol; // Rol de la cuenta
use crate::Sesion; // Cuenta que llama, con su rol

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta
#[cfg(feature = "server")]
use crate::foro::Motivo; // Causa de un reporte
#[cfg(feature = "server")]
use crate::notificaciones::destinatario; // Cuenta que recibe el aviso, si no es uno mismo
#[cfg(feature = "server")]
use crate::notificaciones::notificar; // Aviso dentro de la transacción
#[cfg(feature = "server")]
use crate::notificaciones::Aviso; // Datos del aviso

// CONSTANTES Y VARIABLES GLOBALES

/// Largo máximo de la nota de una decisión.
pub const NOTA_MAXIMA: usize = 500;

/// Reportes pendientes que trae la cola.
#[cfg(feature = "server")]
const REPORTES_MAXIMOS: i64 = 200;

/// Decisiones que trae el historial.
#[cfg(feature = "server")]
const DECISIONES_MAXIMAS: i64 = 50;

// ESTRUCTURAS

/// Clase de contenido sobre el que actúa moderación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoObjetivo
{
    /// Una receta publicada.
    Receta,
    /// Un comentario del hilo.
    Comentario,
}

impl TipoObjetivo
{
    /// Clave del tipo en la base.
    pub fn clave(self) -> &'static str
    {
        match self
        {
            TipoObjetivo::Receta => "receta",
            TipoObjetivo::Comentario => "comentario",
        }
    }

    /// Nombre visible del tipo.
    pub fn etiqueta(self) -> &'static str
    {
        match self
        {
            TipoObjetivo::Receta => "Receta",
            TipoObjetivo::Comentario => "Comentario",
        }
    }
}

/// Decisión que puede tomar moderación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accion
{
    /// Retirar el contenido y aceptar sus reportes.
    Ocultar,
    /// Volver a mostrar un contenido retirado.
    Restaurar,
    /// Dejar el contenido y cerrar sus reportes como infundados.
    Descartar,
    /// Dejar el contenido, aceptar sus reportes y avisar a quien lo publicó.
    Avisar,
}

impl Accion
{
    /// Clave de la acción en la base.
    pub fn clave(self) -> &'static str
    {
        match self
        {
            Accion::Ocultar => "ocultar",
            Accion::Restaurar => "restaurar",
            Accion::Descartar => "descartar",
            Accion::Avisar => "avisar",
        }
    }

    /// Acción que corresponde a una clave de la base; lo desconocido se lee como descartar.
    #[cfg(feature = "server")]
    fn de_clave(clave: &str) -> Accion
    {
        match clave
        {
            "ocultar" => Accion::Ocultar,
            "restaurar" => Accion::Restaurar,
            "avisar" => Accion::Avisar,
            _ => Accion::Descartar,
        }
    }

    /// Texto visible de la acción ya tomada.
    pub fn etiqueta(self) -> &'static str
    {
        match self
        {
            Accion::Ocultar => "Retirado",
            Accion::Restaurar => "Restaurado",
            Accion::Descartar => "Reportes descartados",
            Accion::Avisar => "Aviso enviado",
        }
    }
}

/// Contenido con reportes pendientes, tal como lo ve moderación.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReporteCola
{
    /// Si se reportó una receta o un comentario.
    pub tipo: TipoObjetivo,
    /// Identificador del contenido reportado (UUID en texto).
    pub objetivo_id: String,
    /// Receta reportada, o la receta del comentario reportado.
    pub receta_id: String,
    /// Nombre de esa receta.
    pub receta_nombre: String,
    /// Texto del comentario reportado; vacío si se reportó la receta.
    pub contenido: String,
    /// Alias de quien publicó el contenido.
    pub autor: String,
    /// Un texto por reporte: motivo, detalle y quién reportó.
    pub motivos: Vec<String>,
    /// Fecha del primer reporte, por ejemplo «12 nov 2026».
    pub fecha: String,
}

/// Decisión registrada en el historial de moderación.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision
{
    /// Qué se decidió.
    pub accion: Accion,
    /// Si la decisión fue sobre una receta o un comentario.
    pub tipo: TipoObjetivo,
    /// Identificador del contenido (UUID en texto).
    pub objetivo_id: String,
    /// Nombre de la receta o texto del comentario; vacío si el contenido ya no existe.
    pub contenido: String,
    /// Alias de quien moderó.
    pub moderador: String,
    /// Nota que dejó moderación.
    pub nota: Option<String>,
    /// Fecha de la decisión, por ejemplo «12 nov 2026».
    pub fecha: String,
    /// Si el contenido sigue retirado y se puede restaurar.
    pub retirado: bool,
}

/// Fila de un reporte pendiente (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaReporte
{
    /// Comentario reportado, si se reportó un comentario.
    comentario_id: Option<String>,
    /// Receta reportada o receta del comentario.
    receta_id: String,
    /// Nombre de esa receta.
    receta_nombre: String,
    /// Texto del comentario reportado.
    comentario: Option<String>,
    /// Alias de quien publicó el contenido.
    autor: String,
    /// Clave del motivo.
    motivo: String,
    /// Explicación del reporte.
    detalle: Option<String>,
    /// Alias de quien reportó.
    reportante: String,
    /// Día del reporte.
    dia: i32,
    /// Mes del reporte (1-12).
    mes: i32,
    /// Año del reporte.
    anio: i32,
}

/// Fila de una decisión del historial (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaDecision
{
    /// Clave de la acción.
    accion: String,
    /// Clave del tipo de contenido.
    objetivo_tipo: String,
    /// Identificador del contenido.
    objetivo_id: String,
    /// Nombre de la receta o texto del comentario.
    contenido: Option<String>,
    /// Alias de quien moderó.
    moderador: String,
    /// Nota de la decisión.
    nota: Option<String>,
    /// Si el contenido sigue retirado.
    retirado: bool,
    /// Día de la decisión.
    dia: i32,
    /// Mes de la decisión (1-12).
    mes: i32,
    /// Año de la decisión.
    anio: i32,
}

// FUNCIONES

/// Comprueba que la cuenta puede moderar: solo el rol de moderación pasa.
///
/// # Errores
///
/// Devuelve un fallo «no encontrado» para cualquier otro rol, sin confirmar que la sección existe.
pub fn autorizar(sesion: &Sesion) -> Result<(), ServerFnError>
{
    if sesion.rol == Rol::Moderador
    {
        Ok(())
    }
    else
    {
        Err(crate::fallos::no_encontrado("Esta sección es solo para el equipo de moderación."))
    }
}

/// Cuenta que llama, comprobando en la base que tiene rol de moderación.
#[cfg(feature = "server")]
async fn moderador(token: String) -> Result<Sesion, ServerFnError>
{
    let sesion = crate::auth::sesion(token).await?; // Verifica el token y lee el rol de `usuario`
    autorizar(&sesion)?;

    Ok(sesion)
}

/// Junta los reportes pendientes por contenido, en el orden en que llegaron.
#[cfg(feature = "server")]
fn agrupar(filas_vec: Vec<FilaReporte>) -> Vec<ReporteCola>
{
    let mut cola_vec: Vec<ReporteCola> = Vec::new(); // Un elemento por contenido reportado

    for fila in filas_vec
    {
        let objetivo_id = fila.comentario_id.clone().unwrap_or_else(|| fila.receta_id.clone());
        let motivo = Motivo::de_clave(&fila.motivo).map(Motivo::etiqueta).unwrap_or("Otro motivo");
        let detalle = fila.detalle.as_deref().map(str::trim).filter(|texto| !texto.is_empty());
        let texto = match detalle
        {
            Some(detalle) => format!("{motivo}: {detalle} (reportó {})", fila.reportante),
            None => format!("{motivo} (reportó {})", fila.reportante),
        };

        // Contenido ya visto: sumar el motivo; contenido nuevo: abrir su entrada
        match cola_vec.iter_mut().find(|entrada| entrada.objetivo_id == objetivo_id)
        {
            Some(entrada) => entrada.motivos.push(texto),
            None => cola_vec.push(ReporteCola
            {
                tipo: if fila.comentario_id.is_some() { TipoObjetivo::Comentario } else { TipoObjetivo::Receta },
                objetivo_id,
                receta_id: fila.receta_id,
                receta_nombre: fila.receta_nombre,
                contenido: fila.comentario.unwrap_or_default(),
                autor: fila.autor,
                motivos: vec![texto],
                fecha: crate::foro::fecha_legible(fila.dia, fila.mes, fila.anio),
            }),
        }
    }

    cola_vec // Contenidos con sus reportes
}

/// Trae los contenidos con reportes pendientes, del reporte más antiguo al más nuevo (D8).
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la cuenta no modera o si la consulta falla.
#[server(endpoint = "cola_reportes")]
pub async fn cola_reportes(token: String) -> Result<Vec<ReporteCola>, ServerFnError>
{
    let pool = crate::base::pool()?;
    moderador(token).await?;

    // Consultar los reportes pendientes con su contenido, su autor y quien reportó
    let filas_vec = sqlx::query_as::<_, FilaReporte>(
        r#"
        SELECT rp.comentario_id::text AS comentario_id,
               r.id::text AS receta_id,
               r.nombre AS receta_nombre,
               c.texto AS comentario,
               COALESCE(autor.alias, 'Anónimo') AS autor,
               rp.motivo,
               rp.detalle,
               COALESCE(reportante.alias, 'Anónimo') AS reportante,
               EXTRACT(DAY FROM rp.creado_en)::int AS dia,
               EXTRACT(MONTH FROM rp.creado_en)::int AS mes,
               EXTRACT(YEAR FROM rp.creado_en)::int AS anio
        FROM reporte rp
        LEFT JOIN comentario c ON c.id = rp.comentario_id
        JOIN receta r ON r.id = COALESCE(rp.receta_id, c.receta_id)
        LEFT JOIN usuario autor ON autor.id = COALESCE(c.autor_id, r.autor_id)
        LEFT JOIN usuario reportante ON reportante.id = rp.reportante_id
        WHERE rp.estado = 'pendiente'
        ORDER BY rp.creado_en, rp.id
        LIMIT $1
        "#,
    )
    .bind(REPORTES_MAXIMOS)
    .fetch_all(pool)
    .con_plazo("la cola de reportes")
    .await?;

    Ok(agrupar(filas_vec))
}

/// Aplica una decisión de moderación sobre una receta o un comentario (D8).
///
/// Ocultar y avisar aceptan los reportes pendientes del contenido; descartar los cierra como
/// infundados; restaurar vuelve a mostrar un contenido retirado. Todo ocurre en una sola
/// transacción, junto con el registro de la decisión y los avisos.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la cuenta no modera, si el contenido no
/// existe, si la decisión no aplica a su estado actual o si la escritura falla.
#[server(endpoint = "moderar")]
pub async fn moderar(
    token: String,
    tipo: TipoObjetivo,
    objetivo_id: String,
    accion: Accion,
    nota: Option<String>,
) -> Result<(), ServerFnError>
{
    if !crate::fallos::es_uuid(&objetivo_id)
    {
        return Err(crate::fallos::datos("Ese contenido no es válido: vuelve a abrir la cola."));
    }
    let nota = nota
        .as_deref()
        .map(str::trim)
        .filter(|texto| !texto.is_empty())
        .map(|texto| texto.chars().take(NOTA_MAXIMA).collect::<String>()); // Nota limpia y acotada
    if accion == Accion::Avisar && nota.is_none()
    {
        return Err(crate::fallos::datos("Escribe la nota del aviso: es lo que leerá quien publicó."));
    }

    let pool = crate::base::pool()?;
    let sesion = moderador(token).await?;
    let mut tx = pool.begin().con_plazo("la decisión").await?;

    // Leer el contenido: quién lo publicó, de qué receta es y si está retirado
    let lectura = match tipo
    {
        TipoObjetivo::Receta => "SELECT autor_id::text, id::text, oculta FROM receta WHERE id = $1::uuid FOR UPDATE",
        TipoObjetivo::Comentario => "SELECT autor_id::text, receta_id::text, oculto FROM comentario WHERE id = $1::uuid FOR UPDATE",
    };
    let (autor, receta_id, retirado) = sqlx::query_as::<_, (String, String, bool)>(lectura)
        .bind(&objetivo_id)
        .fetch_optional(&mut *tx)
        .con_plazo("el contenido reportado")
        .await?
        .ok_or_else(|| crate::fallos::no_encontrado("Ese contenido ya no existe."))?;

    // Cambiar la visibilidad cuando la decisión lo pide
    match (accion, retirado)
    {
        (Accion::Ocultar, true) => return Err(crate::fallos::datos("Ese contenido ya está retirado.")),
        (Accion::Restaurar, false) => return Err(crate::fallos::datos("Ese contenido no está retirado.")),
        (Accion::Ocultar | Accion::Restaurar, _) =>
        {
            let escritura = match tipo
            {
                TipoObjetivo::Receta => "UPDATE receta SET oculta = $2 WHERE id = $1::uuid",
                TipoObjetivo::Comentario => "UPDATE comentario SET oculto = $2 WHERE id = $1::uuid",
            };
            sqlx::query(escritura)
                .bind(&objetivo_id)
                .bind(accion == Accion::Ocultar)
                .execute(&mut *tx)
                .con_plazo("la visibilidad del contenido")
                .await?;
        }
        (Accion::Descartar | Accion::Avisar, _) => {} // El contenido se queda como está
    }

    // Cerrar los reportes pendientes del contenido; restaurar no toca reportes
    let reportes_vec: Vec<(String, String)> = if accion == Accion::Restaurar
    {
        Vec::new()
    }
    else
    {
        sqlx::query_as::<_, (String, String)>(
            r#"
            UPDATE reporte
            SET estado = $3, moderador_id = $2::uuid, resuelto_en = now()
            WHERE estado = 'pendiente'
              AND (($4 AND receta_id = $1::uuid) OR (NOT $4 AND comentario_id = $1::uuid))
            RETURNING id::text, reportante_id::text
            "#,
        )
        .bind(&objetivo_id)
        .bind(&sesion.usuario_id)
        .bind(if accion == Accion::Descartar { "descartado" } else { "aceptado" })
        .bind(tipo == TipoObjetivo::Receta)
        .fetch_all(&mut *tx)
        .con_plazo("los reportes del contenido")
        .await?
    };
    if accion == Accion::Descartar && reportes_vec.is_empty()
    {
        return Err(crate::fallos::datos("Ese contenido ya no tiene reportes pendientes."));
    }

    // Registrar la decisión
    sqlx::query(
        r#"
        INSERT INTO moderacion_accion (reporte_id, moderador_id, accion, objetivo_tipo, objetivo_id, nota)
        VALUES ($1::uuid, $2::uuid, $3, $4, $5::uuid, $6)
        "#,
    )
    .bind(reportes_vec.first().map(|(id, _)| id))
    .bind(&sesion.usuario_id)
    .bind(accion.clave())
    .bind(tipo.clave())
    .bind(&objetivo_id)
    .bind(&nota)
    .execute(&mut *tx)
    .con_plazo("el registro de la decisión")
    .await?;

    // Avisar a quien reportó y, salvo al descartar, a quien publicó el contenido
    let comentario_id = (tipo == TipoObjetivo::Comentario).then_some(objetivo_id.as_str());
    let mut avisados_vec: Vec<&str> = reportes_vec.iter().map(|(_, reportante)| reportante.as_str()).collect();
    if accion != Accion::Descartar
    {
        avisados_vec.push(&autor);
    }
    avisados_vec.sort_unstable();
    avisados_vec.dedup();
    for para in avisados_vec.into_iter().filter_map(|cuenta| destinatario(cuenta, &sesion.usuario_id))
    {
        notificar(
            &mut tx,
            Aviso { para, actor: &sesion.usuario_id, tipo: "reporte_resuelto", receta_id: &receta_id, comentario_id },
        )
        .await?;
    }

    tx.commit().con_plazo("la decisión").await?;

    Ok(())
}

/// Trae las últimas 50 decisiones de moderación, de la más reciente a la más antigua (D8).
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la cuenta no modera o si la consulta falla.
#[server(endpoint = "historial_moderacion")]
pub async fn historial_moderacion(token: String) -> Result<Vec<Decision>, ServerFnError>
{
    let pool = crate::base::pool()?;
    moderador(token).await?;

    // Consultar las decisiones con el contenido tal como está hoy
    let filas_vec = sqlx::query_as::<_, FilaDecision>(
        r#"
        SELECT m.accion,
               m.objetivo_tipo,
               m.objetivo_id::text AS objetivo_id,
               COALESCE(c.texto, r.nombre) AS contenido,
               COALESCE(u.alias, 'Anónimo') AS moderador,
               m.nota,
               COALESCE(c.oculto, r.oculta, false) AS retirado,
               EXTRACT(DAY FROM m.creada_en)::int AS dia,
               EXTRACT(MONTH FROM m.creada_en)::int AS mes,
               EXTRACT(YEAR FROM m.creada_en)::int AS anio
        FROM moderacion_accion m
        LEFT JOIN usuario u ON u.id = m.moderador_id
        LEFT JOIN comentario c ON m.objetivo_tipo = 'comentario' AND c.id = m.objetivo_id
        LEFT JOIN receta r ON m.objetivo_tipo = 'receta' AND r.id = m.objetivo_id
        ORDER BY m.creada_en DESC, m.id
        LIMIT $1
        "#,
    )
    .bind(DECISIONES_MAXIMAS)
    .fetch_all(pool)
    .con_plazo("el historial de moderación")
    .await?;

    // Convertir cada fila en la decisión que pinta la app
    let decisiones_vec = filas_vec
        .into_iter()
        .map(|fila| Decision
        {
            accion: Accion::de_clave(&fila.accion),
            tipo: if fila.objetivo_tipo == "comentario" { TipoObjetivo::Comentario } else { TipoObjetivo::Receta },
            objetivo_id: fila.objetivo_id,
            contenido: fila.contenido.unwrap_or_default(),
            moderador: fila.moderador,
            nota: fila.nota,
            fecha: crate::foro::fecha_legible(fila.dia, fila.mes, fila.anio),
            retirado: fila.retirado,
        })
        .collect();

    Ok(decisiones_vec)
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas
{
    // LIBRERÍAS

    use super::autorizar; // Puerta de la moderación
    use crate::Rol; // Roles de una cuenta
    use crate::Sesion; // Cuenta de prueba

    // FUNCIONES

    /// Arma una sesión de prueba con el rol dado.
    fn sesion(rol: Rol) -> Sesion
    {
        Sesion
        {
            usuario_id: "11111111-1111-4111-8111-111111111111".to_string(),
            alias: "ana".to_string(),
            rol,
            mayor_de_edad: true,
        }
    }

    /// Comprueba que solo el rol de moderación puede ocultar, restaurar o descartar.
    #[test]
    fn solo_modera_el_moderador()
    {
        assert!(autorizar(&sesion(Rol::Moderador)).is_ok());

        let rechazo = autorizar(&sesion(Rol::Usuario));
        assert!(rechazo.is_err(), "a regular account must not moderate");
        assert_eq!(
            rechazo.map_err(|causa| crate::fallos::codigo(&causa)),
            Err(crate::fallos::CODIGO_NO_ENCONTRADO)
        );
    }
}
