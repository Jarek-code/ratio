//! Ficha de receta (B3): el vaso con sus capas, las proporciones, los datos y los pasos.

use api::guardados::alternar_guardado;
use api::receta::{PasoFicha, RecetaCompleta};
use dioxus::prelude::*;

use crate::i18n::Texto;
use crate::red::AvisoRed;
use crate::reportar::ModalReportar;
use crate::rutas::Rutas; // Textos traducibles

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
        let cuenta = crate::sesion::cuenta(&sesion);
        crate::red::con_reintentos(move || {
            crate::local::receta(id.clone(), token.clone(), cuenta.clone())
        })
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_FICHA }

        main { class: "ficha",
            BarraSuperior {}

            match &*receta_res.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoReceta)}" }
                },
                Some(Err(causa)) => rsx! {
                    AvisoRed {
                        causa: causa.clone(),
                        reintentar: move |_| receta_res.restart(),
                    }
                },
                Some(Ok(None)) => rsx! {
                    p { class: "estado", "{crate::i18n::ahora(Texto::RecetaNoPublicadaFicha)}" }
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
            Link { to: Rutas::Descubrir, class: "volver", "{crate::i18n::ahora(Texto::VolverDescubrirBoton)}" }
        }
    }
}

/// Cuerpo de la ficha: vaso, proporciones, acciones y pasos.
#[component]
fn Contenido(receta: RecetaCompleta) -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut pendientes = use_context::<crate::local::SenalPendientes>();
    let mut guardando = use_signal(|| false);
    let mut guardada = use_signal(|| false);
    let mut aviso_guardado = use_signal(|| None::<String>);
    let mut reportando = use_signal(|| false);
    let id_guardar = receta.id.clone();
    let capas = proporciones::capas(&receta.receta);
    let graduacion = proporciones::graduacion(&receta.receta);
    let equivalencias = proporciones::equivalencias(&receta.receta);
    let unidad_preferida = use_context::<crate::ajustes::SenalAjustes>()().unidad;
    let texto_volumen = crate::i18n::con(
        crate::i18n::actual(),
        Texto::MlEnVaso,
        &[
            &format!("{:.0}", capas.total_ml),
            &format!("{:.0}", receta.receta.capacidad_ml),
        ],
    );
    let svg = proporciones::svg(
        &receta.receta,
        &proporciones::Opciones {
            escalas: false,
            mini: false,
        },
    );

    rsx! {
        section { class: "ficha-cabeza",
            p { class: "eyebrow", "{crate::biblioteca::familia_legible(&receta.familia)}" }
            h1 { "{receta.receta.nombre}" }
            div { class: "ficha-autor",
                span { class: "ficha-autor-por", "por" }
                Link {
                    to: Rutas::PerfilPublico { alias: receta.autor.clone() },
                    class: "ficha-autor-enlace",

                    span { class: "ficha-autor-inicial", "aria-hidden": "true",
                        "{receta.autor.chars().next().unwrap_or('R').to_uppercase()}"
                    }
                    span { "{receta.autor}" }
                }
            }
            // Versión derivada: crédito al original (D5)
            if let Some(origen) = receta.origen.clone() {
                p { class: "ficha-autor",
                    span { class: "ficha-autor-por", "{crate::i18n::ahora(Texto::VersionDe)}" }
                    Link {
                        to: Rutas::Ficha { id: origen.id.clone() },
                        class: "ficha-autor-enlace",

                        span { class: "ficha-autor-inicial", "aria-hidden": "true", "↩" }
                        span { "{origen.nombre}" }
                    }
                    span { class: "ficha-autor-por", "{crate::i18n::con(crate::i18n::actual(), Texto::OriginalDe, &[&origen.autor])}" }
                }
            }
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
                    dt { "{crate::i18n::ahora(Texto::Vaso)}" }
                    dd { "{receta.vaso_nombre}" }
                }
                div { class: "dato",
                    dt { "{crate::i18n::ahora(Texto::Capacidad)}" }
                    dd { "{receta.receta.capacidad_ml:.0} ml" }
                }
                div { class: "dato",
                    dt { "{crate::i18n::ahora(Texto::Servido)}" }
                    dd { "{capas.total_ml:.0} ml" }
                }
                if graduacion.abv > 0.0 {
                    div { class: "dato",
                        dt { "{crate::i18n::ahora(Texto::Graduacion)}" }
                        dd { "{graduacion.abv:.1} % ABV" }
                    }
                }
                if let Some(minutos) = receta.minutos {
                    div { class: "dato",
                        dt { "{crate::i18n::ahora(Texto::OrdenTiempo)}" }
                        dd { "{minutos} min" }
                    }
                }
                if let Some(dificultad) = receta.dificultad {
                    div { class: "dato",
                        dt { "{crate::i18n::ahora(Texto::OrdenDificultad)}" }
                        dd { "{crate::i18n::con(crate::i18n::actual(), Texto::NivelDeCinco, &[&dificultad.to_string()])}" }
                    }
                }
                if let Some(intensidad) = receta.intensidad {
                    div { class: "dato",
                        dt { "{crate::i18n::ahora(Texto::OrdenIntensidad)}" }
                        dd { "{crate::i18n::con(crate::i18n::actual(), Texto::NivelDeCinco, &[&intensidad.to_string()])}" }
                    }
                }
            }
        }

        crate::fotos::FotosReceta { receta: receta.clone() }

        section { class: "seccion", "aria-labelledby": "titulo-proporciones",
            div { class: "seccion-cabeza",
                p { id: "titulo-proporciones", class: "eyebrow", "{crate::i18n::ahora(Texto::ProporcionesEnElVaso)}" }
                span { class: "mini", "{texto_volumen}" }
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
                                span { class: "leyenda-ml",
                                    // Los líquidos usan la unidad elegida en Ajustes; los sólidos siguen en mg
                                    if unidad == "mg" {
                                        "{cantidad:.0} mg"
                                    } else if let Some(equivalencia) = equivalencias.get(indice) {
                                        "{unidad_preferida.cantidad(equivalencia)}"
                                    } else {
                                        "{cantidad:.0} ml"
                                    }
                                }
                                span { class: "leyenda-pct", "{capa.porcentaje:.0} %" }
                            }
                        }
                    }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-acciones",
            p { id: "titulo-acciones", class: "eyebrow", "{crate::i18n::ahora(Texto::Acciones)}" }

            div { class: "acciones",
                Link {
                    to: Rutas::Preparar { id: receta.id.clone() },
                    class: "primario",
                    {crate::i18n::ahora(Texto::Preparar)}
                }
                div { class: "acciones-rejilla",
                    Link {
                        to: Rutas::Ajustar { id: receta.id.clone() },
                        class: "chip chip--enlace chip--ancho",
                        aria_label: crate::i18n::ahora(Texto::AjustarAria),
                        {crate::i18n::ahora(Texto::AjustarODerivar)}
                    }
                    Link {
                        to: Rutas::Comparar { id: receta.id.clone(), con: String::new() },
                        class: "chip chip--enlace",
                        {crate::i18n::ahora(Texto::CompararBoton)}
                    }
                    Link {
                        to: Rutas::Equivalencias { id: receta.id.clone() },
                        class: "chip chip--enlace",
                        {crate::i18n::ahora(Texto::Ingredientes)}
                    }
                    button {
                        class: "chip",
                        r#type: "button",
                        onclick: move |_| reportando.set(true),
                        {crate::i18n::ahora(Texto::Reportar)}
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
                                    {crate::i18n::ahora(Texto::IniciaSesionGuardar)}.to_string(),
                                ));
                                return;
                            };

                            let id = id_guardar.clone();
                            guardando.set(true);
                            aviso_guardado.set(None);
                            spawn(async move {
                                match alternar_guardado(token, id.clone(), true).await {
                                    Ok(_) => {
                                        guardada.set(true);
                                        guardando.set(false);
                                    }
                                    // Sin red: el guardado queda en la cola
                                    Err(causa)
                                        if crate::red::sin_conexion(&causa)
                                            && crate::local::encolar(
                                                &sesion,
                                                &mut pendientes,
                                                crate::local::Cambio::Guardado {
                                                    receta_id: id,
                                                    guardar: true,
                                                },
                                            ) =>
                                    {
                                        guardada.set(true);
                                        guardando.set(false);
                                        aviso_guardado.set(Some(
                                            {crate::i18n::ahora(Texto::SinConexionSeGuardara)}
                                                .to_string(),
                                        ));
                                    }
                                    Err(causa) => {
                                        aviso_guardado.set(Some(crate::i18n::con(crate::i18n::actual(), Texto::NoSePudoGuardar, &[&crate::red::mensaje(&causa)])));
                                        guardando.set(false);
                                    }
                                }
                            });
                        },
                        if *guardada.read() {
                            {crate::i18n::ahora(Texto::Guardado)}
                        } else if *guardando.read() {
                            {crate::i18n::ahora(Texto::Guardando)}
                        } else {
                            {crate::i18n::ahora(Texto::Guardar)}
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
                p { id: "titulo-preparacion", class: "eyebrow", "{crate::i18n::ahora(Texto::Preparacion)}" }
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
                p { id: "titulo-notas", class: "eyebrow", "{crate::i18n::ahora(Texto::NotasDeSabor)}" }
                ul { class: "notas",
                    for nota in receta.notas.iter() {
                        li { class: "chip", "{nota}" }
                    }
                }
            }
        }

        crate::foro::ResumenHilo { receta: receta.clone() }

        section { class: "seccion", "aria-labelledby": "titulo-versiones",
            p { id: "titulo-versiones", class: "eyebrow", "{crate::i18n::ahora(Texto::VersionesSeccion)}" }
            p { class: "nota", "{crate::i18n::ahora(Texto::CadaVersionDeriva)}" }
            Link {
                to: Rutas::Historial { id: receta.id.clone() },
                class: "chip chip--enlace",
                {crate::i18n::ahora(Texto::VerHistorialDeVersiones)}
            }
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
