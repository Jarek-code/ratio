//! Pantalla Actividad (D6) de La Barra, parte de la app.
//!
//! Lista lo que otras personas hicieron con las recetas y los comentarios de la cuenta:
//! comentarios, respuestas, valoraciones y versiones derivadas. Cada novedad abre su receta.

// LIBRERÍAS

use api::foro::Evento; // Novedad que pinta la lista
use api::foro::TipoEvento; // Clase de novedad
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::rutas::Rutas; // Rutas para abrir la receta y volver

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Frase que resume la novedad, por ejemplo «comentó tu receta».
fn frase(tipo: TipoEvento) -> &'static str
{
    match tipo
    {
        TipoEvento::Comentario => "comentó tu receta",
        TipoEvento::Respuesta => "respondió tu comentario en",
        TipoEvento::Valoracion => "preparó tu receta",
        TipoEvento::Derivada => "publicó una versión:",
    }
}

/// Tarjeta de una novedad que abre su receta.
#[component]
fn TarjetaEvento(evento: Evento) -> Element
{
    rsx!
    {
        li
        {
            Link
            {
                to: Rutas::Ficha { id: evento.receta_id.clone() },
                class: "barra-comentario barra-evento",

                p
                {
                    class: "barra-meta",
                    strong { "{evento.actor}" }
                    " {frase(evento.tipo)} "
                    strong { "{evento.receta_nombre}" }
                }
                p { "{evento.detalle}" }
                p { class: "barra-meta", "{evento.fecha}" }
            }
        }
    }
}

/// Pantalla D6: las últimas novedades sobre las recetas y los comentarios de la cuenta.
#[component]
pub fn PantallaActividad() -> Element
{
    let sesion = use_context::<crate::sesion::SenalSesion>(); // Token para pedir la actividad
    let mut recurso = use_resource(move ||
    {
        let token = crate::sesion::token(&sesion);
        async move
        {
            match token
            {
                Some(token) => api::foro::actividad(token).await.map(Some), // Con sesión: pedir la actividad
                None => Ok(None), // Sin sesión: invitar a entrar
            }
        }
    }); // Actividad del servidor

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::Descubrir, class: "enlace", aria_label: "Volver a Descubrir", "‹ Descubrir" }
            }

            p { class: "eyebrow", "La Barra" }
            h1 { "Actividad" }

            // Elegir contenido según el estado de la petición
            match &*recurso.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "Cargando tu actividad…" }
                },
                Some(Err(causa)) if crate::local::es_sin_conexion(causa) => rsx!
                {
                    div
                    {
                        class: "estado estado-aviso",
                        role: "alert",
                        p { "Sin conexión. Revisa tu red y vuelve a intentarlo." }
                        button { class: "enlace", r#type: "button", onclick: move |_| recurso.restart(), "Reintentar" }
                    }
                },
                Some(Err(_)) => rsx!
                {
                    div
                    {
                        class: "estado estado-error",
                        role: "alert",
                        p { "No pudimos traer tu actividad." }
                        button { class: "enlace", r#type: "button", onclick: move |_| recurso.restart(), "Reintentar" }
                    }
                },
                Some(Ok(None)) => rsx!
                {
                    div
                    {
                        class: "estado",
                        p { "Entra para ver quién comenta y prepara tus recetas." }
                        Link { to: Rutas::Entrar, class: "enlace", "Entrar" }
                    }
                },
                Some(Ok(Some(eventos_vec))) if eventos_vec.is_empty() => rsx!
                {
                    p { class: "estado", "Todavía no hay novedades. Publica una receta en La Barra para empezar." }
                },
                Some(Ok(Some(eventos_vec))) => rsx!
                {
                    ul
                    {
                        class: "barra-hilo",
                        for (indice, evento) in eventos_vec.iter().enumerate()
                        {
                            TarjetaEvento { key: "{indice}", evento: evento.clone() }
                        }
                    }
                },
            }
        }

        BarraInferior { activo: Destino::LaBarra }
    }
}
