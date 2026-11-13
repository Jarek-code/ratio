//! D7: modal para reportar una receta desde la ficha.
//!
//! El modal solo manda el identificador de la receta, el motivo y el detalle.
//! Quien reporta lo decide el servidor a partir del token. Si no hay sesion, el
//! modal lo avisa en vez de dejar el envio a medias. Usa la server function de
//! La Barra (`api::foro::reportar`), que aplica la regla de edad y guarda un solo
//! reporte por cuenta; sin conexion, el reporte queda en la cola.
//!
//! Los comentarios de este modulo van sin acentos por pedido del equipo.

use api::foro::{Motivo, ReporteNuevo, reportar};
use dioxus::prelude::*;

/// Hoja de estilos propia del modal.
const CSS_REPORTAR: Asset = asset!("/assets/reportar.css");

/// Modal de reporte para una receta concreta.
///
/// `on_cerrar` se llama al tocar el fondo, el boton de cerrar o Cancelar.
#[component]
pub fn ModalReportar(receta_id: String, on_cerrar: EventHandler<()>) -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut pendientes = use_context::<crate::local::SenalPendientes>();
    let mut motivo = use_signal(|| Motivo::Spam);
    let mut detalle = use_signal(String::new);
    let mut enviando = use_signal(|| false);
    let mut enviado = use_signal(|| false);
    let mut aviso = use_signal(|| None::<String>);
    let id_reporte = receta_id.clone();

    rsx! {
        document::Stylesheet { href: CSS_REPORTAR }

        div {
            class: "reportar-fondo",
            onclick: move |_| on_cerrar.call(()),

            section {
                class: "reportar-panel",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "titulo-reportar",
                onclick: move |evento| evento.stop_propagation(),

                div { class: "reportar-agarre", "aria-hidden": "true" }

                button {
                    class: "reportar-cerrar",
                    r#type: "button",
                    "aria-label": "Cerrar",
                    onclick: move |_| on_cerrar.call(()),
                    "×"
                }

                h2 { id: "titulo-reportar", class: "reportar-titulo", "Reportar receta" }
                p { class: "reportar-intro",
                    "Cuéntanos qué pasa con esta receta. Tu reporte es privado y lo revisa el equipo de moderación."
                }

                fieldset { class: "reportar-grupo",
                    legend { class: "eyebrow", "Motivo" }
                    div { class: "reportar-motivos",
                        for valor in Motivo::TODOS {
                            button {
                                class: if motivo() == valor { "reportar-motivo reportar-motivo--activo" } else { "reportar-motivo" },
                                r#type: "button",
                                "aria-pressed": if motivo() == valor { "true" } else { "false" },
                                disabled: *enviando.read() || *enviado.read(),
                                onclick: move |_| motivo.set(valor),
                                "{valor.etiqueta()}"
                            }
                        }
                    }
                }

                label { class: "reportar-grupo",
                    span { class: "eyebrow", "Detalle (opcional)" }
                    textarea {
                        class: "reportar-detalle",
                        rows: "4",
                        maxlength: "1000",
                        placeholder: "Agrega contexto para el equipo de moderación.",
                        value: "{detalle}",
                        disabled: *enviando.read() || *enviado.read(),
                        oninput: move |evento| detalle.set(evento.value()),
                    }
                }

                div { class: "reportar-pie",
                    span { class: "reportar-cuenta", "{detalle().chars().count()} / 1000" }
                    span { class: "reportar-privado", "Solo lo ve moderación" }
                }

                if let Some(texto) = aviso() {
                    p { class: "reportar-aviso", role: "alert", "{texto}" }
                }

                if *enviado.read() {
                    p { class: "reportar-gracias", role: "status",
                        "Gracias. Revisaremos el contenido y tomaremos una decisión."
                    }
                }

                div { class: "reportar-acciones",
                    button {
                        class: "chip",
                        r#type: "button",
                        disabled: *enviando.read(),
                        onclick: move |_| on_cerrar.call(()),
                        "Cancelar"
                    }

                    button {
                        class: "primario",
                        r#type: "button",
                        disabled: *enviando.read() || *enviado.read(),
                        onclick: move |_| {
                            if *enviando.read() || *enviado.read() {
                                return;
                            }

                            let Some(token) = crate::sesion::token(&sesion) else {
                                aviso.set(Some(
                                    "Inicia sesión para reportar este contenido.".to_string(),
                                ));
                                return;
                            };

                            let texto = detalle();
                            let limpio = texto.trim();
                            let datos = ReporteNuevo {
                                receta_id: Some(id_reporte.clone()),
                                comentario_id: None,
                                motivo: motivo(),
                                detalle: if limpio.is_empty() {
                                    None
                                } else {
                                    Some(limpio.to_string())
                                },
                            };

                            enviando.set(true);
                            aviso.set(None);
                            spawn(async move {
                                match reportar(token, datos.clone()).await {
                                    Ok(_) => {
                                        enviado.set(true);
                                        enviando.set(false);
                                    }
                                    // Sin red: el reporte queda en la cola
                                    Err(causa)
                                        if crate::local::es_sin_conexion(&causa)
                                            && crate::local::encolar(
                                                &sesion,
                                                &mut pendientes,
                                                crate::local::Cambio::Reporte(datos),
                                            ) =>
                                    {
                                        enviado.set(true);
                                        enviando.set(false);
                                    }
                                    Err(causa) => {
                                        aviso.set(Some(format!(
                                            "No se pudo enviar el reporte: {causa}"
                                        )));
                                        enviando.set(false);
                                    }
                                }
                            });
                        },
                        if *enviado.read() {
                            "Reporte enviado"
                        } else if *enviando.read() {
                            "Enviando…"
                        } else {
                            "Enviar reporte"
                        }
                    }
                }
            }
        }
    }
}
