//! Pantalla C1: ajustar las proporciones de una receta y ver el vaso en vivo.
//!
//! El editor trabaja sobre factores de cada ingrediente, no sobre mililitros absolutos.
//! Así la escala global y los presets pueden cambiar el total sin perder el
//! ajuste individual que hizo la persona. El vaso y la graduación se recalculan con el
//! mismo motor que usa la ficha (B3).
//!
//! El vaso siempre muestra una sola bebida. La escala global se expresa en mililitros
//! y tiene un tope de 500 ml.
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

use crate::i18n::Texto;
use crate::red::AvisoRed;
use crate::rutas::Rutas; // Textos traducibles

/// Hoja de estilos propia del editor de proporciones.
const CSS_AJUSTAR: Asset = asset!("/assets/ajustar.css");
/// Campos, selectores y tarjetas de formulario.
const CSS_FORMULARIO: Asset = asset!("/assets/formulario.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Color de reserva del punto de la leyenda cuando la capa no trae uno.
const COLOR_RESERVA: &str = "#8A6A4A";

/// Porcentaje máximo que permite cada control individual.
const FACTOR_MAXIMO: f64 = 3.0;

/// Tope de la escala global, en mililitros.
///
/// Es un límite de producto: el vaso puede ser mayor, pero C1 nunca propone más de
/// medio litro por bebida para que el ajuste siga siendo legible.
const VOLUMEN_MAXIMO_ML: f64 = 500.0;

/// Cantidad inicial de un ingrediente añadido, en mililitros.
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
    /// Cantidad base, en la unidad del catálogo.
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
            // Las tres peticiones llevan plazo y reintentos: el editor no se queda cargando
            let receta = crate::red::con_reintentos(move || {
                crate::local::receta(id.clone(), token.clone(), cuenta.clone())
            })
            .await?;
            let vasos = crate::red::con_reintentos(catalogo_vasos).await?;
            let ingredientes = crate::red::con_reintentos(catalogo_ingredientes).await?;
            Ok::<_, ServerFnError>((receta, vasos, ingredientes))
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_AJUSTAR }
        document::Stylesheet { href: CSS_FORMULARIO }

        main { class: "ajustar",
            BarraSuperior { id: id.clone() }

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoProporciones)}" }
                },
                Some(Err(causa)) => rsx! {
                    AvisoRed {
                        causa: causa.clone(),
                        reintentar: move |_| recurso.restart(),
                    }
                },
                Some(Ok((None, _, _))) => rsx! {
                    p { class: "estado", "{crate::i18n::ahora(Texto::RecetaNoPublicada)}" }
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
            crate::rutas::Volver {
                etiqueta: crate::i18n::ahora(Texto::VolverALaFichaReceta).to_string(),
                texto: Some(crate::i18n::ahora(Texto::Receta).to_string()),
                respaldo: Rutas::Ficha { id },
            }
        }
    }
}

/// Editor de proporciones: vista del vaso, totales, vaso, escala y capas.
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
    // La escala es un factor aparte: mover una capa no la reinicia ni la pierde
    let escala = use_signal(|| 1.0_f64);
    let agregados = use_signal(Vec::<CapaAgregada>::new);
    let seleccion_ingrediente = use_signal(String::new);
    let mut indice_vaso = use_signal(move || indice_inicial);

    let Some(vaso_actual) = opciones.get(*indice_vaso.read()).cloned() else {
        return rsx! {
            p { class: "estado", "{crate::i18n::ahora(Texto::NoHayVasos)}" }
        };
    };

    let factores_actuales = factores.read().clone();
    let agregados_actuales = agregados.read().clone();
    let indice_vaso_actual = *indice_vaso.read();
    let mut ajustada = receta.receta.clone();
    ajustada.vaso = vaso_actual.vaso.clone();
    ajustada.capacidad_ml = vaso_actual.vaso.capacidad_ml;

    let escala_actual = *escala.read();
    aplicar_factores(
        &mut ajustada,
        &receta.receta,
        &factores_actuales,
        escala_actual,
    );

    // Las capas agregadas se suman al final de la pila, encima de las de la receta.
    // De momento 1 g cuenta como 1 ml para que los sólidos también suban el volumen.
    for extra in &agregados_actuales {
        ajustada.ingredientes.push(proporciones::Componente {
            nombre: extra.nombre.clone(),
            ml: api::ml_equivalentes(extra.cantidad, &extra.unidad) * extra.factor * escala_actual,
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
    // Aviso de lo que sobra, ya en el idioma elegido
    let aviso_sobran = crate::i18n::con(
        crate::i18n::actual(),
        Texto::NoCabeSobran,
        &[&format!("{:.0}", capas.total_ml - capacidad)],
    );

    // La escala global se expresa en mililitros: el deslizador llega hasta 500 ml y
    // reparte ese volumen entre todas las capas de forma proporcional.
    // Volumen de la receta tal como está, sin la escala global: sirve para convertir
    // mililitros del deslizador a un factor de escala.
    let volumen_sin_escala: f64 = receta
        .receta
        .ingredientes
        .iter()
        .enumerate()
        .map(|(indice, capa)| capa.ml * factores_actuales.get(indice).copied().unwrap_or(1.0))
        .sum::<f64>()
        + agregados_actuales
            .iter()
            .map(|extra| api::ml_equivalentes(extra.cantidad, &extra.unidad) * extra.factor)
            .sum::<f64>();
    let volumen_escala = capas.total_ml.clamp(0.0, VOLUMEN_MAXIMO_ML);

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
            p { class: "eyebrow", "{crate::i18n::ahora(Texto::AjustarProporciones)}" }
            h1 { "{receta.receta.nombre}" }
            p { class: "ajustar-autor", "{crate::i18n::con(crate::i18n::actual(), Texto::PorAutor, &[&receta.autor])}" }
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
                    dt { "{crate::i18n::ahora(Texto::Total)}" }
                    dd { "{capas.total_ml:.0} ml" }
                }
                div { class: "dato",
                    dt { "{crate::i18n::ahora(Texto::DelVaso)}" }
                    dd { "{porcentaje_vaso:.0} %" }
                }
                div { class: "dato",
                    dt { "{crate::i18n::ahora(Texto::Graduacion)}" }
                    dd {
                        if graduacion.abv > 0.0 {
                            "{graduacion.abv:.1} % ABV"
                        } else {
                            {crate::i18n::ahora(Texto::SinAlcohol)}
                        }
                    }
                }
                div { class: "dato",
                    dt { "{crate::i18n::ahora(Texto::IntensidadBase)}" }
                    dd {
                        if let Some(intensidad) = receta.intensidad {
                            "{crate::i18n::con(crate::i18n::actual(), Texto::NivelDeCinco, &[&intensidad.to_string()])}"
                        } else {
                            "—"
                        }
                    }
                }
            }

            if capas.total_ml > capacidad {
                p { class: "ajustar-aviso ajustar-aviso--error", role: "alert",
                    "{aviso_sobran}"
                }
            } else if capas.total_ml > capacidad * proporciones::LLENADO_MAX {
                p { class: "ajustar-aviso", role: "status",
                    {crate::i18n::ahora(Texto::VasoAlLimite)}
                }
            }

            if capas.total_ml <= 0.0 {
                p { class: "estado", "{crate::i18n::ahora(Texto::TodasLasCapasACero)}" }
            }
        }


        atajos::Atajos {
            ingredientes: ingredientes.clone(),
            factores,
            agregados,
            escala,
            volumen_sin_escala,
            volumen_escala,
            capacidad_ml: capacidad,
        }

        section { class: "seccion", "aria-labelledby": "titulo-vaso",
            div { class: "seccion-cabeza",
                p { id: "titulo-vaso", class: "eyebrow", "{crate::i18n::ahora(Texto::Vaso)}" }
                span { class: "mini", "{vaso_actual.nombre} · {vaso_actual.vaso.capacidad_ml:.0} ml" }
            }

            label { class: "solo-lector", r#for: "vaso-editor", "{crate::i18n::ahora(Texto::ElegirElVaso)}" }
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
            escala,
            capacidad_ml: capacidad,
        }

        guardar::Acciones {
            receta: receta.clone(),
            vaso: vaso_actual.clone(),
            factores,
            agregados,
            escala,
            seleccion_ingrediente,
            indice_vaso,
            indice_inicial,
        }
    }
}

/// Escala cada capa de `ajustada` por su factor sobre la cantidad de `base`: una sola bebida.
fn aplicar_factores(
    ajustada: &mut proporciones::Receta,
    base: &proporciones::Receta,
    factores: &[f64],
    escala: f64,
) {
    for (indice, componente) in ajustada.ingredientes.iter_mut().enumerate() {
        let factor = factores.get(indice).copied().unwrap_or(1.0);
        componente.ml = base.ingredientes[indice].ml * factor * escala;
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

#[cfg(test)]
mod pruebas {
    use super::aplicar_factores;
    use proporciones::{Componente, Receta, Vaso};

    /// Con factores neutros el vaso lleva lo que pide la receta, sin multiplicar por comensales.
    #[test]
    fn el_vaso_muestra_una_sola_bebida() {
        let base = Receta::nueva(
            "Cortado",
            Vaso::nuevo(135.0, 33.0, 24.0, 53.0, 4.0),
            vec![
                Componente::nuevo("Espresso", 40.0, 0.0),
                Componente::nuevo("Leche", 40.0, 0.0),
            ],
        );
        let mut ajustada = base.clone();

        aplicar_factores(&mut ajustada, &base, &[1.0, 1.5], 1.0);

        assert_eq!(ajustada.ingredientes[0].ml, 40.0);
        assert_eq!(ajustada.ingredientes[1].ml, 60.0);
        assert_eq!(proporciones::capas(&ajustada).total_ml, 100.0); // Cabe en el Gibraltar de 135 ml
    }
}
