//! Pantalla B6: historial de versiones de una bebida.
//!
//! Muestra la receta original y las versiones derivadas en orden cronológico,
//! con el cambio de cada ingrediente respecto a su versión de origen. Los datos
//! salen de `receta.receta_origen_id` y de `componente_receta`.

use dioxus::prelude::*;
use dioxus::router::Link;

use api::historial::{CambioHistorial, VersionHistorial, versiones};

use crate::i18n::Texto;
use crate::marca::Marca;
use crate::red::AvisoRed;
use crate::rutas::Rutas;
use crate::sesion::SenalSesion; // Textos traducibles

/// Hoja del sistema visual que comparten las pantallas: trae la paleta y el tema oscuro.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Hoja del acceso: aporta los estilos de marca que reutiliza el historial.
const CSS_LOGIN: Asset = asset!("/assets/login.css");

/// Hoja propia del historial de versiones.
const CSS_HISTORIAL: Asset = asset!("/assets/historial.css");

/// Filtro de visibilidad de las versiones.
#[derive(Clone, Copy, PartialEq)]
enum Filtro {
    /// Muestra todas las versiones sin importar su visibilidad.
    Todas,
    /// Muestra solo las versiones públicas.
    Publicas,
    /// Muestra solo las versiones privadas.
    Privadas,
}

impl Filtro {
    /// Texto que se muestra en cada chip.
    fn etiqueta(self) -> &'static str {
        match self {
            Self::Todas => crate::i18n::ahora(Texto::TodasFiltro),
            Self::Publicas => crate::i18n::ahora(Texto::Publicas),
            Self::Privadas => crate::i18n::ahora(Texto::Privadas),
        }
    }

    /// Indica si una versión entra en el filtro activo.
    fn incluye(self, version: &VersionHistorial) -> bool {
        match self {
            Self::Todas => true,
            Self::Publicas => version.publica,
            Self::Privadas => !version.publica,
        }
    }
}

/// Etiqueta de visibilidad de una versión.
fn visibilidad(version: &VersionHistorial) -> &'static str {
    if version.publica {
        crate::i18n::ahora(Texto::Publica)
    } else {
        crate::i18n::ahora(Texto::PrivadaFiltro)
    }
}

/// Diferencia entre la versión actual y la de origen.
fn delta(cambio: &CambioHistorial) -> f64 {
    cambio.despues - cambio.antes
}

/// Texto con signo para el indicador de cambio, por ejemplo «+15» o «-20».
fn signo(cambio: &CambioHistorial) -> String {
    let diferencia = delta(cambio);

    if diferencia > 0.0 {
        format!("+{diferencia:.0}")
    } else {
        format!("{diferencia:.0}")
    }
}

/// Permite pintar de distinto color las bajadas.
fn es_baja(cambio: &CambioHistorial) -> bool {
    delta(cambio) < 0.0
}

/// Nombre de la versión original del árbol, si está en la lista.
fn original(versiones: &[VersionHistorial]) -> Option<&VersionHistorial> {
    versiones.iter().find(|version| !version.derivada)
}

/// Pantalla B6: historial de versiones de una bebida.
///
/// Pide el árbol de versiones al servidor y filtra en memoria: son pocas por bebida y así
/// los chips responden sin otra petición.
#[component]
pub fn Historial(id: String) -> Element {
    let sesion = use_context::<SenalSesion>();
    let mut filtro = use_signal(|| Filtro::Todas);
    let id_recurso = id.clone();
    let mut versiones_res = use_resource(use_reactive!(|(id_recurso,)| {
        let token = crate::sesion::token(&sesion);
        crate::red::con_reintentos(move || versiones(id_recurso.clone(), token.clone()))
    }));

    rsx! {
        // Primero la paleta y el tema; después el acceso y las reglas del historial.
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_LOGIN }
        document::Stylesheet { href: CSS_HISTORIAL }

        main { class: "historial-page",
            section { class: "historial-shell",
                header { class: "historial-top",
                    crate::rutas::Volver {
                        etiqueta: crate::i18n::ahora(Texto::VolverALaFichaReceta).to_string(),
                        respaldo: Rutas::Ficha { id: id.clone() },
                    }
                    div { class: "brand-mini",
                        Marca {}
                        span { "Ratio" }
                    }
                }

                div { class: "historial-heading",
                    p { class: "eyebrow", "Recetario · B6" }
                    h1 { "{crate::i18n::ahora(Texto::HistorialDeVersiones)}" }
                }

                match &*versiones_res.read() {
                    None => rsx! {
                        p { class: "historial-estado", role: "status", "{crate::i18n::ahora(Texto::CargandoHistorial)}" }
                    },
                    Some(Err(causa)) => rsx! {
                        AvisoRed {
                            causa: causa.clone(),
                            clase: Some("historial-estado".to_string()),
                            reintentar: move |_| versiones_res.restart(),
                        }
                    },
                    Some(Ok(lista)) if lista.is_empty() => rsx! {
                        p { class: "historial-estado", "{crate::i18n::ahora(Texto::SinVersionesBebida)}" }
                    },
                    Some(Ok(lista)) => rsx! {
                        {
                            let raiz = original(lista);
                            let total = lista.len();
                            let visibles = lista.iter().filter(|version| filtro().incluye(version)).count();

                            rsx! {
                                if let Some(raiz) = raiz {
                                    section { class: "recipe-summary", aria_label: "{crate::i18n::ahora(Texto::RecetaOriginal)}",
                                        div {
                                            class: "summary-mark",
                                            if let Some(bebida) = raiz.bebida.clone() {
                                                div {
                                                    class: "summary-cup",
                                                    dangerous_inner_html: proporciones::svg(
                                                        &bebida,
                                                        &proporciones::Opciones { escalas: false, mini: true },
                                                    ),
                                                }
                                            } else {
                                                Marca {}
                                            }
                                        }
                                        div { class: "recipe-summary__body",
                                            p { class: "summary-label", "{crate::i18n::ahora(Texto::RecetaOriginal)}" }
                                            h2 { "{raiz.nombre}" }
                                            p { "{raiz.autor} · {crate::i18n::fecha_iso(&raiz.fecha)}" }
                                        }
                                        div { class: "recipe-summary__metrics",
                                            div { class: "metric",
                                                strong { "{raiz.total_ml:.0} ml" }
                                                span { "{crate::i18n::ahora(Texto::Total)}" }
                                            }
                                            div { class: "metric",
                                                strong { "{total}" }
                                                span { if total == 1 { "{crate::i18n::ahora(Texto::VersionUna)}" } else { "{crate::i18n::ahora(Texto::Versiones)}" } }
                                            }
                                        }
                                    }
                                }

                                nav { class: "filters", aria_label: "{crate::i18n::ahora(Texto::FiltrarVersiones)}",
                                    for opcion in [Filtro::Todas, Filtro::Publicas, Filtro::Privadas] {
                                        button {
                                            key: "{opcion.etiqueta()}",
                                            class: if filtro() == opcion { "chip chip--active" } else { "chip" },
                                            r#type: "button",
                                            onclick: move |_| filtro.set(opcion),
                                            "{opcion.etiqueta()}"
                                        }
                                    }
                                    span { class: "filters__count", "{crate::i18n::con(crate::i18n::actual(), Texto::DeTotalCorto, &[&visibles.to_string(), &total.to_string()])}" }
                                }

                                section { class: "timeline", aria_label: "{crate::i18n::ahora(Texto::LineaDeTiempoVersiones)}",
                                    for (indice, version) in lista
                                        .iter()
                                        .filter(|version| filtro().incluye(version))
                                        .enumerate()
                                    {
                                        VersionCard {
                                            key: "{version.id}",
                                            indice: indice + 1,
                                            version: version.clone(),
                                        }
                                    }
                                }

                                footer { class: "historial-foot",
                                    span { "{crate::i18n::ahora(Texto::TocaAbrir)}" }
                                    span { "{crate::i18n::ahora(Texto::DatosDeLaBase)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Tarjeta de una versión dentro de la línea de tiempo.
///
/// `indice` es la posición visible (empieza en 1) y `version` son los datos que se pintan.
#[component]
fn VersionCard(indice: usize, version: VersionHistorial) -> Element {
    let numero = format!("{indice:02}");
    let total = format!("{:.0}", version.total_ml);

    rsx! {
        article {
            class: if version.derivada { "version-card" } else { "version-card version-card--origin" },
            div { class: "version-head",
                span { class: "version-index", "{numero}" }
                h2 { class: "version-title", "{version.nombre}" }
                span {
                    class: if version.derivada {
                        "version-badge version-badge--derivada"
                    } else {
                        "version-badge version-badge--original"
                    },
                    if version.derivada { "{crate::i18n::ahora(Texto::Derivada)}" } else { "Original" }
                }
            }

            p { class: "version-meta",
                "{version.autor} · {crate::i18n::fecha_iso(&version.fecha)} · "
                span {
                    class: if version.publica { "visibilidad" } else { "visibilidad visibilidad--privada" },
                    "{visibilidad(&version)}"
                }
            }

            if version.derivada && version.origen.is_none() {
                // Origen oculto: no se muestran sus cantidades
                p { class: "version-note", "{crate::i18n::ahora(Texto::DerivaNoDisponible)}" }
            } else if version.cambios.is_empty() {
                p { class: "version-note",
                    if version.derivada { "{crate::i18n::ahora(Texto::SinCambiosFrenteAlOrigen)}" } else { "{crate::i18n::ahora(Texto::RecetaOriginalSinCambios)}" }
                }
            } else {
                div { class: "diff-list",
                    for cambio in version.cambios.iter() {
                        div { key: "{cambio.ingrediente}", class: "diff-row",
                            span { class: "diff-name", "{cambio.ingrediente}" }
                            span { class: "diff-old", "{cambio.antes:.0}" }
                            span { class: "diff-arrow", "→" }
                            span { class: "diff-new", "{cambio.despues:.0} {cambio.unidad}" }
                            span {
                                class: if es_baja(cambio) {
                                    "diff-delta diff-delta--baja"
                                } else {
                                    "diff-delta"
                                },
                                "{signo(cambio)}"
                            }
                        }
                    }
                }
            }

            if let Some(origen) = version.origen.clone() {
                p { class: "version-origin", "↳ {origen}" }
            }

            div { class: "version-footer",
                span { class: "version-total", "{total} ml · ratio {version.ratio}" }
                Link {
                    to: Rutas::Ficha { id: version.id.clone() },
                    class: "version-open",
                    {crate::i18n::ahora(Texto::Abrir)}
                }
            }
        }
    }
}
