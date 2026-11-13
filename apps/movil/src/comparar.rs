//! Comparar (B4) e Ingredientes y equivalencias (B5), parte de la app.
//!
//! Comparar pone dos recetas lado a lado con su vaso y la cantidad de cada ingrediente.
//! Equivalencias muestra cada capa de una receta en mililitros, onzas, partes y cucharadas.
//! Las dos pantallas leen la receta con A2, así que funcionan sin conexión si ya se vio.

// LIBRERÍAS

use api::receta::RecetaCompleta; // Receta con vaso y capas
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::ajustes::SenalAjustes; // Unidad elegida en Ajustes
use crate::rutas::Rutas; // Rutas para volver a la ficha y elegir otra receta
use crate::sesion::SenalSesion; // Cuenta y token para leer las recetas

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Texto accesible del vaso: nombre y porcentaje de cada capa.
fn descripcion(receta: &proporciones::Receta) -> String
{
    let partes_vec: Vec<String> = proporciones::capas(receta)
        .capas
        .iter()
        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
        .collect(); // Una parte por capa
    format!("{}: {}", receta.nombre, partes_vec.join(", "))
}

/// Estado de carga, error o vacío común a las dos pantallas; `None` si hay receta.
fn estado_de(recurso: &Option<Result<Option<RecetaCompleta>, ServerFnError>>) -> Option<Element>
{
    match recurso
    {
        None => Some(rsx! { p { class: "estado", role: "status", "Cargando la receta…" } }),
        Some(Err(causa)) if crate::local::es_sin_conexion(causa) => Some(rsx!
        {
            p { class: "estado estado-aviso", role: "alert", "Sin conexión y sin copia guardada de esta receta." }
        }),
        Some(Err(_)) => Some(rsx! { p { class: "estado estado-error", role: "alert", "No pudimos cargar la receta." } }),
        Some(Ok(None)) => Some(rsx! { p { class: "estado", "Esta receta no existe o ya no está disponible." } }),
        Some(Ok(Some(_))) => None, // Hay receta: la pantalla la pinta
    }
}

/// Columna de una receta en la comparación: vaso, total y graduación.
#[component]
fn ColumnaReceta(receta: RecetaCompleta) -> Element
{
    let svg_val = proporciones::svg(&receta.receta, &proporciones::Opciones { escalas: false, mini: false }); // Vaso de la receta
    let capas_res = proporciones::capas(&receta.receta);
    let graduacion_res = proporciones::graduacion(&receta.receta);

    rsx!
    {
        div
        {
            class: "comparar-columna",

            div { class: "vaso", role: "img", "aria-label": descripcion(&receta.receta), dangerous_inner_html: svg_val }
            h2 { "{receta.receta.nombre}" }
            p { class: "barra-meta", "{capas_res.total_ml:.0} ml · {receta.vaso_nombre}" }
            p { class: "barra-meta", if graduacion_res.abv > 0.0 { "{graduacion_res.abv:.1} % ABV" } else { "Sin alcohol" } }
        }
    }
}

/// Pantalla B4: compara la receta `id` con la receta `con`; sin `con`, deja elegirla.
#[component]
pub fn PantallaComparar(id: String, con: String) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let ajustes = use_context::<SenalAjustes>(); // Unidad de las cantidades
    let id_a = id.clone();
    let recurso_a = use_resource(use_reactive!(|(id_a,)|
    {
        crate::local::receta(id_a, crate::sesion::token(&sesion), crate::sesion::cuenta(&sesion))
    })); // Receta de la ficha
    let con_b = con.clone();
    let recurso_b = use_resource(use_reactive!(|(con_b,)|
    {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        async move
        {
            // Sin segunda receta todavía: nada que pedir
            if con_b.is_empty() { Ok(None) } else { crate::local::receta(con_b, token, cuenta).await }
        }
    })); // Receta elegida para comparar
    let catalogo_res = use_resource(move || api::biblioteca::catalogo(crate::sesion::token(&sesion))); // Recetas para elegir
    let unidad = ajustes().unidad;

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::Ficha { id: id.clone() }, class: "enlace", aria_label: "Volver a la ficha", "‹ Receta" }
            }
            p { class: "eyebrow", "Recetario" }
            h1 { "Comparar" }

            if let Some(estado) = estado_de(&recurso_a.read())
            {
                {estado}
            }
            else if con.is_empty()
            {
                // Sin segunda receta: lista para elegirla
                section
                {
                    class: "seccion",
                    "aria-labelledby": "titulo-elegir",
                    p { id: "titulo-elegir", class: "eyebrow", "Elige con qué comparar" }

                    match &*catalogo_res.read()
                    {
                        None => rsx! { p { class: "estado", role: "status", "Cargando recetas…" } },
                        Some(Err(_)) => rsx! { p { class: "estado estado-error", role: "alert", "No pudimos traer las recetas." } },
                        Some(Ok(lista_vec)) => rsx!
                        {
                            ul
                            {
                                class: "barra-hilo",
                                for receta in lista_vec.iter().filter(|receta| receta.id != id)
                                {
                                    li
                                    {
                                        key: "{receta.id}",
                                        Link
                                        {
                                            to: Rutas::Comparar { id: id.clone(), con: receta.id.clone() },
                                            class: "barra-comentario barra-evento",
                                            strong { "{receta.receta.nombre}" }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
            else if let Some(estado) = estado_de(&recurso_b.read())
            {
                {estado}
            }
            else if let (Some(Ok(Some(a))), Some(Ok(Some(b)))) = (&*recurso_a.read(), &*recurso_b.read())
            {
                {
                    let eq_a = proporciones::equivalencias(&a.receta);
                    let eq_b = proporciones::equivalencias(&b.receta);
                    let mut nombres_vec: Vec<String> = Vec::new(); // Ingredientes de las dos, sin repetir

                    // Juntar los ingredientes en el orden en que aparecen
                    for eq in eq_a.iter().chain(eq_b.iter())
                    {
                        if !nombres_vec.contains(&eq.nombre)
                        {
                            nombres_vec.push(eq.nombre.clone());
                        }
                    }

                    rsx!
                    {
                        div
                        {
                            class: "comparar-par",
                            ColumnaReceta { receta: a.clone() }
                            ColumnaReceta { receta: b.clone() }
                        }

                        table
                        {
                            class: "comparar-tabla",
                            caption { class: "solo-lector", "Cantidad de cada ingrediente en las dos recetas" }
                            thead
                            {
                                tr
                                {
                                    th { scope: "col", "Ingrediente" }
                                    th { scope: "col", "{a.receta.nombre}" }
                                    th { scope: "col", "{b.receta.nombre}" }
                                }
                            }
                            tbody
                            {
                                for nombre in nombres_vec
                                {
                                    tr
                                    {
                                        key: "{nombre}",
                                        th { scope: "row", "{nombre}" }
                                        td { {eq_a.iter().find(|eq| eq.nombre == nombre).map(|eq| unidad.cantidad(eq)).unwrap_or_else(|| "—".to_string())} }
                                        td { {eq_b.iter().find(|eq| eq.nombre == nombre).map(|eq| unidad.cantidad(eq)).unwrap_or_else(|| "—".to_string())} }
                                    }
                                }
                            }
                        }

                        Link { to: Rutas::Comparar { id: id.clone(), con: String::new() }, class: "chip", "Comparar con otra" }
                    }
                }
            }
        }
    }
}

/// Pantalla B5: cada capa de la receta en mililitros, onzas, partes y cucharadas.
#[component]
pub fn PantallaEquivalencias(id: String) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let ajustes = use_context::<SenalAjustes>(); // Unidad destacada
    let id_recurso = id.clone();
    let recurso = use_resource(use_reactive!(|(id_recurso,)|
    {
        crate::local::receta(id_recurso, crate::sesion::token(&sesion), crate::sesion::cuenta(&sesion))
    })); // Receta de la ficha
    let unidad = ajustes().unidad;

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::Ficha { id: id.clone() }, class: "enlace", aria_label: "Volver a la ficha", "‹ Receta" }
            }
            p { class: "eyebrow", "Recetario" }
            h1 { "Ingredientes y equivalencias" }

            if let Some(estado) = estado_de(&recurso.read())
            {
                {estado}
            }
            else if let Some(Ok(Some(receta))) = &*recurso.read()
            {
                {
                    let equivalencias_vec = proporciones::equivalencias(&receta.receta);

                    rsx!
                    {
                        h2 { "{receta.receta.nombre}" }
                        p { class: "nota", "Unidad preferida: {unidad.etiqueta()}. Las partes se cuentan contra el ingrediente más pequeño que no es hielo ni agua; 1 cucharada = 15 ml." }

                        table
                        {
                            class: "comparar-tabla",
                            caption { class: "solo-lector", "Cada ingrediente en varias unidades" }
                            thead
                            {
                                tr
                                {
                                    th { scope: "col", "Ingrediente" }
                                    th { scope: "col", "ml" }
                                    th { scope: "col", "oz" }
                                    th { scope: "col", "Partes" }
                                    th { scope: "col", "Cdas" }
                                }
                            }
                            tbody
                            {
                                for eq in equivalencias_vec.iter()
                                {
                                    tr
                                    {
                                        key: "{eq.nombre}",
                                        th { scope: "row", "{eq.nombre}" if eq.dilucion { " (dilución)" } }
                                        td { "{eq.ml:.0}" }
                                        td { "{eq.oz:.1}" }
                                        td { "{eq.partes:.1}" }
                                        td { "{eq.cucharadas:.1}" }
                                    }
                                }
                            }
                        }

                        Link { to: Rutas::Ajustes, class: "chip", "Cambiar la unidad preferida" }
                    }
                }
            }
        }
    }
}
