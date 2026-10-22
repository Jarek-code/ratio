//! Pantalla B6: historial de versiones de una bebida.
//!
//! Muestra la receta original y las versiones derivadas en orden cronológico,
//! con el cambio de cada ingrediente respecto a su versión de origen. Por ahora
//! trabaja con datos de demostración; la consulta real se hará contra
//! `public.receta` y `public.componente_receta`.

use dioxus::prelude::*;
use dioxus::router::Link;

use crate::rutas::Rutas;
use crate::marca::Marca;

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
    fn incluye(self, version: &VersionReceta) -> bool {
        match self {
            Self::Todas => true,
            Self::Publicas => version.visibilidad == Visibilidad::Publica,
            Self::Privadas => version.visibilidad == Visibilidad::Privada,
        }
    }
}

/// Visibilidad de una versión dentro del recetario.
#[derive(Clone, Copy, PartialEq)]
enum Visibilidad {
    /// Cualquier persona puede ver la versión en el recetario.
    Publica,
    /// La versión solo es visible para su autor.
    Privada,
}

impl Visibilidad {
    /// Etiqueta legible para la línea de metadatos.
    fn etiqueta(self) -> &'static str {
        match self {
            Self::Publica => "Pública",
            Self::Privada => "Privada",
        }
    }
}

/// Cambio de cantidad de un ingrediente entre dos versiones.
#[derive(Clone, PartialEq)]
struct Cambio {
    /// Nombre visible del ingrediente.
    ingrediente: &'static str,
    /// Cantidad en la versión anterior.
    antes: u16,
    /// Cantidad en la versión que se está mostrando.
    despues: u16,
    /// Unidad de las cantidades, por ejemplo `ml`.
    unidad: &'static str,
}

impl Cambio {
    /// Diferencia entre la versión actual y la anterior.
    fn delta(&self) -> i16 {
        self.despues as i16 - self.antes as i16
    }

    /// Texto con signo para el indicador de cambio.
    fn signo(&self) -> String {
        let delta = self.delta();
        if delta > 0 {
            format!("+{delta}")
        } else {
            format!("{delta}")
        }
    }

    /// Permite pintar de distinto color las bajadas.
    fn es_baja(&self) -> bool {
        self.delta() < 0
    }
}

/// Versión de una bebida tal como se pinta en el historial.
#[derive(Clone, PartialEq)]
struct VersionReceta {
    /// Nombre de esta versión.
    nombre: &'static str,
    /// Persona que la creó.
    autor: &'static str,
    /// Fecha legible para la línea de metadatos.
    fecha: &'static str,
    /// Si la versión es pública o privada.
    visibilidad: Visibilidad,
    /// Nombre de la versión de la que deriva; `None` si es la original.
    origen: Option<&'static str>,
    /// Volumen total de la bebida en mililitros.
    total_ml: u16,
    /// Proporción principal ya formateada, por ejemplo `1:4,5`.
    ratio: &'static str,
    /// Cambios frente a la versión de origen; vacío en la original.
    cambios: Vec<Cambio>,
}

/// Datos de demostración basados en el modelo de `receta` y `componente_receta`.
///
/// La primera versión es la original; las demás son derivadas y guardan el
/// nombre de su origen. El vector está en orden cronológico.
fn versiones_demo() -> Vec<VersionReceta> {
    vec![
        VersionReceta {
            nombre: "Mocha de la casa",
            autor: "Ana R.",
            fecha: "12 oct 2026",
            visibilidad: Visibilidad::Publica,
            origen: None,
            total_ml: 240,
            ratio: "1:4,5",
            cambios: Vec::new(),
        },
        VersionReceta {
            nombre: "Mocha más lácteo",
            autor: "Luis M.",
            fecha: "18 oct 2026",
            visibilidad: Visibilidad::Publica,
            origen: Some("Mocha de la casa"),
            total_ml: 290,
            ratio: "1:6",
            cambios: vec![
                Cambio {
                    ingrediente: "Leche",
                    antes: 110,
                    despues: 150,
                    unidad: "ml",
                },
                Cambio {
                    ingrediente: "Espuma",
                    antes: 60,
                    despues: 70,
                    unidad: "ml",
                },
            ],
        },
        VersionReceta {
            nombre: "Mocha intenso",
            autor: "Ana R.",
            fecha: "21 oct 2026",
            visibilidad: Visibilidad::Privada,
            origen: Some("Mocha de la casa"),
            total_ml: 230,
            ratio: "1:3,8",
            cambios: vec![
                Cambio {
                    ingrediente: "Espresso",
                    antes: 40,
                    despues: 60,
                    unidad: "ml",
                },
                Cambio {
                    ingrediente: "Leche",
                    antes: 110,
                    despues: 80,
                    unidad: "ml",
                },
            ],
        },
        VersionReceta {
            nombre: "Mocha sin azúcar",
            autor: "Sofía G.",
            fecha: "22 oct 2026",
            visibilidad: Visibilidad::Publica,
            origen: Some("Mocha más lácteo"),
            total_ml: 260,
            ratio: "1:5,5",
            cambios: vec![
                Cambio {
                    ingrediente: "Chocolate",
                    antes: 30,
                    despues: 10,
                    unidad: "ml",
                },
                Cambio {
                    ingrediente: "Espuma",
                    antes: 70,
                    despues: 60,
                    unidad: "ml",
                },
            ],
        },
    ]
}

/// Pantalla B6: historial de versiones de una bebida.
///
/// Mantiene el filtro y el aviso en estado local. El filtrado se hace en
/// memoria para que los chips respondan de inmediato mientras no existe la
/// consulta a la base de datos.
#[component]
pub fn Historial() -> Element {
    // Filtro activo; empieza mostrando todas las versiones.
    let mut filtro = use_signal(|| Filtro::Todas);

    // Mensaje temporal para las acciones que aún no tienen backend.
    let mut aviso = use_signal(|| None::<String>);

    // Datos de demostración; más adelante vendrán de la base de datos.
    let versiones = versiones_demo();

    // Número de versiones que cumplen el filtro; alimenta el contador de chips.
    let visibles = versiones
        .iter()
        .filter(|version| filtro().incluye(version))
        .count();

    rsx! {
        style { {crate::styles::LOGIN}{crate::styles::HISTORIAL} }

        main { class: "historial-page",
            section { class: "historial-shell",
                // Cabecera con regreso al acceso y marca de la app.
                header { class: "historial-top",
                    // `Link` navega a la ruta interna sin recargar la página.
                    Link {
                        to: Rutas::Entrar {},
                        class: "back",
                        aria_label: "Volver al acceso",
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
                    p { "Mocha de la casa · 4 versiones" }
                }

                // Resumen de la receta original y sus datos principales.
                section { class: "recipe-summary", aria_label: "Receta original",
                    div { class: "summary-mark", Marca {} }
                    div { class: "recipe-summary__body",
                        p { class: "summary-label", "Receta original" }
                        h2 { "Mocha de la casa" }
                        p { "Ana R. · 12 oct 2026" }
                    }
                    div { class: "recipe-summary__metrics",
                        div { class: "metric",
                            strong { "240 ml" }
                            span { "Total" }
                        }
                        div { class: "metric",
                            strong { "4" }
                            span { "Versiones" }
                        }
                    }
                }

                // Chips de filtrado por visibilidad.
                nav { class: "filters", aria_label: "Filtrar versiones",
                    // Cada chip cambia el filtro y vuelve a calcular la lista.
                    for opcion in [Filtro::Todas, Filtro::Publicas, Filtro::Privadas] {
                        button {
                            class: if filtro() == opcion { "chip chip--active" } else { "chip" },
                            r#type: "button",
                            onclick: move |_| filtro.set(opcion),
                            "{opcion.etiqueta()}"
                        }
                    }
                    span { class: "filters__count", "{visibles} de 4" }
                }

                // Línea de tiempo de versiones.
                section { class: "timeline", aria_label: "Línea de tiempo de versiones",
                    // Se enumeran solo las versiones visibles para que el índice
                    // de la tarjeta coincida con el orden que ve la persona.
                    for (indice, version) in versiones
                        .iter()
                        .filter(|version| filtro().incluye(version))
                        .enumerate()
                    {
                        VersionCard {
                            key: "{version.nombre}",
                            indice: indice + 1,
                            version: version.clone(),
                            // La ficha todavía no existe: se muestra un aviso.
                            al_abrir: move |nombre: &'static str| {
                                aviso.set(Some(format!(
                                    "«{nombre}» se abrirá cuando la ficha de receta esté conectada."
                                )))
                            },
                        }
                    }
                }

                // Pie de la pantalla.
                footer { class: "historial-foot",
                    span { "Toca una versión para ver su ficha." }
                    span { "Actualizado hoy" }
                }
            }

            // Aviso temporal de las acciones pendientes de backend.
            if let Some(mensaje) = aviso() {
                div { class: "toast", role: "status", aria_live: "polite", "{mensaje}" }
            }
        }
    }
}

/// Tarjeta de una versión dentro de la línea de tiempo.
///
/// `indice` es la posición visible (empieza en 1), `version` contiene los datos
/// que se pintan y `al_abrir` avisa a la pantalla cuando se pulsa «Abrir».
#[component]
fn VersionCard(
    indice: usize,
    version: VersionReceta,
    al_abrir: EventHandler<&'static str>,
) -> Element {
    // Número de dos dígitos para el marcador de la línea de tiempo.
    let numero = format!("{indice:02}");

    // Se copia antes del `rsx!` porque el manejador necesita un `'static`.
    let nombre_version = version.nombre;

    rsx! {
        article {
            // La original recibe un fondo distinto para separarla de las derivadas.
            class: if version.origen.is_none() { "version-card version-card--origin" } else { "version-card" },
            div { class: "version-head",
                span { class: "version-index", "{numero}" }
                h2 { class: "version-title", "{version.nombre}" }
                // Badge que identifica si la versión es original o derivada.
                span {
                    class: if version.origen.is_none() {
                        "version-badge version-badge--original"
                    } else {
                        "version-badge version-badge--derivada"
                    },
                    if version.origen.is_none() { "Original" } else { "Derivada" }
                }
            }

            // Autor, fecha y visibilidad de la versión.
            p { class: "version-meta",
                "{version.autor} · {version.fecha} · "
                span {
                    class: if version.visibilidad == Visibilidad::Privada {
                        "visibilidad visibilidad--privada"
                    } else {
                        "visibilidad"
                    },
                    "{version.visibilidad.etiqueta()}"
                }
            }

            // La original no tiene diff; las derivadas muestran sus cambios.
            // Cada fila compara la cantidad anterior con la nueva y resalta bajadas.
            if version.cambios.is_empty() {
                p { class: "version-note", "Receta original · sin cambios" }
            } else {
                div { class: "diff-list",
                    for cambio in version.cambios.iter() {
                        div { class: "diff-row",
                            span { class: "diff-name", "{cambio.ingrediente}" }
                            span { class: "diff-old", "{cambio.antes}" }
                            span { class: "diff-arrow", "→" }
                            span { class: "diff-new", "{cambio.despues} {cambio.unidad}" }
                            span {
                                class: if cambio.es_baja() {
                                    "diff-delta diff-delta--baja"
                                } else {
                                    "diff-delta"
                                },
                                "{cambio.signo()}"
                            }
                        }
                    }
                }
            }

            // Procedencia de la versión dentro del historial.
            if let Some(origen) = version.origen {
                p { class: "version-origin", "↳ {origen}" }
            }

            // Total, ratio y acción de abrir la ficha.
            div { class: "version-footer",
                span { class: "version-total", "{version.total_ml} ml · ratio {version.ratio}" }
                // Avisa a `Historial` con el nombre de esta versión.
                button {
                    class: "version-open",
                    r#type: "button",
                    onclick: move |_| al_abrir.call(nombre_version),
                    "Abrir →"
                }
            }
        }
    }
}
