//! Etiquetas que llegan de la API como texto fijo y aquí se escriben en el idioma elegido.
//!
//! El servidor no sabe qué idioma eligió la persona, así que manda el dato (el enum o el
//! número) y esta capa decide las palabras.

use api::foro::Motivo;
use api::moderacion::Accion;
use api::moderacion::TipoObjetivo;

use crate::i18n::Texto;
use crate::i18n::actual;
use crate::i18n::ahora;
use crate::i18n::con;

/// Duración en palabras: «45 s», «8 min» o «1 h 5 min».
pub fn duracion(segundos: i32) -> String {
    let segundos = segundos.max(0);

    match segundos {
        0..60 => con(actual(), Texto::DuracionSegundos, &[&segundos.to_string()]),
        60..3_600 => con(
            actual(),
            Texto::DuracionMinutos,
            &[&(segundos / 60).to_string()],
        ),
        _ => con(
            actual(),
            Texto::DuracionHoras,
            &[
                &(segundos / 3_600).to_string(),
                &(segundos % 3_600 / 60).to_string(),
            ],
        ),
    }
}

/// Motivo de un reporte, tal como se ofrece en el formulario.
pub fn motivo(motivo: Motivo) -> &'static str {
    match motivo {
        Motivo::Spam => ahora(Texto::MotivoSpam),
        Motivo::Ofensivo => ahora(Texto::MotivoOfensivo),
        Motivo::Peligroso => ahora(Texto::MotivoPeligroso),
        Motivo::Menores => ahora(Texto::MotivoMenores),
        Motivo::Otro => ahora(Texto::MotivoOtro),
    }
}

/// Qué se reportó: una receta o un comentario.
pub fn objetivo(tipo: TipoObjetivo) -> &'static str {
    match tipo {
        TipoObjetivo::Receta => ahora(Texto::ObjetivoReceta),
        TipoObjetivo::Comentario => ahora(Texto::ObjetivoComentario),
    }
}

/// Decisión de moderación, en pasado.
pub fn accion(accion: Accion) -> &'static str {
    match accion {
        Accion::Ocultar => ahora(Texto::AccionRetirado),
        Accion::Restaurar => ahora(Texto::AccionRestaurado),
        Accion::Descartar => ahora(Texto::AccionDescartado),
        Accion::Avisar => ahora(Texto::AccionAvisoEnviado),
    }
}

/// Propuesta de un comentario: «40 ml» o «30 s», con la unidad del idioma elegido.
pub fn propuesta(cantidad: f64, unidad: Option<&str>) -> String {
    match unidad {
        Some("s") => con(actual(), Texto::SegundosCorto, &[&format!("{cantidad:.0}")]),
        _ => con(
            actual(),
            Texto::MililitrosCorto,
            &[&format!("{cantidad:.0}")],
        ),
    }
}

/// Aviso que devuelve el servidor al guardar una versión.
pub fn aviso(aviso: api::guardar::Aviso) -> &'static str {
    match aviso {
        api::guardar::Aviso::AlLimiteDelVaso => ahora(Texto::AvisoAlLimite),
    }
}

/// Quién hizo la novedad: el alias o «Moderación», que pone la app.
pub fn actor(evento: &api::notificaciones::Evento) -> String {
    if evento.tipo == api::notificaciones::TipoEvento::Moderacion {
        ahora(Texto::ModeracionSeccion).to_string()
    } else {
        evento.actor.clone()
    }
}

/// Texto de una novedad: lo que escribió la persona o la frase del sistema.
pub fn detalle(evento: &api::notificaciones::Evento) -> String {
    match &evento.sistema {
        Some(sistema) => self::sistema(sistema),
        None => evento.detalle.clone(),
    }
}

/// Detalle de una novedad del sistema, en el idioma elegido.
pub fn sistema(detalle: &api::notificaciones::DetalleSistema) -> String {
    use api::notificaciones::DetalleSistema;

    match detalle {
        DetalleSistema::LeSalio => ahora(Texto::LeSalio).to_string(),
        DetalleSistema::NoLeSalio => ahora(Texto::NoLeSalio).to_string(),
        DetalleSistema::VersionDe(nombre) => con(actual(), Texto::VersionDeOrigen, &[nombre]),
        DetalleSistema::Moderacion {
            accion,
            nota,
            soy_autor,
            sobre_comentario,
        } => moderacion(
            accion.as_deref(),
            nota.as_deref(),
            *soy_autor,
            *sobre_comentario,
        ),
    }
}

/// Frase de una decisión de moderación, del lado de quien la recibe.
fn moderacion(
    accion: Option<&str>,
    nota: Option<&str>,
    soy_autor: bool,
    comentario: bool,
) -> String {
    let que = if comentario {
        ahora(Texto::ObjetivoComentario)
    } else {
        ahora(Texto::ObjetivoReceta)
    };

    let base = match (accion, soy_autor) {
        (Some("ocultar"), true) => con(actual(), Texto::ModeracionOcultarAutor, &[que]),
        (Some("restaurar"), true) => con(actual(), Texto::ModeracionRestaurarAutor, &[que]),
        (Some("avisar"), true) => con(actual(), Texto::ModeracionAvisoAutor, &[que]),
        (Some("ocultar"), false) => ahora(Texto::ModeracionOcultarReporte).to_string(),
        (Some("avisar"), false) => ahora(Texto::ModeracionAvisoReporte).to_string(),
        _ => ahora(Texto::ModeracionDescartado).to_string(),
    };

    // La nota que dejó moderación se añade tal cual
    match nota {
        Some(nota) if !nota.trim().is_empty() => {
            format!("{base} {}", con(actual(), Texto::ModeracionNota, &[nota]))
        }
        _ => base,
    }
}

/// Un reporte de la cola: «Spam: detalle (reportó alias)».
pub fn reporte_corto(reporte: &api::moderacion::ReporteCorto) -> String {
    let motivo = motivo(reporte.motivo);

    match reporte.detalle.as_deref() {
        Some(detalle) if !detalle.is_empty() => con(
            actual(),
            Texto::ReporteConDetalle,
            &[motivo, detalle, &reporte.reportante],
        ),
        _ => con(
            actual(),
            Texto::ReporteSinDetalle,
            &[motivo, &reporte.reportante],
        ),
    }
}

/// Ancla de un comentario: «Capa · Leche» o «Paso 2 · Espumar».
pub fn ancla(capa: &Option<String>, paso: Option<i16>, titulo: &Option<String>) -> Option<String> {
    match (capa, paso) {
        (Some(capa), _) => Some(con(actual(), Texto::AnclaCapa, &[capa])),
        (None, Some(posicion)) => Some(con(
            actual(),
            Texto::AnclaPaso,
            &[
                &(posicion + 1).to_string(),
                titulo.as_deref().unwrap_or_default(),
            ],
        )),
        (None, None) => None,
    }
}
