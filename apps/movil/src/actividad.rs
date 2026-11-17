//! Actividad (D6) de La Barra, parte de la app.
//!
//! Lista lo que otras personas hicieron con las recetas y los comentarios de la cuenta
//! (comentarios, respuestas, valoraciones y versiones derivadas) y las decisiones de
//! moderación que le tocan. Al abrirla, las novedades quedan marcadas como vistas y el punto
//! de la barra inferior se apaga.

// LIBRERÍAS

use std::time::Duration; // Espera entre dos consultas de novedades

use api::notificaciones::Evento; // Novedad que pinta la lista
use api::notificaciones::TipoEvento; // Clase de novedad
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Rutas para abrir la receta y volver
use crate::sesion::SenalSesion; // Sesión con cuenta y token

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Segundos entre dos consultas de novedades sin leer.
const REVISAR_CADA: u64 = 60;

// ESTRUCTURAS

/// Novedades sin leer de la cuenta actual, compartidas por contexto.
#[derive(Clone, Copy)]
pub struct NoLeidas(pub Signal<u32>);

// FUNCIONES

/// Consulta cada minuto cuántas novedades sin leer tiene la cuenta, mientras la app siga abierta.
///
/// Sin sesión el contador queda en cero. Un fallo de red no cambia el contador: se vuelve a
/// preguntar en la siguiente vuelta.
pub async fn vigilar(sesion: SenalSesion, no_leidas: NoLeidas)
{
    let mut contador = no_leidas.0;

    loop
    {
        match crate::sesion::token(&sesion)
        {
            Some(token) =>
            {
                if let Ok(total) = crate::red::con_plazo(api::notificaciones::no_leidas(token)).await
                {
                    contador.set(total);
                }
            }
            None => contador.set(0), // Sin sesión no hay novedades propias
        }

        tokio::time::sleep(Duration::from_secs(REVISAR_CADA)).await;
    }
}

/// Frase que une a quien hizo la novedad con la receta, por ejemplo «comentó tu receta».
fn frase(tipo: TipoEvento) -> &'static str
{
    match tipo
    {
        TipoEvento::Comentario => "comentó tu receta",
        TipoEvento::Respuesta => "respondió tu comentario en",
        TipoEvento::Valoracion => "preparó tu receta",
        TipoEvento::Derivada => "publicó una versión:",
        TipoEvento::Moderacion => "revisó un reporte sobre",
    }
}

/// Tarjeta de una novedad: quién, qué y sobre qué receta; abre el hilo o la ficha.
#[component]
fn TarjetaEvento(evento: Evento) -> Element
{
    // Una novedad sobre un comentario abre el hilo; las demás, la ficha
    let destino = evento.receta_id.clone().map(|id| match evento.tipo
    {
        TipoEvento::Comentario | TipoEvento::Respuesta => Rutas::Hilo { id },
        TipoEvento::Valoracion | TipoEvento::Derivada | TipoEvento::Moderacion => Rutas::Ficha { id },
    });

    rsx!
    {
        li
        {
            class: if evento.nueva { "barra-comentario barra-comentario--nueva" } else { "barra-comentario" },

            p
            {
                class: "barra-meta",
                if evento.nueva
                {
                    span { class: "muro-insignia muro-insignia--version", "Nueva" }
                    " "
                }
                // Moderación no tiene perfil; las demás novedades llevan al de quien las hizo
                if evento.tipo == TipoEvento::Moderacion
                {
                    strong { "{evento.actor}" }
                }
                else
                {
                    Link
                    {
                        to: Rutas::PerfilPublico { alias: evento.actor.clone() },
                        class: "barra-autor",
                        "aria-label": "Ver el perfil de {evento.actor}",
                        "{evento.actor}"
                    }
                }
                " {frase(evento.tipo)} "
                strong { "{evento.receta_nombre}" }
            }

            // Con receta visible la novedad es un enlace; sin ella, solo texto
            if let Some(destino) = destino
            {
                Link
                {
                    to: destino,
                    class: "barra-evento",
                    "aria-label": "Abrir {evento.receta_nombre}",

                    p { "{evento.detalle}" }
                    p { class: "barra-meta", "{evento.fecha}" }
                }
            }
            else
            {
                p { "{evento.detalle}" }
                p { class: "barra-meta", "{evento.fecha}" }
            }
        }
    }
}

/// Lista de novedades de la cuenta; al cargarla las marca como vistas.
///
/// La usan la pantalla Actividad y la sección Actividad de La Barra.
#[component]
pub fn ContenidoActividad() -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para pedir la actividad
    let mut no_leidas = use_context::<NoLeidas>().0; // Punto de la barra inferior
    let mut marcada = use_signal(|| false); // Si ya se avisó al servidor de que se vio
    let mut recurso = use_resource(move ||
    {
        let token = crate::sesion::token(&sesion);
        async move
        {
            match token
            {
                Some(token) => crate::red::con_reintentos(move || api::notificaciones::actividad(token.clone())).await.map(Some), // Con sesión: pedir la actividad
                None => Ok(None), // Sin sesión: invitar a entrar
            }
        }
    }); // Actividad del servidor

    // Con la lista en pantalla, marcar las novedades como vistas una sola vez
    use_effect(move ||
    {
        let cargada = matches!(&*recurso.read(), Some(Ok(Some(_))));
        if cargada && !*marcada.peek()
            && let Some(token) = crate::sesion::token(&sesion)
        {
            marcada.set(true);
            spawn(async move
            {
                if crate::red::con_plazo(api::notificaciones::marcar_leidas(token)).await.is_ok()
                {
                    no_leidas.set(0);
                }
            });
        }
    });

    rsx!
    {
        // Elegir contenido según el estado de la petición
        match &*recurso.read()
        {
            None => rsx!
            {
                p { class: "estado", role: "status", "Cargando tu actividad…" }
            },
            Some(Err(causa)) => rsx!
            {
                // Fallo de red o del servidor: explicar qué pasó y ofrecer reintentar
                AvisoRed
                {
                    causa: causa.clone(),
                    reintentar: move |_| recurso.restart(),
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
}

/// Pantalla D6: las últimas novedades sobre las recetas y los comentarios de la cuenta.
#[component]
pub fn PantallaActividad() -> Element
{
    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::la_barra(), class: "enlace", aria_label: "Volver a La Barra", "‹ La Barra" }
            }

            p { class: "eyebrow", "La Barra" }
            h1 { "Actividad" }

            ContenidoActividad {}
        }

        BarraInferior { activo: Destino::LaBarra }
    }
}
