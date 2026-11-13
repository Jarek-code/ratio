//! Perfil propio (E2) y perfil público (E3), parte de la app.
//!
//! Con alias vacío la pantalla muestra el perfil de la sesión, con accesos a Mi recetario,
//! Guardados, Actividad, Ajustes y Salir. Con un alias muestra el perfil público de esa
//! cuenta, sus recetas publicadas y el botón para seguirla.

// LIBRERÍAS

use api::biblioteca::RecetaResumen; // Receta publicada del perfil
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::local::Cambio; // Cambio que se encola sin conexión
use crate::local::SenalPendientes; // Contador de cambios pendientes
use crate::rutas::Rutas; // Rutas de los accesos del perfil
use crate::sesion::SenalSesion; // Sesión con alias y token

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Tarjeta de una receta publicada con su vaso en miniatura.
#[component]
fn TarjetaReceta(receta: RecetaResumen) -> Element
{
    let svg_val = proporciones::svg(&receta.receta, &proporciones::Opciones { escalas: false, mini: true }); // Vaso en miniatura
    let capas_res = proporciones::capas(&receta.receta); // Total servido
    let descripcion = proporciones::capas(&receta.receta)
        .capas
        .iter()
        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
        .collect::<Vec<_>>()
        .join(", "); // Texto accesible del vaso

    rsx!
    {
        li
        {
            Link
            {
                to: Rutas::Ficha { id: receta.id.clone() },
                class: "barra-comentario barra-evento perfil-receta",

                div
                {
                    class: "perfil-mini",
                    role: "img",
                    "aria-label": "{receta.receta.nombre}: {descripcion}",
                    dangerous_inner_html: svg_val,
                }
                div
                {
                    p { strong { "{receta.receta.nombre}" } }
                    p { class: "barra-meta", "{crate::biblioteca::familia_legible(&receta.familia)} · {capas_res.total_ml:.0} ml" }
                }
            }
        }
    }
}

/// Accesos del perfil propio: recetario, guardados, actividad, ajustes y salir.
#[component]
fn AccesosPropios() -> Element
{
    let mut sesion = use_context::<SenalSesion>(); // Para cerrar la sesión
    let navegador = navigator(); // Para volver al acceso al salir

    rsx!
    {
        nav
        {
            class: "perfil-accesos",
            "aria-label": "Tu cuenta",

            Link { to: Rutas::MisRecetas, class: "chip", "Mi recetario" }
            Link { to: Rutas::Guardados, class: "chip", "Guardados" }
            Link { to: Rutas::Actividad, class: "chip", "Actividad" }
            Link { to: Rutas::Ajustes, class: "chip", "Ajustes" }
            button
            {
                class: "chip",
                r#type: "button",
                onclick: move |_|
                {
                    crate::sesion::salir(&mut sesion);
                    navegador.push(Rutas::Entrar);
                },
                "Salir"
            }
        }
    }
}

/// Botón para seguir o dejar de seguir un perfil ajeno (E3).
///
/// `alias` es el perfil y `lo_sigo` el estado con el que se abrió la pantalla.
#[component]
fn BotonSeguir(alias: String, lo_sigo: bool) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para seguir
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut sigue_sel = use_signal(|| lo_sigo); // Estado visible
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    rsx!
    {
        button
        {
            class: if sigue_sel() { "chip chip--activo" } else { "primario" },
            r#type: "button",
            "aria-pressed": if sigue_sel() { "true" } else { "false" },
            onclick: move |_|
            {
                let Some(token) = crate::sesion::token(&sesion) else
                {
                    return; // Sin sesión el botón no se muestra
                };
                let sigue = !sigue_sel();
                let objetivo = alias.clone();
                sigue_sel.set(sigue); // Mostrar el cambio de inmediato
                aviso.set(None);

                spawn(async move
                {
                    match api::perfil::seguir(token, objetivo.clone(), sigue).await
                    {
                        Ok(_) => {}
                        Err(causa) if crate::local::es_sin_conexion(&causa)
                            && crate::local::encolar(&sesion, &mut pendientes, Cambio::Seguimiento { alias: objetivo, sigue }) =>
                        {
                            aviso.set(Some("Sin conexión: se aplicará cuando vuelva la red.".to_string()));
                        }
                        Err(_) =>
                        {
                            sigue_sel.set(!sigue); // Deshacer el cambio visible
                            aviso.set(Some("No se pudo actualizar.".to_string()));
                        }
                    }
                });
            },
            if sigue_sel() { "Siguiendo" } else { "Seguir" }
        }

        if let Some(texto) = aviso()
        {
            p { class: "barra-aviso", role: "status", "{texto}" }
        }
    }
}

/// Perfil propio (alias vacío) o público (con alias), con sus recetas publicadas (E2 y E3).
#[component]
pub fn PantallaPerfil(alias: String) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Alias propio y token
    let mut recurso = use_resource(use_reactive!(|(alias,)|
    {
        let propio = sesion.read().as_ref().map(|guardada| guardada.perfil.alias.clone());
        let token = crate::sesion::token(&sesion);
        async move
        {
            // Alias vacío: el perfil de la sesión, o nada si no hay sesión
            let buscado = if alias.is_empty() { propio } else { Some(alias) };
            match buscado
            {
                Some(buscado) => api::perfil::perfil(buscado, token).await,
                None => Ok(None),
            }
        }
    })); // Perfil del servidor
    let con_sesion = sesion.read().is_some();

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

            // Elegir contenido según el estado de la petición
            match &*recurso.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "Cargando el perfil…" }
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
                        p { "No pudimos traer el perfil." }
                        button { class: "enlace", r#type: "button", onclick: move |_| recurso.restart(), "Reintentar" }
                    }
                },
                Some(Ok(None)) if !con_sesion => rsx!
                {
                    div
                    {
                        class: "estado",
                        p { "Entra para ver tu perfil, tus recetas y a quién sigues." }
                        Link { to: Rutas::Entrar, class: "enlace", "Entrar" }
                    }
                },
                Some(Ok(None)) => rsx!
                {
                    p { class: "estado", "Este perfil no existe." }
                },
                Some(Ok(Some(perfil))) => rsx!
                {
                    p { class: "eyebrow", if perfil.propio { "Tu perfil" } else { "La Barra" } }
                    h1 { "{perfil.alias}" }
                    p
                    {
                        class: "barra-meta",
                        "{perfil.recetas.len()} recetas publicadas · {perfil.seguidores} seguidores · {perfil.siguiendo} siguiendo"
                    }

                    // Perfil propio: accesos de la cuenta; ajeno con sesión: seguir
                    if perfil.propio
                    {
                        AccesosPropios {}
                    }
                    else if con_sesion
                    {
                        BotonSeguir { alias: perfil.alias.clone(), lo_sigo: perfil.lo_sigo }
                    }

                    section
                    {
                        class: "seccion",
                        "aria-labelledby": "titulo-perfil-recetas",

                        p { id: "titulo-perfil-recetas", class: "eyebrow", "Recetas publicadas" }
                        if perfil.recetas.is_empty()
                        {
                            p { class: "estado", "Todavía no hay recetas publicadas." }
                        }
                        ul
                        {
                            class: "barra-hilo",
                            for receta in perfil.recetas.iter()
                            {
                                TarjetaReceta { key: "{receta.id}", receta: receta.clone() }
                            }
                        }
                    }
                },
            }
        }

        BarraInferior { activo: Destino::Perfil }
    }
}
