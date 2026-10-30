//! Pantalla C1: ajustar las proporciones de una receta y ver el vaso en vivo.
//!
//! El editor trabaja sobre factores de cada ingrediente, no sobre mililitros absolutos.
//! Así el escalado por comensales y los presets pueden cambiar el total sin perder el
//! ajuste individual que hizo la persona. El vaso y la graduación se recalculan con el
//! mismo motor que usa la ficha (B3).

use api::guardar::{ComponenteNuevo, RecetaNueva, guardar_receta};
use api::receta::{RecetaCompleta, completa};
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

/// Atajo de proporción que se aplica a todas las capas a la vez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preset {
    /// Vuelve a las cantidades publicadas.
    Original,
    /// Sube los ingredientes que aportan fuerza.
    Fuerte,
    /// Sube la leche, la crema o la nata.
    Lacteo,
    /// Deja en cero las capas de espuma.
    SinEspuma,
}

impl Preset {
    /// Presets visibles, en el orden en que se ofrecen.
    const TODOS: [Preset; 4] = [
        Preset::Original,
        Preset::Fuerte,
        Preset::Lacteo,
        Preset::SinEspuma,
    ];

    /// Texto corto del botón.
    fn etiqueta(self) -> &'static str {
        match self {
            Preset::Original => "Original",
            Preset::Fuerte => "Más fuerte",
            Preset::Lacteo => "Más lácteo",
            Preset::SinEspuma => "Sin espuma",
        }
    }
}

/// Pantalla C1: carga la receta y el catálogo de vasos antes de abrir el editor.
#[component]
pub fn PantallaAjustar(id: String) -> Element {
    let id_recurso = id.clone();
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut recurso = use_resource(move || {
        let id = id_recurso.clone();
        let token = crate::sesion::token(&sesion);
        async move {
            let receta = completa(id, token).await?;
            let catalogo = catalogo_vasos().await?;
            Ok::<_, ServerFnError>((receta, catalogo))
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
                Some(Ok((None, _))) => rsx! {
                    p { class: "estado", "Esta receta no existe o ya no está publicada." }
                },
                Some(Ok((Some(receta), catalogo))) => rsx! {
                    Editor { receta: receta.clone(), vasos: catalogo.clone() }
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

/// Editor de proporciones: vista del vaso, totales, vaso, escalado y capas.
#[component]
fn Editor(receta: RecetaCompleta, vasos: Vec<VasoOpcion>) -> Element {
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
    let mut comensales = use_signal(|| 1_u32);
    let mut indice_vaso = use_signal(move || indice_inicial);
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
    let comensales_actuales = *comensales.read();
    let mut ajustada = receta.receta.clone();
    ajustada.vaso = vaso_actual.vaso.clone();
    ajustada.capacidad_ml = vaso_actual.vaso.capacidad_ml;

    for (indice, componente) in ajustada.ingredientes.iter_mut().enumerate() {
        let factor = factores_actuales.get(indice).copied().unwrap_or(1.0);
        componente.ml =
            receta.receta.ingredientes[indice].ml * factor * f64::from(comensales_actuales);
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
    let escala = escala_activa(&factores_actuales);
    let descripcion = descripcion(&ajustada);

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
                                },
                                "{preset.etiqueta()}"
                            }
                        }
                    }
                }
            }

            div { class: "ajustar-porcentajes", role: "group", "aria-label": "Escalar todas las capas",
                span { class: "ajustar-etiqueta", "Escala" }
                for valor in [0.5_f64, 1.0, 1.5, 2.0] {
                    button {
                        key: "escala-{valor}",
                        class: if escala == Some(valor) { "chip activo" } else { "chip" },
                        r#type: "button",
                        onclick: move |_| {
                            let mut nuevos = factores.read().clone();
                            nuevos.fill(valor);
                            factores.set(nuevos);
                        },
                        "{valor * 100.0:.0} %"
                    }
                }
            }
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

            div { class: "ajustar-vasos", role: "group", "aria-label": "Elegir el vaso",
                for (indice, opcion) in opciones.iter().enumerate() {
                    button {
                        key: "{opcion.id}",
                        class: if indice == *indice_vaso.read() { "chip ajustar-vaso-chip activo" } else { "chip ajustar-vaso-chip" },
                        r#type: "button",
                        "aria-pressed": if indice == *indice_vaso.read() { "true" } else { "false" },
                        onclick: move |_| indice_vaso.set(indice),
                        span { "{opcion.nombre}" }
                        span { class: "ajustar-vaso-capacidad", "{opcion.vaso.capacidad_ml:.0} ml" }
                    }
                }
            }
        }

        section { class: "seccion", "aria-labelledby": "titulo-capas",
            div { class: "seccion-cabeza",
                p { id: "titulo-capas", class: "eyebrow", "Capas" }
                span { class: "mini", "{total_ingredientes} ingredientes" }
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

                        rsx! {
                            li { class: "ajustar-capa",
                                div { class: "ajustar-capa-cabeza",
                                    span {
                                        class: "ajustar-capa-punto",
                                        style: "background:{color}",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "ajustar-capa-nombre", "{ingrediente.nombre}" }
                                    span { class: "ajustar-capa-cifra", "{ml:.0} ml · {porcentaje:.0} %" }
                                }
                                input {
                                    class: "ajustar-rango",
                                    r#type: "range",
                                    min: "0",
                                    max: "{FACTOR_MAXIMO * 100.0:.0}",
                                    step: "5",
                                    value: "{factor * 100.0:.0}",
                                    aria_label: etiqueta,
                                    "aria-valuetext": "{ml:.0} mililitros, {factor * 100.0:.0} por ciento",
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
                                    span { "{porcentaje:.0} % del total" }
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
                    let mut indice_liquido = 0;

                    for capa in &receta_para_guardar.componentes {
                        if capa.unidad != "ml" {
                            continue;
                        }

                        let factor = factores_guardar
                            .get(indice_liquido)
                            .copied()
                            .unwrap_or(1.0);
                        indice_liquido += 1;

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
                    };

                    guardando.set(true);
                    guardada.set(None);
                    aviso.set(None);

                    spawn(async move {
                        match guardar_receta(token, datos).await {
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
                if aporta_fuerza(ingrediente) {
                    *factor = (*factor * 1.25).clamp(0.0, FACTOR_MAXIMO);
                }
            }
        }
        Preset::Lacteo => {
            for (factor, ingrediente) in factores.iter_mut().zip(ingredientes) {
                if es_lacteo(&ingrediente.nombre) {
                    *factor = (*factor * 1.3).clamp(0.0, FACTOR_MAXIMO);
                }
            }
        }
        Preset::SinEspuma => {
            for (factor, ingrediente) in factores.iter_mut().zip(ingredientes) {
                if es_espuma(&ingrediente.nombre) {
                    *factor = 0.0;
                }
            }
        }
    }

    factores
}

/// Indica si la capa aporta fuerza: alcohol o la parte concentrada del café.
fn aporta_fuerza(componente: &proporciones::Componente) -> bool {
    componente.graduacion > 0.0
        || contiene(
            &componente.nombre,
            &["espresso", "café", "cafe", "cold brew"],
        )
}

/// Indica si la capa es leche, crema, nata o yogur.
fn es_lacteo(nombre: &str) -> bool {
    contiene(
        nombre,
        &["leche", "crema", "nata", "yogur", "lácteo", "lacteo"],
    )
}

/// Indica si la capa es espuma.
fn es_espuma(nombre: &str) -> bool {
    contiene(nombre, &["espuma", "foam"])
}

/// Compara sin distinguir mayúsculas y acepta cualquiera de las palabras.
fn contiene(nombre: &str, palabras: &[&str]) -> bool {
    let nombre = nombre.to_lowercase();
    palabras.iter().any(|palabra| nombre.contains(palabra))
}

/// Devuelve la escala común de todas las capas, si comparten el mismo factor.
fn escala_activa(factores: &[f64]) -> Option<f64> {
    let primero = *factores.first()?;

    if factores
        .iter()
        .all(|factor| (factor - primero).abs() < 0.005)
    {
        Some(primero)
    } else {
        None
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
