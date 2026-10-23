//! Pantalla Descubrir (B1) del módulo Recetario (app)
//!
//! Muestra el buscador, los accesos por familia, la sugerencia del día y las últimas recetas vistas

// LIBRERÍAS
use dioxus::prelude::*; // Macros rsx! y component, hooks y elementos HTML (única importación con comodín permitida)
use api::descubrir; // Tipo RecetaSugerida y server function sugerencias_dia
use crate::barra::{BarraInferior, Destino}; // Barra inferior compartida
use crate::proximamente::Proximamente; // Aviso de las funciones que aún no tienen pantalla
use crate::rutas::Rutas; // Rutas de la app, para abrir la ficha de una receta

// CONSTANTES Y VARIABLES GLOBALES
/// Hoja de estilos de la pantalla Descubrir
const CSS_DESCUBRIR: Asset = asset!("/assets/descubrir.css");

// FUNCIONES
/// Resume los porcentajes de las capas de abajo hacia arriba (ej. «27 · 60 · 13 %»)
fn porcentajes_texto(receta: &proporciones::Receta) -> String
{
    // Redondear el porcentaje de cada capa y unirlos con un punto medio
    let porcentajes_vec: Vec<String> = proporciones::capas(receta)
        .capas
        .iter()
        .map(|capa_iter| format!("{:.0}", capa_iter.porcentaje))
        .collect();

    format!("{} %", porcentajes_vec.join(" · ")) // Retornar porcentajes con su unidad
}

/// Describe en texto la receta y sus capas para lectores de pantalla (ej. «Flat White: espresso 27 %, leche 60 %»)
fn descripcion_capas(receta: &proporciones::Receta) -> String
{
    // Nombrar cada capa con su porcentaje redondeado
    let capas_vec: Vec<String> = proporciones::capas(receta)
        .capas
        .iter()
        .map(|capa_iter| format!("{} {:.0} %", capa_iter.nombre.to_lowercase(), capa_iter.porcentaje))
        .collect();

    format!("{}: {}", receta.nombre, capas_vec.join(", ")) // Retornar descripción completa
}

/// Dibuja el logo de Ratio: un vaso con tres capas de bebida
#[component]
fn Marca() -> Element
{
    rsx!
    {
        // Dibujar la silueta del vaso y sus tres capas
        svg
        {
            class: "marca",
            view_box: "0 0 64 64",
            "aria-hidden": "true",

            path { d: "M18 9 H46 L43.4 50.5 Q43 56 37.5 56 H26.5 Q21 56 20.6 50.5 Z", fill: "none", stroke: "currentColor", stroke_width: "2.6", stroke_linejoin: "round" }
            path { d: "M24.3 43.5 H39.7 L38.9 50.2 Q38.7 53.5 35.7 53.5 H28.3 Q25.3 53.5 25.1 50.2 Z", fill: "#B9633B" }
            path { d: "M22.8 31 H41.2 L40.2 43.5 H23.8 Z", fill: "#D8A44C" }
            path { d: "M21.3 20 H42.7 L41.7 31 H22.3 Z", fill: "#6E4A32" }
            path { d: "M27 62 H37", fill: "none", stroke: "currentColor", stroke_width: "2.6", stroke_linecap: "round" }
        }
    }
}

/// Dibuja el vaso de una receta con sus capas reales usando el motor de proporciones
///
/// `receta` es la receta a dibujar. El SVG lo genera `proporciones::svg()`, que escapa los nombres;
/// el contenedor lleva el texto accesible con el porcentaje de cada capa.
#[component]
fn VasoCapas(receta: proporciones::Receta) -> Element
{
    // Generar el SVG del vaso sin etiquetas ni escalas
    let svg_val = proporciones::svg(&receta, &proporciones::Opciones
    {
        escalas: false,
        mini: false,
    });

    rsx!
    {
        // Mostrar el vaso con su descripción accesible
        div
        {
            class: "vaso",
            role: "img",
            "aria-label": descripcion_capas(&receta),
            dangerous_inner_html: svg_val,
        }
    }
}

/// Encabezado con la marca y el avatar: si hay sesión, muestra el alias y deja salir.
#[component]
fn Encabezado() -> Element
{
    let mut sesion = use_context::<crate::sesion::SenalSesion>(); // Sesión iniciada, si la hay
    let mut abierto = use_signal(|| false); // Si el menú del perfil está desplegado
    let navegador = navigator(); // Para volver al acceso al salir

    let perfil = sesion.read().as_ref().map(|guardada| guardada.perfil.alias.clone());

    rsx!
    {
        header
        {
            class: "encabezado",

            div
            {
                class: "encabezado-marca",

                Marca {}
                span { "Ratio" }
            }

            div
            {
                class: "encabezado-perfil",

                if let Some(alias) = perfil.clone()
                {
                    button
                    {
                        class: "avatar avatar-con-sesion",
                        r#type: "button",
                        "aria-label": "Perfil de {alias}",
                        "aria-expanded": if abierto() { "true" } else { "false" },
                        onclick: move |_| abierto.toggle(),

                        {alias.chars().next().unwrap_or('R').to_uppercase().to_string()}
                    }

                    if abierto()
                    {
                        div
                        {
                            class: "perfil-menu",
                            role: "menu",

                            span { class: "perfil-alias", "{alias}" }

                            button
                            {
                                class: "perfil-salir",
                                r#type: "button",
                                onclick: move |_|
                                {
                                    sesion.set(None);
                                    abierto.set(false);
                                    navegador.push(crate::rutas::Rutas::Entrar);
                                },
                                "Salir"
                            }
                        }
                    }
                }
                else
                {
                    // Sin sesión, el perfil sigue pendiente
                    Proximamente
                    {
                        button
                        {
                            class: "avatar",
                            r#type: "button",
                            "aria-label": "Perfil",
                            "aria-disabled": "true",

                            svg
                            {
                                view_box: "0 0 24 24",
                                "aria-hidden": "true",

                                circle { cx: "12", cy: "9", r: "4", fill: "none", stroke: "currentColor", stroke_width: "1.8" }
                                path { d: "M4.5 20 Q12 13.5 19.5 20", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Buscador() -> Element
{
    rsx!
    {
        // Formulario de búsqueda que no recarga la vista
        form
        {
            class: "buscador",
            role: "search",
            onsubmit: move |evento_val| evento_val.prevent_default(),

            label { class: "solo-lector", r#for: "buscar", "Buscar recetas" }
            input
            {
                id: "buscar",
                r#type: "search",
                placeholder: "Buscar por bebida, ingrediente o método",
                autocomplete: "off",
            }

            // Botón de búsqueda (abre Biblioteca cuando exista)
            Proximamente
            {
                button
                {
                    class: "buscador-boton",
                    r#type: "submit",
                    "aria-label": "Buscar",
                    "aria-disabled": "true",

                    svg
                    {
                        view_box: "0 0 24 24",
                        "aria-hidden": "true",

                        circle { cx: "11", cy: "11", r: "6.5", fill: "none", stroke: "currentColor", stroke_width: "2" }
                        path { d: "M16 16 L20.5 20.5", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round" }
                    }
                }
            }
        }
    }
}

/// Muestra los accesos por familia de bebida, que abrirán Biblioteca ya filtrada
#[component]
fn ChipsFamilia() -> Element
{
    rsx!
    {
        // Lista de familias en forma de píldora
        nav
        {
            class: "chips",
            "aria-label": "Familias de bebida",

            Proximamente
            {
                button { class: "chip", r#type: "button", "aria-disabled": "true", "Espresso" }
            }
            Proximamente
            {
                button { class: "chip", r#type: "button", "aria-disabled": "true", "Con leche" }
            }

            // shortcut: «Cócteles» se omite para todos porque sin JWT la persona cuenta como menor de edad, mostrarlo cuando la sesión declare mayoría de edad
        }
    }
}

/// Muestra la tarjeta destacada de una sugerencia: vaso, nombre, porcentajes y acceso a la receta
///
/// `sugerencia` es la receta del día que llegó del servidor.
#[component]
fn TarjetaSugerencia(sugerencia: descubrir::RecetaSugerida) -> Element
{
    rsx!
    {
        // Tarjeta con el vaso a la izquierda y los datos a la derecha
        article
        {
            class: "tarjeta tarjeta-sugerencia",

            VasoCapas { receta: sugerencia.receta.clone() }

            div
            {
                class: "tarjeta-datos",

                h3 { "{sugerencia.receta.nombre}" }
                p { class: "porcentajes", {porcentajes_texto(&sugerencia.receta)} }

                // Acceso a la ficha de la receta
                Link { to: Rutas::Ficha { id: sugerencia.id.clone() }, class: "primario", "Ver receta" }
            }
        }
    }
}

/// Muestra la sugerencia del día en un carrusel de café y coctelería con sus estados de carga
#[component]
fn SeccionSugerencias() -> Element
{
    let sesion = use_context::<crate::sesion::SenalSesion>(); // Sesión con la que se pide el catálogo
    let mut sugerencias_res = use_resource(move || {
        descubrir::sugerencias_dia(crate::sesion::token(&sesion))
    }); // Sugerencias del día pedidas al servidor
    let mut indice_sel = use_signal(|| 0_usize); // Sugerencia visible en el carrusel

    rsx!
    {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-sugerencia",

            // Elegir contenido según el estado de la petición
            match &*sugerencias_res.read()
            {
                None => rsx!
                {
                    // Petición en curso: mostrar aviso de carga
                    p { id: "titulo-sugerencia", class: "eyebrow", "Sugerencia del día" }
                    p { class: "estado", role: "status", "Buscando la sugerencia de hoy…" }
                },
                Some(Ok(sugerencias_vec)) if sugerencias_vec.is_empty() => rsx!
                {
                    // Sin recetas publicadas: invitar a volver después
                    p { id: "titulo-sugerencia", class: "eyebrow", "Sugerencia del día" }
                    p { class: "estado", "Todavía no hay recetas publicadas. Vuelve pronto." }
                },
                Some(Ok(sugerencias_vec)) =>
                {
                    // Recetas recibidas: mostrar la elegida en el carrusel
                    let total_val = sugerencias_vec.len(); // Cantidad de sugerencias del día
                    let indice_val = *indice_sel.read() % total_val; // Posición visible dentro del rango

                    rsx!
                    {
                        // Encabezado con el contador del carrusel
                        div
                        {
                            class: "seccion-cabeza",

                            p { id: "titulo-sugerencia", class: "eyebrow", "Sugerencia del día" }
                            div
                            {
                                class: "contador",

                                button
                                {
                                    r#type: "button",
                                    "aria-label": "Sugerencia anterior",
                                    disabled: total_val < 2,
                                    onclick: move |_| indice_sel.set((indice_val + total_val - 1) % total_val),
                                    "‹"
                                }
                                span { "aria-live": "polite", "{indice_val + 1} de {total_val}" }
                                button
                                {
                                    r#type: "button",
                                    "aria-label": "Sugerencia siguiente",
                                    disabled: total_val < 2,
                                    onclick: move |_| indice_sel.set((indice_val + 1) % total_val),
                                    "›"
                                }
                            }
                        }

                        TarjetaSugerencia { sugerencia: sugerencias_vec[indice_val].clone() }
                    }
                }
                Some(Err(ServerFnError::Request(error_val))) if !matches!(error_val, dioxus::fullstack::RequestError::Status(..)) => rsx!
                {
                    // Sin conexión (el servidor no respondió): avisar y permitir reintentar
                    p { id: "titulo-sugerencia", class: "eyebrow", "Sugerencia del día" }
                    div
                    {
                        class: "estado estado-aviso",
                        role: "alert",

                        p { "Sin conexión. Revisa tu red y vuelve a intentarlo." }
                        button { class: "enlace", r#type: "button", onclick: move |_| sugerencias_res.restart(), "Reintentar" }
                    }
                },
                Some(Err(_)) => rsx!
                {
                    // Error del servidor (respondió con fallo): avisar y permitir reintentar
                    p { id: "titulo-sugerencia", class: "eyebrow", "Sugerencia del día" }
                    div
                    {
                        class: "estado estado-error",
                        role: "alert",

                        p { "No pudimos traer la sugerencia de hoy." }
                        button { class: "enlace", r#type: "button", onclick: move |_| sugerencias_res.restart(), "Reintentar" }
                    }
                },
            }
        }
    }
}

/// Muestra las últimas recetas vistas, que salen del historial local del dispositivo
#[component]
fn SeccionUltimasVistas() -> Element
{
    rsx!
    {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-ultimas",

            // Encabezado con el acceso a Biblioteca
            div
            {
                class: "seccion-cabeza",

                p { id: "titulo-ultimas", class: "eyebrow", "Sigue donde lo dejaste" }
                Link { to: Rutas::Comparar, class: "enlace", "Comparar" }
                Link { to: Rutas::Biblioteca, class: "enlace", "Biblioteca" }
            }

            // shortcut: siempre se muestra el estado vacío porque el repositorio local (A2) no existe, leer de A2 y agregar los estados de carga, error y sin conexión cuando exista
            p { class: "estado", "Aún no has preparado nada. Empieza por la sugerencia de hoy." }
        }
    }
}

/// Muestra la pantalla Descubrir completa: encabezado, buscador, familias, sugerencia y últimas vistas
#[component]
pub fn PantallaDescubrir() -> Element
{
    rsx!
    {
        document::Stylesheet { href: CSS_DESCUBRIR }

        // Contenido desplazable de la pantalla
        main
        {
            class: "descubrir",

            Encabezado {}
            h1 { "¿Qué vas a preparar hoy?" }
            Buscador {}
            ChipsFamilia {}
            SeccionSugerencias {}
            SeccionUltimasVistas {}
        }

        BarraInferior { activo: Destino::Descubrir }
    }
}
