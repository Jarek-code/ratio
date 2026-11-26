//! Perfil propio (E2) y perfil público (E3), parte de la app.
//!
//! Con alias vacío la pantalla muestra el perfil de la sesión, con accesos a Mi recetario,
//! Guardados, Actividad, Ajustes y Salir, y el diario de preparaciones. Con un alias muestra
//! el perfil público de esa cuenta, sus recetas publicadas y el botón para seguirla.

// LIBRERÍAS

use api::diario::EntradaDiario; // Preparación del diario
use api::perfil::PerfilPublico; // Perfil con sus recetas y contadores
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::i18n::Texto;
use crate::la_barra::TarjetaMuro; // Tarjeta de receta compartida con el muro
use crate::local::Cambio; // Cambio que se encola sin conexión
use crate::local::SenalPendientes; // Contador de cambios pendientes
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Rutas de los accesos del perfil
use crate::sesion::SenalSesion; // Sesión con alias y token // Textos traducibles

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Accesos del perfil propio: recetario, guardados, actividad, ajustes, moderación y salir.
#[component]
fn AccesosPropios() -> Element {
    let mut sesion = use_context::<SenalSesion>(); // Para cerrar la sesión
    let navegador = navigator(); // Para volver al acceso al salir
    let modera = sesion
        .read()
        .as_ref()
        .is_some_and(|guardada| guardada.perfil.rol == api::Rol::Moderador); // El servidor lo vuelve a comprobar

    rsx! {
        nav
        {
            class: "perfil-accesos",
            "aria-label": "{crate::i18n::ahora(Texto::TuCuenta)}",

            Link { to: Rutas::MisRecetas, class: "chip", "{crate::i18n::ahora(Texto::MiRecetario)}" }
            Link { to: Rutas::Guardados, class: "chip", "{crate::i18n::ahora(Texto::Guardados)}" }
            Link { to: Rutas::Actividad, class: "chip", "{crate::i18n::ahora(Texto::Actividad)}" }
            Link { to: Rutas::Ajustes, class: "chip", "{crate::i18n::ahora(Texto::Ajustes)}" }
            if modera
            {
                Link { to: Rutas::Moderacion, class: "chip", "{crate::i18n::ahora(Texto::ModeracionSeccion)}" }
            }
            button
            {
                class: "chip",
                r#type: "button",
                onclick: move |_|
                {
                    crate::sesion::salir(&mut sesion);
                    navegador.push(Rutas::Entrar);
                },
                "{crate::i18n::ahora(Texto::Salir)}"
            }
        }
    }
}

/// Botón para seguir o dejar de seguir un perfil ajeno (E3).
///
/// `alias` es el perfil y `sigue` el estado visible, que también mueve el contador de seguidores.
#[component]
fn BotonSeguir(alias: String, sigue: Signal<bool>) -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para seguir
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut sigue = sigue;
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    rsx! {
        button
        {
            class: if sigue() { "chip chip--activo" } else { "primario" },
            r#type: "button",
            "aria-pressed": if sigue() { "true" } else { "false" },
            onclick: move |_|
            {
                let Some(token) = crate::sesion::token(&sesion) else
                {
                    return; // Sin sesión el botón no se muestra
                };
                let nuevo = !sigue();
                let objetivo = alias.clone();
                sigue.set(nuevo); // Mostrar el cambio de inmediato
                aviso.set(None);

                spawn(async move
                {
                    match crate::red::con_plazo(api::perfil::seguir(token, objetivo.clone(), nuevo)).await
                    {
                        Ok(_) => {}
                        Err(causa) if crate::red::sin_conexion(&causa)
                            && crate::local::encolar(&sesion, &mut pendientes, Cambio::Seguimiento { alias: objetivo, sigue: nuevo }) =>
                        {
                            aviso.set(Some(crate::i18n::ahora(Texto::AplicarSinConexion).to_string()));
                        }
                        Err(causa) =>
                        {
                            sigue.set(!nuevo); // Deshacer el cambio visible
                            aviso.set(Some(crate::i18n::con(crate::i18n::actual(), Texto::NoSePudoActualizar, &[&crate::red::mensaje(&causa)])));
                        }
                    }
                });
            },
            if sigue() { {crate::i18n::ahora(Texto::Siguiendo)} } else { {crate::i18n::ahora(Texto::Seguir)} }
        }

        if let Some(texto) = aviso()
        {
            p { class: "barra-aviso", role: "status", "{texto}" }
        }
    }
}

/// Una preparación del diario: receta, resultado, gusto y duración.
#[component]
fn TarjetaDiario(entrada: EntradaDiario) -> Element {
    let mut datos_vec = vec![
        if entrada.me_salio {
            crate::i18n::ahora(Texto::MeSalio)
        } else {
            crate::i18n::ahora(Texto::NoMeSalio)
        }
        .to_string(),
    ]; // Resultado y detalles
    if let Some(gusto) = entrada.gusto {
        datos_vec.push(crate::i18n::con(
            crate::i18n::actual(),
            Texto::GustoDeCinco,
            &[&gusto.to_string()],
        ));
    }
    if let Some(segundos) = entrada.duracion_s {
        datos_vec.push(api::diario::duracion_legible(segundos));
    }

    rsx! {
        li
        {
            class: "barra-comentario",

            // Con receta visible el nombre es un enlace; sin ella, solo texto
            if let Some(id) = entrada.receta_id.clone()
            {
                Link { to: Rutas::Ficha { id }, class: "barra-autor", "{entrada.receta_nombre}" }
            }
            else
            {
                p { strong { "{entrada.receta_nombre}" } }
            }
            p { "{datos_vec.join(\" · \")}" }
            p { class: "barra-meta", "{crate::i18n::fecha_iso(&entrada.fecha)}" }
        }
    }
}

/// Sección «Mi diario» del perfil propio: las preparaciones de la cuenta (D4).
#[component]
fn MiDiario() -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para pedir el diario
    let mut recurso = use_resource(move || {
        let token = crate::sesion::token(&sesion);
        async move {
            match token {
                Some(token) => {
                    crate::red::con_reintentos(move || api::diario::diario(token.clone())).await
                }
                None => Ok(Vec::new()), // Sin sesión no hay diario
            }
        }
    }); // Diario del servidor

    rsx! {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-diario",

            p { id: "titulo-diario", class: "eyebrow", "{crate::i18n::ahora(Texto::MiDiario)}" }

            // Elegir contenido según el estado de la petición
            match &*recurso.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoDiario)}" }
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
                Some(Ok(entradas_vec)) if entradas_vec.is_empty() => rsx!
                {
                    p { class: "estado", "{crate::i18n::ahora(Texto::DiarioVacioAjeno)}" }
                },
                Some(Ok(entradas_vec)) => rsx!
                {
                    ul
                    {
                        class: "barra-hilo",
                        for (indice, entrada) in entradas_vec.iter().enumerate()
                        {
                            TarjetaDiario { key: "{indice}", entrada: entrada.clone() }
                        }
                    }
                },
            }
        }
    }
}

/// Cuerpo de un perfil ya cargado: contadores, accesos o seguir, recetas y diario.
#[component]
fn CuerpoPerfil(perfil: PerfilPublico) -> Element {
    let sesion = use_context::<SenalSesion>(); // Para saber si se puede seguir
    let sigue = use_signal(|| perfil.lo_sigo); // Estado visible del seguimiento
    let con_sesion = sesion.read().is_some();

    // El contador se mueve con el botón, sin esperar al servidor
    let seguidores = perfil.seguidores + i64::from(sigue()) - i64::from(perfil.lo_sigo);

    rsx! {
        p { class: "eyebrow", if perfil.propio { "{crate::i18n::ahora(Texto::TuPerfil)}" } else { "{crate::i18n::ahora(Texto::LaBarra)}" } }
        h1 { "{perfil.alias}" }
        p
        {
            class: "barra-meta",
            role: "status",
            "{crate::i18n::con(
                crate::i18n::actual(),
                Texto::ResumenPerfil,
                &[
                    &perfil.recetas.len().to_string(),
                    &seguidores.max(0).to_string(),
                    &perfil.siguiendo.to_string(),
                ],
            )}"
        }

        // Perfil propio: accesos de la cuenta; ajeno con sesión: seguir
        if perfil.propio
        {
            AccesosPropios {}
        }
        else if con_sesion
        {
            BotonSeguir { alias: perfil.alias.clone(), sigue }
        }

        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-perfil-recetas",

            p { id: "titulo-perfil-recetas", class: "eyebrow", "{crate::i18n::ahora(Texto::RecetasPublicadas)}" }
            if perfil.recetas.is_empty()
            {
                p { class: "estado", "{crate::i18n::ahora(Texto::RecetasPublicadasVacio)}" }
            }
            ul
            {
                class: "muro",
                for tarjeta in perfil.recetas.iter()
                {
                    TarjetaMuro { key: "{tarjeta.receta.id}", tarjeta: tarjeta.clone() }
                }
            }
        }

        if perfil.propio
        {
            MiDiario {}
        }
    }
}

/// Perfil propio (alias vacío) o público (con alias), con sus recetas publicadas (E2 y E3).
#[component]
pub fn PantallaPerfil(alias: String) -> Element {
    let sesion = use_context::<SenalSesion>(); // Alias propio y token
    let mut recurso = use_resource(use_reactive!(|(alias,)| {
        let propio = sesion
            .read()
            .as_ref()
            .map(|guardada| guardada.perfil.alias.clone());
        let token = crate::sesion::token(&sesion);
        async move {
            // Alias vacío: el perfil de la sesión, o nada si no hay sesión
            let buscado = if alias.is_empty() {
                propio
            } else {
                Some(alias)
            };
            match buscado {
                Some(buscado) => {
                    crate::red::con_reintentos(move || {
                        api::perfil::perfil(buscado.clone(), token.clone())
                    })
                    .await
                }
                None => Ok(None),
            }
        }
    })); // Perfil del servidor
    let con_sesion = sesion.read().is_some();

    rsx! {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
            }

            // Elegir contenido según el estado de la petición
            match &*recurso.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoPerfil)}" }
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
                Some(Ok(None)) if !con_sesion => rsx!
                {
                    div
                    {
                        class: "estado",
                        p { "{crate::i18n::ahora(Texto::PerfilSinSesion)}" }
                        Link { to: Rutas::Entrar, class: "enlace", "{crate::i18n::ahora(Texto::Entrar)}" }
                    }
                },
                Some(Ok(None)) => rsx!
                {
                    p { class: "estado", "{crate::i18n::ahora(Texto::PerfilNoExiste)}" }
                },
                Some(Ok(Some(perfil))) => rsx!
                {
                    // La clave rehace el cuerpo al pasar de un perfil a otro
                    CuerpoPerfil { key: "{perfil.alias}", perfil: perfil.clone() }
                },
            }
        }

        BarraInferior { activo: Destino::Perfil }
    }
}
