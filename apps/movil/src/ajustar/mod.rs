//! Pantalla C1: ajustar las proporciones de una receta y ver el vaso en vivo.
//!
//! El editor trabaja sobre factores de cada ingrediente, no sobre mililitros absolutos.
//! Así el escalado por comensales y los presets pueden cambiar el total sin perder el
//! ajuste individual que hizo la persona. El vaso y la graduación se recalculan con el
//! mismo motor que usa la ficha (B3).
//!
//! La escala global se expresa en mililitros por comensal y tiene un tope de 500 ml.
//! Además de las capas de la receta, C1 permite añadir ingredientes del catálogo (ml o
//! mg). De momento los sólidos se convierten con `1 g = 1 ml` para que también sumen
//! volumen mientras el catálogo termina de definirse.


mod atajos;
mod capas;
mod guardar;

use api::ingredientes::{IngredienteOpcion, catalogo as catalogo_ingredientes};
use api::receta::RecetaCompleta;
use api::vasos::{VasoOpcion, vasos as catalogo_vasos};
use dioxus::prelude::*;

use crate::rutas::Rutas;

/// Hoja de estilos propia del editor de proporciones.
const CSS_AJUSTAR: Asset = asset!("/assets/ajustar.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Color de reserva del punto de la leyenda cuando la capa no trae uno.
const COLOR_RESERVA: &str = "#8A6A4A";

/// Porcentaje máximo que permite cada control individual.
const FACTOR_MAXIMO: f64 = 3.0;

/// Comensales máximos para el escalado rápido.
const COMENSALES_MAXIMOS: u32 = 12;

/// Tope de la escala global, en mililitros.
///
/// Es un límite de producto: el vaso puede ser mayor, pero C1 nunca propone más de
/// medio litro por comensal para que el ajuste siga siendo legible.
const VOLUMEN_MAXIMO_ML: f64 = 500.0;

/// Cantidad inicial de un ingrediente añadido, en mililitros por comensal.
///
/// Se elige una porción pequeña para que la capa nueva no desborde el vaso antes de que
/// la persona la ajuste con su deslizador.
const ML_AGREGADO_POR_DEFECTO: f64 = 30.0;

/// Cantidad inicial de un ingrediente añadido en miligramos.
///
/// Se parte de 1 g para que, con la equivalencia temporal `1 g = 1 ml`, la capa nueva
/// suba el volumen de forma visible antes de ajustarla con su deslizador.
const MG_AGREGADO_POR_DEFECTO: f64 = 1000.0;

/// Capa que nació del desplegable de nuevos ingredientes.
///
/// Guarda la metadata del catálogo para dibujarla o medirla, más su unidad, su cantidad
/// base y el factor que comparten los controles. Los mg suman volumen con 1 g = 1 ml.
#[derive(Debug, Clone, PartialEq)]
struct CapaAgregada {
    /// Ingrediente del catálogo.
    ingrediente_id: String,
    /// Nombre visible de la capa.
    nombre: String,
    /// Alcohol por volumen del ingrediente.
    graduacion: f64,
    /// Color de la capa, si el catálogo lo define.
    color: Option<String>,
    /// Si el ingrediente diluye la bebida.
    dilucion: bool,
    /// Unidad de la cantidad: "ml" o "mg".
    unidad: String,
    /// Cantidad base por comensal, en la unidad del catálogo.
    cantidad: f64,
    /// Factor del deslizador; 1.0 es la cantidad base.
    factor: f64,
}

/// Pantalla C1: carga la receta, el catálogo de vasos y el de ingredientes.
#[component]
pub fn PantallaAjustar(id: String) -> Element {
    let id_recurso = id.clone();
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut recurso = use_resource(move || {
        let id = id_recurso.clone();
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        async move {
            let receta = crate::local::receta(id, token, cuenta).await?;
            let vasos = catalogo_vasos().await?;
            let ingredientes = catalogo_ingredientes().await?;
            Ok::<_, ServerFnError>((receta, vasos, ingredientes))
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_AJUSTAR }

        main { class: "ajustar",
            BarraSuperior { id: id.clone() }

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando las proporciones…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos cargar el editor: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| recurso.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok((None, _, _))) => rsx! {
                    p { class: "estado", "Esta receta no existe o ya no está publicada." }
                },
                Some(Ok((Some(receta), vasos, ingredientes))) => rsx! {
                    Editor {
                        receta: receta.clone(),
                        vasos: vasos.clone(),
                        ingredientes_catalogo: ingredientes.clone(),
                    }
                },
            }
        }
    }
}

/// Barra superior con la vuelta a la ficha de la receta.
#[component]
fn BarraSuperior(id: String) -> Element {
    rsx! {
        header { class: "ajustar-barra",
            Link {
                to: Rutas::Ficha { id },
                class: "ajustar-volver",
                aria_label: "Volver a la ficha de la receta",
                "‹ Receta"
            }
        }
    }
}

/// Editor de proporciones: vista del vaso, totales, vaso, comensales, escala y capas.
#[component]
fn Editor(
    receta: RecetaCompleta,
    vasos: Vec<VasoOpcion>,
    ingredientes_catalogo: Vec<IngredienteOpcion>,
) -> Element {
    let ingredientes = receta.receta.ingredientes.clone();
    let total_ingredientes = ingredientes.len();

    // Si el vaso de la receta no está en el catálogo, se ofrece igual como primera opción.
    let mut opciones = vasos;
    let indice_inicial = match opciones
        .iter()
        .position(|opcion| opcion.id == receta.vaso_id)
    {
        Some(indice) => indice,
        None => {
            opciones.insert(
                0,
                VasoOpcion {
                    id: receta.vaso_id.clone(),
                    nombre: receta.vaso_nombre.clone(),
                    vaso: receta.receta.vaso.clone(),
                },
            );
            0
        }
    };

    let factores = use_signal(|| vec![1.0_f64; total_ingredientes]);
    let agregados = use_signal(Vec::<CapaAgregada>::new);
    let seleccion_ingrediente = use_signal(String::new);
    let mut comensales = use_signal(|| 1_u32);
    let mut indice_vaso = use_signal(move || indice_inicial);

    let Some(vaso_actual) = opciones.get(*indice_vaso.read()).cloned() else {
        return rsx! {
            p { class: "estado", "No hay vasos disponibles para ajustar la receta." }
        };
    };

    let factores_actuales = factores.read().clone();
    let agregados_actuales = agregados.read().clone();
    let comensales_actuales = *comensales.read();
    let indice_vaso_actual = *indice_vaso.read();
    let mut ajustada = receta.receta.clone();
    ajustada.vaso = vaso_actual.vaso.clone();
    ajustada.capacidad_ml = vaso_actual.vaso.capacidad_ml;

    for (indice, componente) in ajustada.ingredientes.iter_mut().enumerate() {
        let factor = factores_actuales.get(indice).copied().unwrap_or(1.0);
        componente.ml =
            receta.receta.ingredientes[indice].ml * factor * f64::from(comensales_actuales);
    }

    // Las capas agregadas se suman al final de la pila, encima de las de la receta.
    // De momento 1 g cuenta como 1 ml para que los sólidos también suban el volumen.
    for extra in &agregados_actuales {
        ajustada.ingredientes.push(proporciones::Componente {
            nombre: extra.nombre.clone(),
            ml: api::ml_equivalentes(extra.cantidad, &extra.unidad)
                * extra.factor
                * f64::from(comensales_actuales),
            graduacion: extra.graduacion,
            dilucion: extra.dilucion,
            color: extra.color.clone(),
            superficie: None,
        });
    }

    let capas = proporciones::capas(&ajustada);
    let graduacion = proporciones::graduacion(&ajustada);
    let svg = proporciones::svg(
        &ajustada,
        &proporciones::Opciones {
            escalas: true,
            mini: false,
        },
    );
    let capacidad = ajustada.capacidad_ml.max(1.0);
    let porcentaje_vaso = capas.total_ml / capacidad * 100.0;
    let descripcion = descripcion(&ajustada);

    // La escala global se expresa en mililitros por comensal: el deslizador llega hasta
    // 500 ml y reparte ese volumen entre todas las capas de forma proporcional. Los
    // comensales multiplican después el total, como en el resto de la pantalla.
    let total_base: f64 = receta.receta.ingredientes.iter().map(|c| c.ml).sum::<f64>()
        + agregados_actuales
            .iter()
            .map(|extra| api::ml_equivalentes(extra.cantidad, &extra.unidad))
            .sum::<f64>();
    let volumen_porcion = capas.total_ml / f64::from(comensales_actuales.max(1));
    let volumen_escala = volumen_porcion.clamp(0.0, VOLUMEN_MAXIMO_ML);

    // El desplegable oculta los ingredientes que ya están en la receta o ya se agregaron.
    let mut ids_en_uso: Vec<String> = receta
        .componentes
        .iter()
        .map(|componente| componente.ingrediente_id.clone())
        .collect();
    ids_en_uso.extend(
        agregados_actuales
            .iter()
            .map(|extra| extra.ingrediente_id.clone()),
    );
    let ingredientes_disponibles: Vec<IngredienteOpcion> = ingredientes_catalogo
        .iter()
        .filter(|opcion| !ids_en_uso.contains(&opcion.id))
        .cloned()
        .collect();

    rsx! {
        header { class: "ajustar-cabeza",
            p { class: "eyebrow", "Ajustar proporciones" }
            h1 { "{receta.receta.nombre}" }
            p { class: "ajustar-autor", "por {receta.autor}" }
        }

        article { class: "tarjeta ajustar-preview",
            div {
                class: "vaso ajustar-vaso",
                role: "img",
                "aria-label": descripcion,
                dangerous_inner_html: svg,
            }

            dl { class: "ajustar-totales", "aria-live": "polite",
                div { class: "dato",
                    dt { "Total" }
                    dd { "{capas.total_ml:.0} ml" }
                }
                div { class: "dato",
                    dt { "Del vaso" }
                    dd { "{porcentaje_vaso:.0} %" }
                }
                div { class: "dato",
                    dt { "Graduación" }
                    dd {
                        if graduacion.abv > 0.0 {
                            "{graduacion.abv:.1} % ABV"
                        } else {
                            "Sin alcohol"
                        }
                    }
                }
                div { class: "dato",
                    dt { "Intensidad base" }
                    dd {
                        if let Some(intensidad) = receta.intensidad {
                            "{intensidad} de 5"
                        } else {
                            "—"
                        }
                    }
                }
            }

            if capas.total_ml > capacidad {
                p { class: "ajustar-aviso ajustar-aviso--error", role: "alert",
                    "No cabe: sobran {(capas.total_ml - capacidad):.0} ml. Baja una capa o elige un vaso más grande."
                }
            } else if capas.total_ml > capacidad * proporciones::LLENADO_MAX {
                p { class: "ajustar-aviso", role: "status",
                    "El vaso queda al límite: el dibujo se corta al 94 % de la capacidad."
                }
            }

            if capas.total_ml <= 0.0 {
                p { class: "estado", "Todas las capas están a cero. Sube alguna para ver la bebida." }
            }
        }


        atajos::Atajos {
            ingredientes: ingredientes.clone(),
            factores,
            agregados,
            total_base,
            volumen_escala,
        }

        section { class: "seccion", "aria-labelledby": "titulo-comensales",
            div { class: "seccion-cabeza",
                p { id: "titulo-comensales", class: "eyebrow", "Comensales" }
                span { class: "mini", "Multiplica todas las capas" }
            }

            div { class: "ajustar-comensales",
                div { class: "ajustar-comensales-texto",
                    strong { "{comensales_actuales}" }
                    span {
                        if comensales_actuales == 1 { " comensal" } else { " comensales" }
                    }
                }
                div { class: "contador",
                    button {
                        r#type: "button",
                        disabled: comensales_actuales <= 1,
                        aria_label: "Quitar un comensal",
                        onclick: move |_| {
                            let siguiente = (*comensales.read()).saturating_sub(1).max(1);
                            comensales.set(siguiente);
                        },
                        "−"
                    }
                    strong { "{comensales_actuales}" }
                    button {
                        r#type: "button",
                        disabled: comensales_actuales >= COMENSALES_MAXIMOS,
                        aria_label: "Añadir un comensal",
                        onclick: move |_| {
                            let siguiente = (*comensales.read() + 1).min(COMENSALES_MAXIMOS);
                            comensales.set(siguiente);
                        },
                        "+"
                    }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-vaso",
            div { class: "seccion-cabeza",
                p { id: "titulo-vaso", class: "eyebrow", "Vaso" }
                span { class: "mini", "{vaso_actual.nombre} · {vaso_actual.vaso.capacidad_ml:.0} ml" }
            }

            label { class: "solo-lector", r#for: "vaso-editor", "Elegir el vaso" }
            select {
                id: "vaso-editor",
                class: "ajustar-select",
                value: "{indice_vaso_actual}",
                onchange: move |evento| {
                    if let Ok(indice) = evento.value().parse::<usize>()
                        && indice < opciones.len()
                    {
                        indice_vaso.set(indice);
                    }
                },
                for (indice, opcion) in opciones.iter().enumerate() {
                    option {
                        key: "{opcion.id}",
                        value: "{indice}",
                        "{opcion.nombre} · {opcion.vaso.capacidad_ml:.0} ml"
                    }
                }
            }
        }

        capas::AgregarIngrediente {
            disponibles: ingredientes_disponibles,
            catalogo: ingredientes_catalogo,
            agregados,
            seleccion_ingrediente,
        }

        capas::ListaCapas {
            receta: receta.clone(),
            capas: capas.clone(),
            factores,
            agregados,
            comensales: comensales_actuales,
        }

        guardar::Acciones {
            receta: receta.clone(),
            vaso: vaso_actual.clone(),
            factores,
            agregados,
            seleccion_ingrediente,
            comensales,
            indice_vaso,
            indice_inicial,
        }
    }
}

/// Texto accesible del vaso: nombre y porcentaje de cada capa.
fn descripcion(receta: &proporciones::Receta) -> String {
    let capas = proporciones::capas(receta);
    let partes: Vec<String> = capas
        .capas
        .iter()
        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
        .collect();

    format!("{}: {}", receta.nombre, partes.join(", "))
}
