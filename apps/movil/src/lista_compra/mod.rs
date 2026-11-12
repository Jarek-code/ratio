//! Pantalla C7: convierte las recetas elegidas en una lista para el mercado.
//!
//! La pantalla no vende ni cobra nada: suma los ingredientes de las recetas
//! seleccionadas, ajusta por comensales y los agrupa por tipo para que la compra
//! se pueda seguir con el dedo. Las casillas y la seleccion se guardan en el
//! almacenamiento local del dispositivo porque la lista es una ayuda de
//! preparacion, no un pedido.

use api::biblioteca::{RecetaResumen, catalogo};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::barra::{BarraInferior, Destino};
use crate::biblioteca::familia_legible;
use crate::rutas::Rutas;

mod calculo;

use calculo::{agregar, agrupar, clave_compra, filtrar_recetas};

/// Hoja de estilos propia de la lista de compra.
const CSS_LISTA_COMPRA: Asset = asset!("/assets/lista_compra.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Maximo de comensales que permite el ajuste rapido.
const COMENSALES_MAXIMOS: u32 = 12;

/// Archivo de la carpeta privada de la app donde se conserva la lista entre visitas.
const ARCHIVO_LISTA: &str = "lista_compra.json";

/// Estado minimo que se guarda; la lista se reconstruye desde el catalogo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct EstadoLista {
    /// IDs de las recetas elegidas.
    seleccion: Vec<String>,
    /// Claves normalizadas de los ingredientes marcados.
    marcados: Vec<String>,
    /// Numero de comensales con el que se estaba mirando la lista.
    comensales: u32,
    /// Unidad preferida para mostrar las cantidades.
    unidad: Unidad,
}

impl Default for EstadoLista {
    fn default() -> Self {
        Self {
            seleccion: Vec::new(),
            marcados: Vec::new(),
            comensales: 1,
            unidad: Unidad::Mililitros,
        }
    }
}

/// Unidad en la que se muestran las cantidades de la lista.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Unidad {
    /// Mililitros; se abrevia a litros cuando la lista crece.
    Mililitros,
    /// Onzas liquidas.
    Onzas,
    /// Cucharadas de cocina.
    Cucharadas,
}

impl Unidad {
    /// Unidades disponibles en el selector.
    const TODAS: [Unidad; 3] = [Unidad::Mililitros, Unidad::Onzas, Unidad::Cucharadas];

    /// Texto corto del selector.
    fn etiqueta(self) -> &'static str {
        match self {
            Unidad::Mililitros => "ml",
            Unidad::Onzas => "oz",
            Unidad::Cucharadas => "cdas",
        }
    }

    /// Texto accesible del selector.
    fn nombre(self) -> &'static str {
        match self {
            Unidad::Mililitros => "mililitros",
            Unidad::Onzas => "onzas",
            Unidad::Cucharadas => "cucharadas",
        }
    }

    /// Convierte mililitros a la unidad elegida.
    fn convertir(self, ml: f64) -> f64 {
        match self {
            Unidad::Mililitros => ml,
            Unidad::Onzas => ml / proporciones::OZ_ML,
            Unidad::Cucharadas => ml / proporciones::CUCHARA_ML,
        }
    }

    /// Cantidad de una linea de la lista.
    fn cantidad(self, ml: f64) -> String {
        match self {
            Unidad::Mililitros if ml >= 1000.0 => format!("{:.1} L", ml / 1000.0),
            Unidad::Mililitros if ml.fract().abs() < 0.05 => format!("{ml:.0} ml"),
            Unidad::Mililitros => format!("{ml:.1} ml"),
            Unidad::Onzas => format!("{:.1} oz", self.convertir(ml)),
            Unidad::Cucharadas => format!("{:.0} cdas", self.convertir(ml)),
        }
    }

    /// Total de la lista, con litros cuando corresponde.
    fn total(self, ml: f64) -> String {
        if self == Unidad::Mililitros && ml >= 1000.0 {
            format!("{:.1} L", ml / 1000.0)
        } else {
            self.cantidad(ml)
        }
    }
}


/// Pantalla C7: carga el catalogo, deja elegir recetas y arma la lista.
#[component]
pub fn PantallaListaCompra() -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut recurso = use_resource(move || catalogo(crate::sesion::token(&sesion)));

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_LISTA_COMPRA }

        main { class: "lista-compra",
            BarraSuperior {}

            header { class: "lista-compra-cabeza",
                p { class: "eyebrow", "Preparación" }
                h1 { "Lista de compra" }
                p { class: "lista-compra-sub", "Todo lo que necesitas para tus recetas, sumado en una sola lista." }
            }

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando el recetario…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos traer las recetas: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| recurso.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(lista)) => rsx! {
                    Contenido { lista: lista.clone() }
                },
            }
        }

        BarraInferior { activo: Destino::Biblioteca }
    }
}

/// Barra superior con la vuelta a la biblioteca.
#[component]
fn BarraSuperior() -> Element {
    rsx! {
        header { class: "lista-compra-barra",
            Link {
                to: Rutas::biblioteca(),
                class: "lista-compra-volver",
                aria_label: "Volver a la biblioteca",
                "‹ Biblioteca"
            }
            span { class: "lista-compra-marca", "Ratio" }
        }
    }
}

/// Cuerpo de C7: resumen, seleccion de recetas y lista agrupada.
#[component]
fn Contenido(lista: Vec<RecetaResumen>) -> Element {
    // El estado se lee del archivo local al montar; el catalogo ya esta disponible,
    // asi que se descartan IDs de recetas que hayan dejado de existir.
    let estado_inicial = use_hook(|| {
        let mut estado = crate::local::leer_valor::<EstadoLista>(ARCHIVO_LISTA).unwrap_or_default();
        estado
            .seleccion
            .retain(|id| lista.iter().any(|receta| &receta.id == id));
        estado.comensales = estado.comensales.clamp(1, COMENSALES_MAXIMOS);
        estado
    });
    let mut seleccion = use_signal(|| estado_inicial.seleccion.clone());
    let mut marcados = use_signal(|| estado_inicial.marcados.clone());
    let mut busqueda = use_signal(String::new);
    let mut comensales = use_signal(|| estado_inicial.comensales);
    let mut unidad = use_signal(|| estado_inicial.unidad);

    // Cada cambio se guarda en el archivo local, sin JavaScript
    use_effect(move || {
        let estado = EstadoLista {
            seleccion: seleccion.read().clone(),
            marcados: marcados.read().clone(),
            comensales: *comensales.read(),
            unidad: *unidad.read(),
        };

        crate::local::escribir(ARCHIVO_LISTA, &estado);
    });

    let ids = seleccion.read().clone();
    let comidas = *comensales.read();
    let unidad_actual = *unidad.read();
    let compras = agregar(&lista, &ids, comidas);
    let total_ml: f64 = compras.iter().map(|compra| compra.ml).sum();
    let marcados_actuales = marcados.read().clone();
    let terminados = compras
        .iter()
        .filter(|compra| {
            marcados_actuales
                .iter()
                .any(|clave| clave == &clave_compra(&compra.nombre))
        })
        .count();
    let seleccionadas: Vec<RecetaResumen> = ids
        .iter()
        .filter_map(|id| lista.iter().find(|receta| &receta.id == id).cloned())
        .collect();
    let filtradas = filtrar_recetas(&lista, &busqueda.read(), &ids);

    rsx! {
        section { class: "compra-resumen", "aria-labelledby": "compra-resumen-titulo",
            div { class: "compra-resumen-cabeza",
                div {
                    p { id: "compra-resumen-titulo", class: "eyebrow", "Resumen" }
                    p { class: "compra-resumen-meta", "{seleccionadas.len()} recetas · {compras.len()} ingredientes" }
                }
                if !marcados_actuales.is_empty() {
                    button {
                        class: "enlace",
                        r#type: "button",
                        onclick: move |_| marcados.write().clear(),
                        "Desmarcar todo"
                    }
                }
            }

            dl { class: "compra-metricas",
                div { class: "compra-metrica",
                    dt { "Recetas" }
                    dd { "{seleccionadas.len()}" }
                }
                div { class: "compra-metrica",
                    dt { "Ingredientes" }
                    dd { "{compras.len()}" }
                }
                div { class: "compra-metrica",
                    dt { "Total" }
                    dd { "{unidad_actual.total(total_ml)}" }
                }
            }

            div { class: "compra-controles",
                div { class: "compra-comensales",
                    span { class: "compra-control-etiqueta", "Comensales" }
                    div { class: "compra-stepper",
                        button {
                            r#type: "button",
                            aria_label: "Quitar un comensal",
                            disabled: comidas <= 1,
                            onclick: move |_| comensales.set(comidas.saturating_sub(1).max(1)),
                            "−"
                        }
                        strong { "{comidas}" }
                        button {
                            r#type: "button",
                            aria_label: "Agregar un comensal",
                            disabled: comidas >= COMENSALES_MAXIMOS,
                            onclick: move |_| comensales.set((comidas + 1).min(COMENSALES_MAXIMOS)),
                            "+"
                        }
                    }
                }

                div { class: "compra-unidades", role: "group", "aria-label": "Unidad de la lista",
                    for opcion in Unidad::TODAS {
                        button {
                            key: "{opcion.etiqueta()}",
                            r#type: "button",
                            class: if opcion == unidad_actual { "compra-unidad activa" } else { "compra-unidad" },
                            aria_label: "{opcion.nombre()}",
                            onclick: move |_| unidad.set(opcion),
                            "{opcion.etiqueta()}"
                        }
                    }
                }
            }
        }

        section { class: "compra-recetas", "aria-labelledby": "compra-recetas-titulo",
            div { class: "compra-seccion-cabeza",
                h2 { id: "compra-recetas-titulo", "Recetas en la lista" }
                if !seleccionadas.is_empty() {
                    button {
                        class: "enlace",
                        r#type: "button",
                        onclick: move |_| {
                            seleccion.write().clear();
                            marcados.write().clear();
                        },
                        "Vaciar"
                    }
                }
            }

            if seleccionadas.is_empty() {
                p { class: "compra-nota", "Elige una o más recetas para empezar." }
            } else {
                div { class: "compra-chips",
                    for receta in seleccionadas.iter().cloned() {
                        button {
                            key: "{receta.id}",
                            class: "compra-chip",
                            r#type: "button",
                            aria_label: "Quitar {receta.receta.nombre}",
                            onclick: {
                                let id = receta.id.clone();
                                move |_| {
                                    seleccion.write().retain(|actual| actual != &id);
                                }
                            },
                            span { "{receta.receta.nombre}" }
                            span { class: "compra-chip-x", "aria-hidden": "true", "×" }
                        }
                    }
                }
            }

            details { class: "compra-picker",
                summary { "Agregar receta" }

                form {
                    class: "compra-buscador",
                    role: "search",
                    onsubmit: move |evento| evento.prevent_default(),
                    label { class: "solo-lector", r#for: "buscar-compra", "Buscar recetas para la lista" }
                    input {
                        id: "buscar-compra",
                        r#type: "search",
                        placeholder: "Buscar por nombre",
                        autocomplete: "off",
                        value: "{busqueda}",
                        oninput: move |evento| busqueda.set(evento.value()),
                    }
                }

                if filtradas.is_empty() {
                    p { class: "compra-nota", "No hay recetas que coincidan." }
                } else {
                    ul { class: "compra-picker-lista",
                        for receta in filtradas {
                            li {
                                button {
                                    key: "{receta.id}",
                                    class: if ids.iter().any(|id| id == &receta.id) { "compra-picker-opcion activa" } else { "compra-picker-opcion" },
                                    r#type: "button",
                                    aria_pressed: if ids.iter().any(|id| id == &receta.id) { "true" } else { "false" },
                                    onclick: {
                                        let id = receta.id.clone();
                                        move |_| {
                                            let mut actual = seleccion.write();
                                            if let Some(posicion) = actual.iter().position(|valor| valor == &id) {
                                                actual.remove(posicion);
                                            } else {
                                                actual.push(id.clone());
                                            }
                                        }
                                    },
                                    span { class: "compra-picker-nombre", "{receta.receta.nombre}" }
                                    span { class: "compra-picker-familia", "{familia_legible(&receta.familia)}" }
                                    span { class: "compra-picker-check", "aria-hidden": "true",
                                        if ids.iter().any(|id| id == &receta.id) { "✓" } else { "+" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        section { class: "compra-lista", "aria-labelledby": "compra-lista-titulo",
            div { class: "compra-seccion-cabeza",
                h2 { id: "compra-lista-titulo", "Tu lista" }
                if !compras.is_empty() {
                    span { class: "compra-cuenta", "{terminados} de {compras.len()} listos" }
                }
            }

            if seleccionadas.is_empty() {
                div { class: "estado compra-vacio",
                    p { "La lista aparecerá aquí en cuanto elijas una receta." }
                }
            } else if compras.is_empty() {
                p { class: "compra-nota", "Estas recetas no tienen ingredientes que comprar." }
            } else {
                for grupo in agrupar(&compras) {
                    section { class: "compra-grupo",
                        h3 { "{grupo.0.etiqueta()}" }
                        ul { class: "compra-items",
                            for compra in grupo.1 {
                                li {
                                    button {
                                        key: "{clave_compra(&compra.nombre)}",
                                        class: if marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)) { "compra-item marcado" } else { "compra-item" },
                                        r#type: "button",
                                        role: "checkbox",
                                        aria_checked: if marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)) { "true" } else { "false" },
                                        onclick: {
                                            let clave = clave_compra(&compra.nombre);
                                            move |_| {
                                                let mut actual = marcados.write();
                                                if let Some(posicion) = actual.iter().position(|valor| valor == &clave) {
                                                    actual.remove(posicion);
                                                } else {
                                                    actual.push(clave.clone());
                                                }
                                            }
                                        },
                                        span { class: "compra-check", "aria-hidden": "true",
                                            if marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)) { "✓" }
                                        }
                                        span { class: "compra-nombre", "{compra.nombre}" }
                                        span { class: "compra-cantidad", "{unidad_actual.cantidad(compra.ml)}" }
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

