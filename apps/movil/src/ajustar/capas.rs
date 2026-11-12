//! Capas de C1: agregar ingredientes del catálogo y ajustar cada capa con su deslizador.

use api::ingredientes::IngredienteOpcion;
use api::receta::RecetaCompleta;
use dioxus::prelude::*;

use super::{
    COLOR_RESERVA, CapaAgregada, FACTOR_MAXIMO, MG_AGREGADO_POR_DEFECTO, ML_AGREGADO_POR_DEFECTO,
};

/// Desplegable que agrega una capa nueva del catálogo.
///
/// `disponibles` son las opciones que aún no están en la receta y `catalogo` el catálogo
/// completo del que se copia la metadata de la capa elegida.
#[component]
pub(super) fn AgregarIngrediente(
    disponibles: Vec<IngredienteOpcion>,
    catalogo: Vec<IngredienteOpcion>,
    agregados: Signal<Vec<CapaAgregada>>,
    seleccion_ingrediente: Signal<String>,
) -> Element {
    let mut agregados = agregados;
    let mut seleccion_ingrediente = seleccion_ingrediente;
    let ingredientes_disponibles = disponibles;
    let catalogo_para_agregar = catalogo;

    rsx! {
        section { class: "seccion", "aria-labelledby": "titulo-agregados",
            div { class: "seccion-cabeza",
                p { id: "titulo-agregados", class: "eyebrow", "Nuevos ingredientes" }
                span { class: "mini", "Líquidos o sólidos · 1 g = 1 ml" }
            }

            div { class: "ajustar-agregar",
                label { class: "solo-lector", r#for: "ingrediente-nuevo", "Ingrediente del catálogo" }
                select {
                    id: "ingrediente-nuevo",
                    class: "ajustar-select",
                    value: "{seleccion_ingrediente}",
                    disabled: ingredientes_disponibles.is_empty(),
                    onchange: move |evento| {
                        let id = evento.value();
                        if id.is_empty() {
                            return;
                        }

                        let Some(opcion) = catalogo_para_agregar
                            .iter()
                            .find(|opcion| opcion.id == id)
                            .cloned()
                        else {
                            return;
                        };

                        // El catálogo lo mantiene el equipo: elegir una opción ya crea
                        // la capa; no hay un paso extra de "añadir".
                        if agregados.read().iter().any(|extra| extra.ingrediente_id == id) {
                            seleccion_ingrediente.set(String::new());
                            return;
                        }

                        let cantidad = if opcion.unidad == "mg" {
                            MG_AGREGADO_POR_DEFECTO
                        } else {
                            ML_AGREGADO_POR_DEFECTO
                        };

                        agregados.write().push(CapaAgregada {
                            ingrediente_id: opcion.id,
                            nombre: opcion.nombre,
                            graduacion: opcion.graduacion_pct,
                            color: opcion.color,
                            dilucion: opcion.dilucion,
                            unidad: opcion.unidad,
                            cantidad,
                            factor: 1.0,
                        });
                        seleccion_ingrediente.set(String::new());
                    },
                    option { value: "", "Elige un ingrediente (ml o mg)" }
                    for opcion in ingredientes_disponibles.iter() {
                        if opcion.unidad == "mg" {
                            option {
                                key: "{opcion.id}",
                                value: "{opcion.id}",
                                "{opcion.nombre} · mg"
                            }
                        } else {
                            option {
                                key: "{opcion.id}",
                                value: "{opcion.id}",
                                "{opcion.nombre} · {opcion.graduacion_pct:.0} % ABV"
                            }
                        }
                    }
                }
            }

            if ingredientes_disponibles.is_empty() {
                p { class: "mini", "No hay más ingredientes disponibles en el catálogo." }
            }
        }
    }
}

/// Lista de capas con su deslizador: las de la receta y, encima, las agregadas.
///
/// `capas` es el reparto ya calculado de la bebida ajustada y `comensales` el multiplicador
/// que se muestra en las cantidades.
#[component]
pub(super) fn ListaCapas(
    receta: RecetaCompleta,
    capas: proporciones::Capas,
    factores: Signal<Vec<f64>>,
    agregados: Signal<Vec<CapaAgregada>>,
    comensales: u32,
) -> Element {
    let mut factores = factores;
    let mut agregados = agregados;
    let ingredientes = receta.receta.ingredientes.clone();
    let total_ingredientes = ingredientes.len();
    let factores_actuales = factores.read().clone();
    let agregados_actuales = agregados.read().clone();
    let comensales_actuales = comensales;

    rsx! {
        section { class: "seccion", "aria-labelledby": "titulo-capas",
            div { class: "seccion-cabeza",
                p { id: "titulo-capas", class: "eyebrow", "Capas" }
                span { class: "mini", "{total_ingredientes + agregados_actuales.len()} ingredientes" }
            }

            ul { class: "ajustar-capas",
                for (indice, ingrediente) in ingredientes.iter().enumerate() {
                    {
                        let factor = factores_actuales.get(indice).copied().unwrap_or(1.0);
                        let ml = ingrediente.ml * factor * f64::from(comensales_actuales);
                        let porcentaje = capas
                            .capas
                            .get(indice)
                            .map(|capa| capa.porcentaje)
                            .unwrap_or(0.0);
                        let color = ingrediente
                            .color
                            .clone()
                            .unwrap_or_else(|| COLOR_RESERVA.to_string());
                        let etiqueta = format!("Cantidad de {}", ingrediente.nombre);

                        // La capa del motor siempre es volumen; la unidad original solo
                        // cambia cómo se rotula. Los mg se convierten con 1 g = 1 ml.
                        let es_liquido = receta
                            .componentes
                            .get(indice)
                            .map(|capa| capa.unidad == "ml")
                            .unwrap_or(true);
                        let unidad = if es_liquido { "ml" } else { "mg" };
                        let cantidad = if es_liquido { ml } else { ml * 1000.0 };
                        let cifra = format!("{cantidad:.0} {unidad} · {porcentaje:.0} %");
                        let factor_pct = factor * 100.0;
                        let valor_lector = if es_liquido {
                            format!("{cantidad:.0} mililitros, {factor_pct:.0} por ciento")
                        } else {
                            format!(
                                "{cantidad:.0} miligramos, suma {ml:.2} mililitros, {factor_pct:.0} por ciento"
                            )
                        };

                        rsx! {
                            li { class: "ajustar-capa",
                                div { class: "ajustar-capa-cabeza",
                                    span {
                                        class: "ajustar-capa-punto",
                                        style: "background:{color}",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "ajustar-capa-nombre", "{ingrediente.nombre}" }
                                    span { class: "ajustar-capa-cifra", "{cifra}" }
                                }
                                input {
                                    class: "ajustar-rango",
                                    r#type: "range",
                                    min: "0",
                                    max: "{FACTOR_MAXIMO * 100.0:.0}",
                                    step: "5",
                                    value: "{factor * 100.0:.0}",
                                    aria_label: etiqueta,
                                    "aria-valuetext": "{valor_lector}",
                                    oninput: move |evento| {
                                        let porcentaje = evento.value().parse::<f64>().unwrap_or(100.0);
                                        let mut nuevos = factores.read().clone();
                                        if let Some(slot) = nuevos.get_mut(indice) {
                                            *slot = (porcentaje / 100.0).clamp(0.0, FACTOR_MAXIMO);
                                        }
                                        factores.set(nuevos);
                                    },
                                }
                                div { class: "ajustar-capa-pie",
                                    span { "{factor * 100.0:.0} % de la receta" }
                                    if es_liquido {
                                        span { "{porcentaje:.0} % del total" }
                                    } else {
                                        span { "Suma {ml:.2} ml" }
                                    }
                                }
                            }
                        }
                    }
                }

                for (indice_extra, extra) in agregados_actuales.iter().enumerate() {
                    {
                        let factor = extra.factor;
                        let es_liquido = extra.unidad == "ml";
                        let cantidad =
                            extra.cantidad * factor * f64::from(comensales_actuales);
                        let volumen = api::ml_equivalentes(extra.cantidad, &extra.unidad)
                            * factor
                            * f64::from(comensales_actuales);

                        // Todos los agregados ocupan volumen y por eso comparten el
                        // índice de `capas`; la unidad solo cambia la etiqueta visible.
                        let porcentaje = capas
                            .capas
                            .get(total_ingredientes + indice_extra)
                            .map(|capa| capa.porcentaje)
                            .unwrap_or(0.0);

                        let color = extra
                            .color
                            .clone()
                            .unwrap_or_else(|| COLOR_RESERVA.to_string());
                        let etiqueta = format!("Cantidad de {}", extra.nombre);
                        let nombre = extra.nombre.clone();
                        let unidad = if es_liquido { "ml" } else { "mg" };
                        let cifra = format!("{cantidad:.0} {unidad} · {porcentaje:.0} %");
                        let valor_lector = if es_liquido {
                            format!(
                                "{cantidad:.0} mililitros, {:.0} por ciento",
                                factor * 100.0
                            )
                        } else {
                            format!(
                                "{cantidad:.0} miligramos, suma {volumen:.2} mililitros, {:.0} por ciento",
                                factor * 100.0
                            )
                        };

                        rsx! {
                            li { class: "ajustar-capa",
                                div { class: "ajustar-capa-cabeza ajustar-capa-cabeza--extra",
                                    span {
                                        class: "ajustar-capa-punto",
                                        style: "background:{color}",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "ajustar-capa-nombre", "{nombre}" }
                                    span { class: "ajustar-capa-cifra", "{cifra}" }
                                    button {
                                        class: "ajustar-capa-quitar",
                                        r#type: "button",
                                        aria_label: "Quitar {nombre}",
                                        onclick: move |_| {
                                            agregados.write().remove(indice_extra);
                                        },
                                        "Quitar"
                                    }
                                }
                                input {
                                    class: "ajustar-rango",
                                    r#type: "range",
                                    min: "0",
                                    max: "{FACTOR_MAXIMO * 100.0:.0}",
                                    step: "5",
                                    value: "{factor * 100.0:.0}",
                                    aria_label: etiqueta,
                                    "aria-valuetext": "{valor_lector}",
                                    oninput: move |evento| {
                                        let porcentaje = evento.value().parse::<f64>().unwrap_or(100.0);
                                        let mut nuevos = agregados.read().clone();
                                        if let Some(slot) = nuevos.get_mut(indice_extra) {
                                            slot.factor =
                                                (porcentaje / 100.0).clamp(0.0, FACTOR_MAXIMO);
                                        }
                                        agregados.set(nuevos);
                                    },
                                }
                                div { class: "ajustar-capa-pie",
                                    span { "{factor * 100.0:.0} % del agregado" }
                                    if es_liquido {
                                        span { "{porcentaje:.0} % del total" }
                                    } else {
                                        span { "Suma {volumen:.2} ml" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
