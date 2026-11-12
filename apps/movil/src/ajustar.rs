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

use api::guardar::{ComponenteAgregado, ComponenteNuevo, RecetaNueva, guardar_receta};
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

/// Atajo de proporción que se aplica a todas las capas a la vez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preset {
    /// Vuelve a las cantidades publicadas.
    Original,
    /// Sube los ingredientes que aportan fuerza.
    Fuerte,
    /// Baja los ingredientes que aportan fuerza.
    Debil,
}

impl Preset {
    /// Presets visibles, en el orden en que se ofrecen.
    const TODOS: [Preset; 3] = [Preset::Original, Preset::Fuerte, Preset::Debil];

    /// Texto corto del botón.
    fn etiqueta(self) -> &'static str {
        match self {
            Preset::Original => "Original",
            Preset::Fuerte => "Más fuerte",
            Preset::Debil => "Más débil",
        }
    }
}

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

    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut factores = use_signal(|| vec![1.0_f64; total_ingredientes]);
    let mut agregados = use_signal(Vec::<CapaAgregada>::new);
    let mut seleccion_ingrediente = use_signal(String::new);
    let mut comensales = use_signal(|| 1_u32);
    let mut indice_vaso = use_signal(move || indice_inicial);
    let mut pendientes = use_context::<crate::local::SenalPendientes>();
    let mut guardando = use_signal(|| false);
    let mut guardada = use_signal(|| None::<String>);
    let mut aviso = use_signal(|| None::<String>);

    let Some(vaso_actual) = opciones.get(*indice_vaso.read()).cloned() else {
        return rsx! {
            p { class: "estado", "No hay vasos disponibles para ajustar la receta." }
        };
    };

    let receta_para_guardar = receta.clone();
    let vaso_para_guardar = vaso_actual.clone();

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
    let catalogo_para_agregar = ingredientes_catalogo.clone();

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

        section { class: "seccion", "aria-labelledby": "titulo-atajos",
            div { class: "seccion-cabeza",
                p { id: "titulo-atajos", class: "eyebrow", "Atajos de proporción" }
                span { class: "mini", "Se aplican a todas las capas" }
            }

            div { class: "ajustar-presets",
                for preset in Preset::TODOS {
                    {
                        let ingredientes = ingredientes.clone();
                        rsx! {
                            button {
                                key: "{preset.etiqueta()}",
                                class: "chip",
                                r#type: "button",
                                onclick: move |_| {
                                    let actuales = factores.read().clone();
                                    factores.set(factores_preset(preset, &ingredientes, &actuales));

                                    // Los ingredientes agregados siguen la misma regla que
                                    // las capas del origen para que el atajo sea coherente.
                                    let mut extras = agregados.read().clone();
                                    for extra in &mut extras {
                                        extra.factor = factor_preset(
                                            preset,
                                            &extra.nombre,
                                            extra.graduacion,
                                            extra.factor,
                                        );
                                    }
                                    agregados.set(extras);
                                },
                                "{preset.etiqueta()}"
                            }
                        }
                    }
                }
            }

            div { class: "ajustar-escala",
                label {
                    class: "ajustar-etiqueta",
                    r#for: "escala-ml",
                    "Escala"
                }
                input {
                    id: "escala-ml",
                    class: "ajustar-rango ajustar-rango--escala",
                    r#type: "range",
                    min: "0",
                    max: "{VOLUMEN_MAXIMO_ML:.0}",
                    step: "5",
                    value: "{volumen_escala:.0}",
                    disabled: total_base <= 0.0,
                    aria_label: "Volumen por comensal en mililitros",
                    "aria-valuetext": "{volumen_escala:.0} mililitros por comensal",
                    oninput: move |evento| {
                        let objetivo = evento
                            .value()
                            .parse::<f64>()
                            .unwrap_or(0.0)
                            .clamp(0.0, VOLUMEN_MAXIMO_ML);

                        // Un solo factor para todas las capas conserva la proporción de
                        // la receta mientras el volumen por comensal se mueve en mililitros.
                        if total_base > 0.0 {
                            let factor = (objetivo / total_base).clamp(0.0, FACTOR_MAXIMO);
                            let mut nuevos = factores.read().clone();
                            nuevos.fill(factor);
                            factores.set(nuevos);

                            let mut extras = agregados.read().clone();
                            for extra in &mut extras {
                                extra.factor = factor;
                            }
                            agregados.set(extras);
                        }
                    },
                }
                span { class: "ajustar-escala-valor", "{volumen_escala:.0} ml" }
            }

            p { class: "mini", "La escala termina en 500 ml por comensal y mueve todas las capas." }
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
                    if let Ok(indice) = evento.value().parse::<usize>() {
                        if indice < opciones.len() {
                            indice_vaso.set(indice);
                        }
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

        div { class: "ajustar-acciones",
            button {
                class: "chip",
                r#type: "button",
                onclick: move |_| {
                    factores.set(vec![1.0; total_ingredientes]);
                    agregados.set(Vec::new());
                    seleccion_ingrediente.set(String::new());
                    comensales.set(1);
                    indice_vaso.set(indice_inicial);
                },
                "Restablecer"
            }

            button {
                class: "chip",
                r#type: "button",
                disabled: *guardando.read(),
                onclick: move |_| {
                    if *guardando.read() {
                        return;
                    }

                    let Some(token) = crate::sesion::token(&sesion) else {
                        aviso.set(Some("Inicia sesión para guardar una versión.".to_string()));
                        return;
                    };

                    // La version guarda las proporciones por comensal; los comensales
                    // siguen siendo un ajuste de la vista y no una propiedad de la receta.
                    let factores_guardar = factores.read().clone();
                    let mut componentes = Vec::new();

                    // C1 muestra todas las capas del motor, también las mg (1 g = 1 ml),
                    // así que se envían todas con su factor y conservan su unidad original.
                    for (indice_capa, capa) in
                        receta_para_guardar.componentes.iter().enumerate()
                    {
                        let factor = factores_guardar
                            .get(indice_capa)
                            .copied()
                            .unwrap_or(1.0);

                        // Un factor cero quita la capa; no se guarda una cantidad invalida.
                        if factor <= 0.0 {
                            continue;
                        }

                        componentes.push(ComponenteNuevo {
                            ingrediente_id: capa.ingrediente_id.clone(),
                            posicion: capa.posicion,
                            cantidad: capa.cantidad * factor,
                        });
                    }

                    // Las capas nuevas viajan aparte: el servidor valida su metadata
                    // contra el catálogo y las coloca encima de las del origen.
                    let mut agregados_guardar = Vec::new();
                    for extra in agregados.read().iter() {
                        if extra.factor <= 0.0 {
                            continue;
                        }

                        agregados_guardar.push(ComponenteAgregado {
                            ingrediente_id: extra.ingrediente_id.clone(),
                            cantidad: extra.cantidad * extra.factor,
                        });
                    }

                    let nombre_version = format!(
                        "{} · versión",
                        receta_para_guardar.receta.nombre
                    )
                    .chars()
                    .take(120)
                    .collect::<String>();

                    let datos = RecetaNueva {
                        origen_id: receta_para_guardar.id.clone(),
                        nombre: nombre_version,
                        vaso_id: vaso_para_guardar.id.clone(),
                        capacidad_ml: vaso_para_guardar.vaso.capacidad_ml,
                        componentes,
                        agregados: agregados_guardar,
                        id: crate::local::nuevo_id(),
                    };

                    guardando.set(true);
                    guardada.set(None);
                    aviso.set(None);

                    spawn(async move {
                        match guardar_receta(token, datos.clone()).await {
                            Ok(resultado) => {
                                let mensaje = if resultado.advertencias.is_empty() {
                                    "Versión guardada. Ya aparece en Mi recetario.".to_string()
                                } else {
                                    format!(
                                        "Versión guardada. {}",
                                        resultado.advertencias.join(" ")
                                    )
                                };
                                aviso.set(Some(mensaje));
                                guardada.set(Some(resultado.receta_id));
                            }
                            // Sin red: la versión queda en la cola con su mismo id
                            Err(causa)
                                if crate::local::es_sin_conexion(&causa)
                                    && crate::local::encolar(
                                        &sesion,
                                        &mut pendientes,
                                        crate::local::Cambio::Version(datos),
                                    ) =>
                            {
                                aviso.set(Some(
                                    "Sin conexión: la versión se enviará cuando vuelva la red y aparecerá en Mi recetario."
                                        .to_string(),
                                ));
                            }
                            Err(causa) => {
                                aviso.set(Some(format!("No pudimos guardar la versión: {causa}")));
                            }
                        }

                        guardando.set(false);
                    });
                },
                if *guardando.read() {
                    "Guardando…"
                } else {
                    "Guardar versión privada"
                }
            }

            if let Some(id) = guardada.read().as_ref() {
                Link {
                    to: Rutas::Ficha { id: id.clone() },
                    class: "mini",
                    "Abrir versión"
                }
            }
        }

        if let Some(mensaje) = aviso.read().as_ref() {
            p {
                class: "ajustar-aviso",
                role: "status",
                "{mensaje}"
            }
        }
    }
}

/// Aplica un preset sobre los factores actuales y los deja dentro del rango del control.
fn factores_preset(
    preset: Preset,
    ingredientes: &[proporciones::Componente],
    actuales: &[f64],
) -> Vec<f64> {
    let mut factores = if actuales.len() == ingredientes.len() {
        actuales.to_vec()
    } else {
        vec![1.0; ingredientes.len()]
    };

    match preset {
        Preset::Original => factores.fill(1.0),
        Preset::Fuerte => {
            for (factor, ingrediente) in factores.iter_mut().zip(ingredientes) {
                if aporta_fuerza(&ingrediente.nombre, ingrediente.graduacion) {
                    *factor = (*factor * 1.25).clamp(0.0, FACTOR_MAXIMO);
                }
            }
        }
        Preset::Debil => {
            for (factor, ingrediente) in factores.iter_mut().zip(ingredientes) {
                if aporta_fuerza(&ingrediente.nombre, ingrediente.graduacion) {
                    *factor = (*factor * 0.75).clamp(0.0, FACTOR_MAXIMO);
                }
            }
        }
    }

    factores
}

/// Calcula el factor de una capa agregada para el preset elegido.
fn factor_preset(preset: Preset, nombre: &str, graduacion: f64, actual: f64) -> f64 {
    match preset {
        Preset::Original => 1.0,
        Preset::Fuerte if aporta_fuerza(nombre, graduacion) => {
            (actual * 1.25).clamp(0.0, FACTOR_MAXIMO)
        }
        Preset::Debil if aporta_fuerza(nombre, graduacion) => {
            (actual * 0.75).clamp(0.0, FACTOR_MAXIMO)
        }
        Preset::Fuerte | Preset::Debil => actual,
    }
}

/// Indica si la capa aporta fuerza: alcohol o la parte concentrada del café.
fn aporta_fuerza(nombre: &str, graduacion: f64) -> bool {
    graduacion > 0.0 || contiene(nombre, &["espresso", "café", "cafe", "cold brew"])
}

/// Compara sin distinguir mayúsculas y acepta cualquiera de las palabras.
fn contiene(nombre: &str, palabras: &[&str]) -> bool {
    let nombre = nombre.to_lowercase();
    palabras.iter().any(|palabra| nombre.contains(palabra))
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
