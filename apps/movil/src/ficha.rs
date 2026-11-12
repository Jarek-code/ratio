//! Ficha de receta (B3): el vaso con sus capas, las proporciones, los datos y los pasos.

use api::guardados::alternar_guardado;
use api::receta::{PasoFicha, RecetaCompleta, completa};
use dioxus::prelude::*;

use crate::proximamente::Proximamente;
use crate::reportar::ModalReportar;
use crate::rutas::Rutas;

/// Hoja de estilos propia de la ficha.
const CSS_FICHA: Asset = asset!("/assets/ficha.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Color de reserva del punto de la leyenda cuando la capa no trae uno.
const COLOR_RESERVA: &str = "#8A6A4A";

/// Ficha de una receta publicada, cargada por su identificador.
#[component]
pub fn PantallaFicha(id: String) -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut receta_res = use_resource(move || {
        let id = id.clone();
        let token = crate::sesion::token(&sesion);
        async move { completa(id, token).await }
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_FICHA }

        main { class: "ficha",
            BarraSuperior {}

            match &*receta_res.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando la receta…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos cargar la receta: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| receta_res.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(None)) => rsx! {
                    p { class: "estado", "Esta receta no existe o ya no está publicada." }
                },
                Some(Ok(Some(receta))) => rsx! {
                    Contenido { receta: receta.clone() }
                },
            }
        }
    }
}

/// Barra superior con la vuelta a Descubrir.
#[component]
fn BarraSuperior() -> Element {
    rsx! {
        header { class: "ficha-barra",
            Link { to: Rutas::Descubrir, class: "volver", "‹ Descubrir" }
        }
    }
}

/// Cuerpo de la ficha: vaso, proporciones, acciones y pasos.
#[component]
fn Contenido(receta: RecetaCompleta) -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut guardando = use_signal(|| false);
    let mut guardada = use_signal(|| false);
    let mut aviso_guardado = use_signal(|| None::<String>);
    let mut reportando = use_signal(|| false);
    let id_guardar = receta.id.clone();
    let capas = proporciones::capas(&receta.receta);
    let graduacion = proporciones::graduacion(&receta.receta);
    let svg = proporciones::svg(
        &receta.receta,
        &proporciones::Opciones {
            escalas: false,
            mini: false,
        },
    );

    rsx! {
        section { class: "ficha-cabeza",
            p { class: "eyebrow", "{receta.familia}" }
            h1 { "{receta.receta.nombre}" }
            p { class: "ficha-autor", "por {receta.autor}" }
        }

        article { class: "tarjeta ficha-vaso",
            div {
                class: "vaso",
                role: "img",
                "aria-label": descripcion(&receta),
                dangerous_inner_html: svg,
            }

            dl { class: "ficha-datos",
                div { class: "dato",
                    dt { "Vaso" }
                    dd { "{receta.vaso_nombre}" }
                }
                div { class: "dato",
                    dt { "Capacidad" }
                    dd { "{receta.receta.capacidad_ml:.0} ml" }
                }
                div { class: "dato",
                    dt { "Servido" }
                    dd { "{capas.total_ml:.0} ml" }
                }
                if graduacion.abv > 0.0 {
                    div { class: "dato",
                        dt { "Graduación" }
                        dd { "{graduacion.abv:.1} % ABV" }
                    }
                }
                if let Some(minutos) = receta.minutos {
                    div { class: "dato",
                        dt { "Tiempo" }
                        dd { "{minutos} min" }
                    }
                }
                if let Some(dificultad) = receta.dificultad {
                    div { class: "dato",
                        dt { "Dificultad" }
                        dd { "{dificultad} de 5" }
                    }
                }
                if let Some(intensidad) = receta.intensidad {
                    div { class: "dato",
                        dt { "Intensidad" }
                        dd { "{intensidad} de 5" }
                    }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-proporciones",
            div { class: "seccion-cabeza",
                p { id: "titulo-proporciones", class: "eyebrow", "Proporciones en el vaso" }
                span { class: "mini", "{capas.total_ml:.0} ml en {receta.receta.capacidad_ml:.0} ml de vaso" }
            }

            ul { class: "leyenda",
                for (indice, capa) in capas.capas.iter().enumerate() {
                    {
                        // El motor mide todo en ml; la unidad original solo cambia la
                        // etiqueta. De momento 1 g se dibuja como 1 ml.
                        let unidad = receta
                            .componentes
                            .get(indice)
                            .map(|componente| {
                                if componente.unidad == "mg" { "mg" } else { "ml" }
                            })
                            .unwrap_or("ml");
                        let cantidad = if unidad == "mg" {
                            capa.ml * 1000.0
                        } else {
                            capa.ml
                        };

                        rsx! {
                            li { class: "leyenda-fila",
                                span {
                                    class: "leyenda-punto",
                                    style: "background:{capa.color.clone().unwrap_or_else(|| COLOR_RESERVA.to_string())}",
                                    "aria-hidden": "true",
                                }
                                span { class: "leyenda-nombre", "{capa.nombre}" }
                                span { class: "leyenda-ml", "{cantidad:.0} {unidad}" }
                                span { class: "leyenda-pct", "{capa.porcentaje:.0} %" }
                            }
                        }
                    }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-acciones",
            p { id: "titulo-acciones", class: "eyebrow", "Acciones" }

            div { class: "acciones",
                Proximamente {
                    button { class: "primario", r#type: "button", "aria-disabled": "true", "Preparar" }
                }
                div { class: "acciones-fila",
                    Link {
                        to: Rutas::Historial { id: receta.id.clone() },
                        class: "chip chip--enlace",
                        "Versiones"
                    }
                    Link {
                        to: Rutas::Ajustar { id: receta.id.clone() },
                        class: "chip chip--enlace",
                        "Ajustar"
                    }
                    Proximamente {
                        button { class: "chip", r#type: "button", "aria-disabled": "true", "Comparar" }
                    }
                    Proximamente {
                        button { class: "chip", r#type: "button", "aria-disabled": "true", "Derivar" }
                    }
                    button {
                        class: "chip",
                        r#type: "button",
                        onclick: move |_| reportando.set(true),
                        "Reportar"
                    }
                    button {
                        class: "chip",
                        r#type: "button",
                        disabled: *guardando.read() || *guardada.read(),
                        onclick: move |_| {
                            if *guardando.read() || *guardada.read() {
                                return;
                            }

                            let Some(token) = crate::sesion::token(&sesion) else {
                                aviso_guardado.set(Some(
                                    "Inicia sesión para guardar la receta.".to_string(),
                                ));
                                return;
                            };

                            let id = id_guardar.clone();
                            guardando.set(true);
                            aviso_guardado.set(None);
                            spawn(async move {
                                match alternar_guardado(token, id, true).await {
                                    Ok(_) => {
                                        guardada.set(true);
                                        guardando.set(false);
                                    }
                                    Err(causa) => {
                                        aviso_guardado.set(Some(format!(
                                            "No se pudo guardar: {causa}"
                                        )));
                                        guardando.set(false);
                                    }
                                }
                            });
                        },
                        if *guardada.read() {
                            "Guardado"
                        } else if *guardando.read() {
                            "Guardando…"
                        } else {
                            "Guardar"
                        }
                    }
                }

                if let Some(texto) = aviso_guardado() {
                    p { class: "ficha-aviso", role: "alert", "{texto}" }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-preparacion",
            div { class: "seccion-cabeza",
                p { id: "titulo-preparacion", class: "eyebrow", "Preparación" }
                span { class: "mini", "{receta.pasos.len()} pasos" }
            }

            ol { class: "pasos",
                for (indice, paso) in receta.pasos.iter().enumerate() {
                    Paso { numero: indice + 1, paso: paso.clone() }
                }
            }
        }

        if !receta.notas.is_empty() {
            section { class: "seccion", "aria-labelledby": "titulo-notas",
                p { id: "titulo-notas", class: "eyebrow", "Notas de sabor" }
                ul { class: "notas",
                    for nota in receta.notas.iter() {
                        li { class: "chip", "{nota}" }
                    }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-barra",
            p { id: "titulo-barra", class: "eyebrow", "La Barra · comentarios anclados" }
            p { class: "estado", "El hilo de esta receta llegará con el foro." }
        }

        section { class: "seccion", "aria-labelledby": "titulo-versiones",
            p { id: "titulo-versiones", class: "eyebrow", "Versiones" }
            p { class: "estado", "Las versiones derivadas llegarán con el versionado." }
        }

        if reportando() {
            ModalReportar {
                receta_id: receta.id.clone(),
                on_cerrar: move |_| reportando.set(false),
            }
        }
    }
}

/// Un paso de la preparación, con su número y su temporizador.
#[component]
fn Paso(numero: usize, paso: PasoFicha) -> Element {
    rsx! {
        li { class: "paso",
            span { class: "paso-numero", "aria-hidden": "true", "{numero}" }
            div { class: "paso-texto",
                h3 { "{paso.titulo}" }
                p { "{paso.instruccion}" }
            }
            if let Some(segundos) = paso.temporizador_s {
                span { class: "paso-tiempo", "{segundos} s" }
            }
        }
    }
}

/// Texto accesible del vaso: nombre y porcentaje de cada capa.
fn descripcion(receta: &RecetaCompleta) -> String {
    let capas = proporciones::capas(&receta.receta);
    let partes: Vec<String> = capas
        .capas
        .iter()
        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
        .collect();

    format!("{}: {}", receta.receta.nombre, partes.join(", "))
}
