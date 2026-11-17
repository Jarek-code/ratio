//! Moderación (D8): cola de reportes e historial de decisiones, parte de la app.
//!
//! Solo la ve una cuenta con rol de moderación; el servidor lo comprueba en cada petición.
//! En la cola, cada contenido reportado se puede retirar, dejar con un aviso a quien lo
//! publicó o dejar tal cual descartando sus reportes. En el historial, un contenido retirado
//! se puede restaurar.

// LIBRERÍAS

use api::moderacion::Accion; // Decisión de moderación
use api::moderacion::Decision; // Decisión del historial
use api::moderacion::ReporteCola; // Contenido con reportes pendientes
use api::moderacion::TipoObjetivo; // Receta o comentario
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Rutas de la receta, el hilo y los perfiles
use crate::sesion::SenalSesion; // Sesión con token

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Envía una decisión al servidor y devuelve el texto del fallo, si lo hubo.
///
/// Moderar necesita conexión: una decisión no se deja en la cola sin conexión.
async fn decidir(
    sesion: SenalSesion,
    tipo: TipoObjetivo,
    objetivo_id: String,
    accion: Accion,
    nota: Option<String>,
) -> Result<(), String>
{
    let Some(token) = crate::sesion::token(&sesion) else
    {
        return Err("Vuelve a entrar para moderar.".to_string());
    };

    crate::red::con_plazo(api::moderacion::moderar(token, tipo, objetivo_id, accion, nota))
        .await
        .map_err(|causa| crate::red::mensaje(&causa))
}

/// Tarjeta de un contenido reportado, con sus motivos y las decisiones posibles.
///
/// `al_resolver` recarga la cola cuando la decisión se guardó.
#[component]
fn TarjetaReporte(reporte: ReporteCola, al_resolver: EventHandler<()>) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para decidir
    let mut nota_sel = use_signal(String::new); // Nota escrita
    let mut enviando = use_signal(|| false); // Decisión en curso
    let mut aviso = use_signal(|| None::<String>); // Fallo visible
    let sufijo = reporte.objetivo_id.clone(); // Ids únicos en la página
    let es_comentario = reporte.tipo == TipoObjetivo::Comentario;

    // Aplicar una decisión sobre este contenido
    let tipo = reporte.tipo;
    let objetivo_id = reporte.objetivo_id.clone();
    let aplicar = use_callback(move |accion: Accion|
    {
        let nota = nota_sel().trim().to_string();
        if accion == Accion::Avisar && nota.is_empty()
        {
            aviso.set(Some("Escribe la nota del aviso: es lo que leerá quien publicó.".to_string()));
            return; // El aviso necesita su texto
        }
        let objetivo_id = objetivo_id.clone();
        enviando.set(true);
        aviso.set(None);

        spawn(async move
        {
            match decidir(sesion, tipo, objetivo_id, accion, Some(nota)).await
            {
                Ok(()) => al_resolver.call(()),
                Err(mensaje) => aviso.set(Some(format!("No se pudo guardar la decisión: {mensaje}"))),
            }
            enviando.set(false);
        });
    });

    rsx!
    {
        li
        {
            class: "barra-comentario",

            p
            {
                class: "barra-meta",
                span { class: "muro-insignia", "{reporte.tipo.etiqueta()}" }
                " de "
                Link
                {
                    to: Rutas::PerfilPublico { alias: reporte.autor.clone() },
                    class: "barra-autor",
                    "aria-label": "Ver el perfil de {reporte.autor}",
                    "{reporte.autor}"
                }
                " · primer reporte el {reporte.fecha}"
            }
            p
            {
                Link
                {
                    to: if es_comentario { Rutas::Hilo { id: reporte.receta_id.clone() } } else { Rutas::Ficha { id: reporte.receta_id.clone() } },
                    class: "barra-autor",
                    "{reporte.receta_nombre}"
                }
            }
            if es_comentario
            {
                p { class: "moderacion-cita", "«{reporte.contenido}»" }
            }

            p { class: "barra-meta", if reporte.motivos.len() == 1 { "Reportado 1 vez" } else { "Reportado {reporte.motivos.len()} veces" } }
            ul
            {
                class: "moderacion-motivos",
                for (indice, motivo) in reporte.motivos.iter().enumerate()
                {
                    li { key: "{indice}", "{motivo}" }
                }
            }

            label { r#for: "moderacion-nota-{sufijo}", class: "barra-meta", "Nota (obligatoria para avisar)" }
            textarea
            {
                id: "moderacion-nota-{sufijo}",
                class: "barra-texto",
                rows: "2",
                maxlength: "{api::moderacion::NOTA_MAXIMA}",
                placeholder: "Qué decidiste y por qué",
                value: "{nota_sel}",
                oninput: move |evento| nota_sel.set(evento.value()),
            }

            div
            {
                class: "acciones-fila",
                role: "group",
                "aria-label": "Decisión sobre este contenido",

                button { class: "chip", r#type: "button", disabled: enviando(), onclick: move |_| aplicar.call(Accion::Ocultar), "Retirar" }
                button { class: "chip", r#type: "button", disabled: enviando(), onclick: move |_| aplicar.call(Accion::Avisar), "Avisar" }
                button { class: "chip", r#type: "button", disabled: enviando(), onclick: move |_| aplicar.call(Accion::Descartar), "Descartar reportes" }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "alert", "{texto}" }
            }
        }
    }
}

/// Tarjeta de una decisión ya tomada; si el contenido sigue retirado, permite restaurarlo.
///
/// `al_resolver` recarga el historial cuando el contenido se restauró.
#[component]
fn TarjetaDecision(decision: Decision, al_resolver: EventHandler<()>) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para restaurar
    let mut enviando = use_signal(|| false); // Restauración en curso
    let mut aviso = use_signal(|| None::<String>); // Fallo visible
    let restaurable = decision.accion == Accion::Ocultar && decision.retirado;
    let tipo = decision.tipo;
    let objetivo_id = decision.objetivo_id.clone();

    rsx!
    {
        li
        {
            class: "barra-comentario",

            p
            {
                class: "barra-meta",
                span { class: "muro-insignia", "{decision.accion.etiqueta()}" }
                " {decision.tipo.etiqueta()} · {decision.moderador} · {decision.fecha}"
            }
            if decision.contenido.is_empty()
            {
                p { class: "barra-meta", "El contenido ya no existe." }
            }
            else
            {
                p { class: "moderacion-cita", "«{decision.contenido}»" }
            }
            if let Some(nota) = decision.nota.clone()
            {
                p { class: "barra-meta", "Nota: {nota}" }
            }

            // Contenido que sigue retirado: ofrecer restaurarlo
            if restaurable
            {
                button
                {
                    class: "chip",
                    r#type: "button",
                    disabled: enviando(),
                    onclick: move |_|
                    {
                        let objetivo_id = objetivo_id.clone();
                        enviando.set(true);
                        aviso.set(None);

                        spawn(async move
                        {
                            match decidir(sesion, tipo, objetivo_id, Accion::Restaurar, None).await
                            {
                                Ok(()) => al_resolver.call(()),
                                Err(mensaje) => aviso.set(Some(format!("No se pudo restaurar: {mensaje}"))),
                            }
                            enviando.set(false);
                        });
                    },
                    if enviando() { "Un momento…" } else { "Restaurar" }
                }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "alert", "{texto}" }
            }
        }
    }
}

/// Cola de contenidos con reportes pendientes.
#[component]
fn Cola() -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para pedir la cola
    let mut recurso = use_resource(move ||
    {
        let token = crate::sesion::token(&sesion).unwrap_or_default(); // Sin sesión la pantalla no llega aquí
        crate::red::con_reintentos(move || api::moderacion::cola_reportes(token.clone()))
    }); // Cola del servidor

    rsx!
    {
        // Elegir contenido según el estado de la petición
        match &*recurso.read()
        {
            None => rsx!
            {
                p { class: "estado", role: "status", "Cargando la cola de reportes…" }
            },
            Some(Err(causa)) => rsx!
            {
                // Fallo de red, del servidor o cuenta sin permiso: explicar y ofrecer reintentar
                AvisoRed
                {
                    causa: causa.clone(),
                    reintentar: move |_| recurso.restart(),
                }
            },
            Some(Ok(cola_vec)) if cola_vec.is_empty() => rsx!
            {
                p { class: "estado", "No hay reportes pendientes." }
            },
            Some(Ok(cola_vec)) => rsx!
            {
                ul
                {
                    class: "barra-hilo",
                    for reporte in cola_vec.iter()
                    {
                        TarjetaReporte
                        {
                            key: "{reporte.objetivo_id}",
                            reporte: reporte.clone(),
                            al_resolver: move |_| recurso.restart(),
                        }
                    }
                }
            },
        }
    }
}

/// Historial con las últimas decisiones de moderación.
#[component]
fn Historial() -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para pedir el historial
    let mut recurso = use_resource(move ||
    {
        let token = crate::sesion::token(&sesion).unwrap_or_default(); // Sin sesión la pantalla no llega aquí
        crate::red::con_reintentos(move || api::moderacion::historial_moderacion(token.clone()))
    }); // Historial del servidor

    rsx!
    {
        // Elegir contenido según el estado de la petición
        match &*recurso.read()
        {
            None => rsx!
            {
                p { class: "estado", role: "status", "Cargando el historial…" }
            },
            Some(Err(causa)) => rsx!
            {
                // Fallo de red, del servidor o cuenta sin permiso: explicar y ofrecer reintentar
                AvisoRed
                {
                    causa: causa.clone(),
                    reintentar: move |_| recurso.restart(),
                }
            },
            Some(Ok(decisiones_vec)) if decisiones_vec.is_empty() => rsx!
            {
                p { class: "estado", "Todavía no hay decisiones registradas." }
            },
            Some(Ok(decisiones_vec)) => rsx!
            {
                ul
                {
                    class: "barra-hilo",
                    for (indice, decision) in decisiones_vec.iter().enumerate()
                    {
                        TarjetaDecision
                        {
                            key: "{indice}-{decision.objetivo_id}",
                            decision: decision.clone(),
                            al_resolver: move |_| recurso.restart(),
                        }
                    }
                }
            },
        }
    }
}

/// Pantalla D8: la cola de reportes y el historial de decisiones.
#[component]
pub fn PantallaModeracion() -> Element
{
    let sesion = use_context::<SenalSesion>(); // Para saber si hay sesión
    let mut en_historial = use_signal(|| false); // Pestaña visible
    let con_sesion = sesion.read().is_some();

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::Perfil, class: "enlace", aria_label: "Volver a tu perfil", "‹ Perfil" }
            }

            p { class: "eyebrow", "La Barra" }
            h1 { "Moderación" }

            // Sin sesión: invitar a entrar; con sesión: cola o historial
            if !con_sesion
            {
                div
                {
                    class: "estado",
                    p { "Entra con una cuenta de moderación para revisar los reportes." }
                    Link { to: Rutas::Entrar, class: "enlace", "Entrar" }
                }
            }
            else
            {
                nav
                {
                    class: "chips",
                    "aria-label": "Secciones de moderación",

                    button
                    {
                        class: if en_historial() { "chip" } else { "chip chip--activo" },
                        r#type: "button",
                        "aria-pressed": if en_historial() { "false" } else { "true" },
                        onclick: move |_| en_historial.set(false),
                        "Cola"
                    }
                    button
                    {
                        class: if en_historial() { "chip chip--activo" } else { "chip" },
                        r#type: "button",
                        "aria-pressed": if en_historial() { "true" } else { "false" },
                        onclick: move |_| en_historial.set(true),
                        "Historial"
                    }
                }

                if en_historial()
                {
                    Historial {}
                }
                else
                {
                    Cola {}
                }
            }
        }

        BarraInferior { activo: Destino::Perfil }
    }
}
