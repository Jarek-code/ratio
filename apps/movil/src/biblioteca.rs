//! Biblioteca (B2): el catálogo completo con búsqueda, filtros y orden.

use api::biblioteca::{RecetaResumen, catalogo};
use dioxus::prelude::*;

use crate::barra::{BarraInferior, Destino};
use crate::rutas::Rutas;

/// Hoja de estilos propia de la biblioteca.
const CSS_BIBLIOTECA: Asset = asset!("/assets/biblioteca.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Cómo se ordena la lista.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Orden {
    /// De la A a la Z.
    Nombre,
    /// De la más intensa a la más suave.
    Intensidad,
    /// De la más fácil a la más difícil.
    Dificultad,
    /// De la más rápida a la más lenta.
    Tiempo,
}

impl Orden {
    /// Los cuatro órdenes, en el orden en que se ofrecen.
    const TODOS: [Orden; 4] = [Orden::Nombre, Orden::Intensidad, Orden::Dificultad, Orden::Tiempo];

    /// Texto del botón de orden.
    fn etiqueta(self) -> &'static str {
        match self {
            Orden::Nombre => "Nombre",
            Orden::Intensidad => "Intensidad",
            Orden::Dificultad => "Dificultad",
            Orden::Tiempo => "Tiempo",
        }
    }

    /// Valor por el que se ordena la lista, con los huecos al final.
    fn clave(self, receta: &RecetaResumen) -> i32 {
        match self {
            Orden::Nombre => 0,
            Orden::Intensidad => receta.intensidad.map_or(i32::MAX, i32::from).saturating_neg(),
            Orden::Dificultad => receta.dificultad.map_or(i32::MAX, i32::from),
            Orden::Tiempo => receta.minutos.map_or(i32::MAX, i32::from),
        }
    }
}

/// Catálogo completo de recetas publicadas, con buscador, chips de familia y orden.
#[component]
pub fn PantallaBiblioteca() -> Element {
    let mut catalogo_res = use_resource(catalogo);
    let busqueda = use_signal(String::new);
    let familia = use_signal(|| None::<String>);
    let orden = use_signal(|| Orden::Nombre);

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_BIBLIOTECA }

        main { class: "biblioteca",
            header { class: "biblioteca-cabeza",
                p { class: "eyebrow", "Recetario" }
                h1 { "Biblioteca" }
                p { class: "biblioteca-sub", "Todas las recetas publicadas, con su proporción a la vista." }
                Link {
                    to: Rutas::ListaCompra,
                    class: "chip biblioteca-compra",
                    "Lista de compra"
                }
            }

            Buscador { busqueda }

            match &*catalogo_res.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando el catálogo…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos traer el catálogo: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| catalogo_res.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(lista)) => rsx! {
                    ChipsFamilia { lista: lista.clone(), familia }
                    Ordenes { orden }

                    {
                        let filtradas = filtrar(lista, &busqueda.read(), &familia.read(), *orden.read());

                        rsx! {
                            if filtradas.is_empty() {
                                p { class: "estado", "No hay recetas que coincidan. Prueba con otra palabra o quita el filtro." }
                            } else {
                                p { class: "biblioteca-cuenta", role: "status", "{filtradas.len()} recetas" }
                                ul { class: "lista-recetas",
                                    for receta in filtradas {
                                        RecetaEnLista { key: "{receta.id}", receta }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        }

        BarraInferior { activo: Destino::Biblioteca }
    }
}

/// Buscador por nombre de bebida.
#[component]
fn Buscador(busqueda: Signal<String>) -> Element {
    let mut busqueda = busqueda;

    rsx! {
        form {
            class: "buscador",
            role: "search",
            onsubmit: move |evento| evento.prevent_default(),

            label { class: "solo-lector", r#for: "buscar-biblioteca", "Buscar recetas" }
            input {
                id: "buscar-biblioteca",
                r#type: "search",
                placeholder: "Buscar por nombre",
                autocomplete: "off",
                value: "{busqueda}",
                oninput: move |evento| busqueda.set(evento.value()),
            }
        }
    }
}

/// Filtros por familia, sacados de las recetas que hay.
#[component]
fn ChipsFamilia(lista: Vec<RecetaResumen>, familia: Signal<Option<String>>) -> Element {
    let mut familia = familia;
    let mut familias: Vec<String> = lista.iter().map(|r| r.familia.clone()).collect();
    familias.sort();
    familias.dedup();
    let activa = familia.read().clone();

    rsx! {
        div { class: "chips", role: "group", "aria-label": "Filtrar por familia",
            button {
                class: if activa.is_none() { "chip activo" } else { "chip" },
                r#type: "button",
                onclick: move |_| familia.set(None),
                "Todas"
            }
            for nombre in familias {
                button {
                    key: "{nombre}",
                    class: if activa.as_deref() == Some(nombre.as_str()) { "chip activo" } else { "chip" },
                    r#type: "button",
                    onclick: {
                        let nombre = nombre.clone();
                        move |_| familia.set(Some(nombre.clone()))
                    },
                    "{familia_legible(&nombre)}"
                }
            }
        }
    }
}

/// Orden de la lista.
#[component]
fn Ordenes(orden: Signal<Orden>) -> Element {
    let mut orden = orden;
    let activo = *orden.read();

    rsx! {
        div { class: "chips", role: "group", "aria-label": "Ordenar la lista",
            span { class: "chips-titulo", "Ordenar" }
            for valor in Orden::TODOS {
                button {
                    key: "{valor.etiqueta()}",
                    class: if valor == activo { "chip activo" } else { "chip" },
                    r#type: "button",
                    onclick: move |_| orden.set(valor),
                    "{valor.etiqueta()}"
                }
            }
        }
    }
}

/// Una receta de la lista: miniatura del vaso, datos y barra de proporción.
#[component]
fn RecetaEnLista(receta: RecetaResumen) -> Element {
    let capas = proporciones::capas(&receta.receta);
    let graduacion = proporciones::graduacion(&receta.receta);
    let svg = proporciones::svg(
        &receta.receta,
        &proporciones::Opciones { escalas: false, mini: true },
    );
    let mut datos: Vec<String> = Vec::new();

    datos.push(format!("{:.0} ml", capas.total_ml));

    if graduacion.abv > 0.0 {
        datos.push(format!("{:.1} % ABV", graduacion.abv));
    }

    if let Some(minutos) = receta.minutos {
        datos.push(format!("{minutos} min"));
    }

    if let Some(dificultad) = receta.dificultad {
        datos.push(format!("Dificultad {dificultad}"));
    }

    rsx! {
        li {
            Link { to: Rutas::Ficha { id: receta.id.clone() }, class: "rec-tarjeta",
                div { class: "rec-mini", dangerous_inner_html: svg }

                div { class: "rec-cuerpo",
                    div { class: "rec-top",
                        h3 { "{receta.receta.nombre}" }
                        span { class: "rec-familia", "{familia_legible(&receta.familia)}" }
                    }

                    p { class: "rec-datos", "{datos.join(\" · \")}" }

                    div { class: "rec-barra", "aria-hidden": "true",
                        for capa in capas.capas.iter() {
                            span {
                                class: if capa.dilucion { "rec-segmento rec-segmento--dilucion" } else { "rec-segmento" },
                                style: "flex-grow:{capa.ml};background:{capa.color.clone().unwrap_or_else(|| \"#B9A48C\".to_string())}",
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Nombre visible de una familia del catálogo (`con_leche` → «Con leche»).
pub(crate) fn familia_legible(clave: &str) -> String {
    let texto = match clave {
        "cafe_coctel" => "Café coctel".to_string(),
        "con_leche" => "Con leche".to_string(),
        "sin_alcohol" => "Sin alcohol".to_string(),
        otra => otra.replace('_', " "),
    };

    let mut letras = texto.chars();

    match letras.next() {
        Some(primera) => primera.to_uppercase().collect::<String>() + letras.as_str(),
        None => texto,
    }
}

/// Aplica búsqueda, familia y orden a la lista del catálogo.
fn filtrar(
    lista: &[RecetaResumen],
    busqueda: &str,
    familia: &Option<String>,
    orden: Orden,
) -> Vec<RecetaResumen> {
    let texto = busqueda.trim().to_lowercase();
    let mut salida: Vec<RecetaResumen> = lista
        .iter()
        .filter(|receta| familia.as_deref().is_none_or(|f| receta.familia == f))
        .filter(|receta| {
            texto.is_empty() || receta.receta.nombre.to_lowercase().contains(&texto)
        })
        .cloned()
        .collect();

    if orden != Orden::Nombre {
        salida.sort_by_key(|receta| orden.clave(receta));
    }

    salida
}
