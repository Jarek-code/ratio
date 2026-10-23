//! Pantalla B6: historial de versiones de una bebida.
//!
//! Muestra la receta original y las versiones derivadas en orden cronológico,
//! con el cambio de cada ingrediente respecto a su versión de origen. Los datos
//! salen de `receta.receta_origen_id` y de `componente_receta`.

use dioxus::prelude::*;
use dioxus::router::Link;

use api::historial::{CambioHistorial, VersionHistorial, versiones};

use crate::marca::Marca;
use crate::rutas::Rutas;
use crate::sesion::SenalSesion;

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
            Self::Todas => "Todas",
            Self::Publicas => "Públicas",
            Self::Privadas => "Privadas",
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
    if version.publica { "Pública" } else { "Privada" }
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
    versiones.iter().find(|version| version.origen.is_none())
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
        versiones(id_recurso, token)
    }));

    rsx! {
        style { {crate::styles::LOGIN}{crate::styles::HISTORIAL} }

        main { class: "historial-page",
            section { class: "historial-shell",
                header { class: "historial-top",
                    Link {
                        to: Rutas::Ficha { id: id.clone() },
                        class: "back",
                        aria_label: "Volver a la ficha de la receta",
                        "←"
                    }
                    div { class: "brand-mini",
                        Marca {}
                        span { "Ratio" }
                    }
                }

                div { class: "historial-heading",
                    p { class: "eyebrow", "Recetario · B6" }
                    h1 { "Historial de versiones" }
                }

                match &*versiones_res.read() {
                    None => rsx! {
                        p { class: "historial-estado", role: "status", "Cargando el historial…" }
                    },
                    Some(Err(causa)) => rsx! {
                        div { class: "historial-estado", role: "alert",
                            p { "No pudimos traer el historial: {causa}" }
                            button {
                                class: "version-open",
                                r#type: "button",
                                onclick: move |_| versiones_res.restart(),
                                "Reintentar"
                            }
                        }
                    },
                    Some(Ok(lista)) if lista.is_empty() => rsx! {
                        p { class: "historial-estado", "Esta bebida todavía no tiene versiones publicadas." }
                    },
                    Some(Ok(lista)) => rsx! {
                        {
                            let raiz = original(lista);
                            let total = lista.len();
                            let visibles = lista.iter().filter(|version| filtro().incluye(version)).count();

                            rsx! {
                                if let Some(raiz) = raiz {
                                    section { class: "recipe-summary", aria_label: "Receta original",
                                        div { class: "summary-mark", Marca {} }
                                        div { class: "recipe-summary__body",
                                            p { class: "summary-label", "Receta original" }
                                            h2 { "{raiz.nombre}" }
                                            p { "{raiz.autor} · {raiz.fecha}" }
                                        }
                                        div { class: "recipe-summary__metrics",
                                            div { class: "metric",
                                                strong { "{raiz.total_ml:.0} ml" }
                                                span { "Total" }
                                            }
                                            div { class: "metric",
                                                strong { "{total}" }
                                                span { if total == 1 { "Versión" } else { "Versiones" } }
                                            }
                                        }
                                    }
                                }

                                nav { class: "filters", aria_label: "Filtrar versiones",
                                    for opcion in [Filtro::Todas, Filtro::Publicas, Filtro::Privadas] {
                                        button {
                                            key: "{opcion.etiqueta()}",
                                            class: if filtro() == opcion { "chip chip--active" } else { "chip" },
                                            r#type: "button",
                                            onclick: move |_| filtro.set(opcion),
                                            "{opcion.etiqueta()}"
                                        }
                                    }
                                    span { class: "filters__count", "{visibles} de {total}" }
                                }

                                section { class: "timeline", aria_label: "Línea de tiempo de versiones",
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
                                    span { "Toca «Abrir» para ver la ficha de esa versión." }
                                    span { "Datos de la base de Ratio" }
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
            class: if version.origen.is_none() { "version-card version-card--origin" } else { "version-card" },
            div { class: "version-head",
                span { class: "version-index", "{numero}" }
                h2 { class: "version-title", "{version.nombre}" }
                span {
                    class: if version.origen.is_none() {
                        "version-badge version-badge--original"
                    } else {
                        "version-badge version-badge--derivada"
                    },
                    if version.origen.is_none() { "Original" } else { "Derivada" }
                }
            }

            p { class: "version-meta",
                "{version.autor} · {version.fecha} · "
                span {
                    class: if version.publica { "visibilidad" } else { "visibilidad visibilidad--privada" },
                    "{visibilidad(&version)}"
                }
            }

            if version.cambios.is_empty() {
                p { class: "version-note", "Receta original · sin cambios" }
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
                    "Abrir →"
                }
            }
        }
    }
}
