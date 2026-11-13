//! Pantalla C5: las recetas que la cuenta guardó para volver a prepararlas.
//!
//! Reutiliza la miniatura del vaso de la biblioteca y el resumen de receta del backend.
//! Quitar una receta no la borra: solo elimina la fila de `guardado`, por eso la lista
//! se recarga en el sitio sin salir de la pantalla.

use api::guardados::{RecetaGuardada, alternar_guardado, guardados};
use dioxus::prelude::*;

use crate::barra::{BarraInferior, Destino};
use crate::biblioteca::familia_legible;
use crate::rutas::Rutas;

/// Hoja de estilos propia de la pantalla Guardados.
const CSS_GUARDADOS: Asset = asset!("/assets/guardados.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Pantalla C5: carga la lista de guardados de la cuenta.
#[component]
pub fn PantallaGuardados() -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let recarga = use_signal(|| 0_u32);
    let mut recurso = use_resource(move || {
        // Leer la señal dentro del recurso lo vuelve a ejecutar al quitar una receta.
        let _ = recarga();
        let token = crate::sesion::token(&sesion);
        async move {
            match token {
                Some(token) => guardados(token).await.map(Some),
                None => Ok(None),
            }
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_GUARDADOS }

        main { class: "guardados",
            BarraSuperior {}

            header { class: "guardados-cabeza",
                p { class: "eyebrow", "Tu lista" }
                h1 { "Guardados" }
                p { class: "guardados-sub", "Las recetas que quieres volver a preparar." }
            }

            crate::local::AvisoPendientes {}

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando tus guardados…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos traer tus guardados: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| recurso.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(None)) => rsx! {
                    SinSesion {}
                },
                Some(Ok(Some(lista))) => rsx! {
                    ListaGuardados { lista: lista.clone(), recarga }
                },
            }
        }

        BarraInferior { activo: Destino::Perfil }
    }
}

/// Barra superior con la vuelta a Descubrir.
#[component]
fn BarraSuperior() -> Element {
    rsx! {
        header { class: "guardados-barra",
            Link {
                to: Rutas::Descubrir,
                class: "guardados-volver",
                aria_label: "Volver a Descubrir",
                "‹ Descubrir"
            }
            span { class: "guardados-marca", "Ratio" }
        }
    }
}

/// Aviso para quien abre C5 sin sesión.
#[component]
fn SinSesion() -> Element {
    rsx! {
        div { class: "estado",
            p { "Inicia sesión para ver las recetas que guardaste." }
            Link { to: Rutas::Entrar, class: "guardados-accion", "Entrar" }
        }
    }
}

/// Lista de guardados, o el estado vacío con una salida a la biblioteca.
#[component]
fn ListaGuardados(lista: Vec<RecetaGuardada>, recarga: Signal<u32>) -> Element {
    if lista.is_empty() {
        return rsx! {
            div { class: "estado",
                p { "Todavía no has guardado recetas. Abre una ficha y usa «Guardar»." }
                Link { to: Rutas::biblioteca(), class: "guardados-accion", "Ir a la biblioteca" }
            }
        };
    }

    let total = lista.len();
    let resumen = if total == 1 {
        "1 receta guardada".to_string()
    } else {
        format!("{total} recetas guardadas")
    };

    rsx! {
        p { class: "guardados-cuenta", role: "status", "{resumen}" }

        ul { class: "guardados-lista",
            for receta in lista {
                TarjetaGuardada {
                    key: "{receta.receta.id}",
                    receta,
                    recarga,
                }
            }
        }
    }
}

/// Tarjeta de una receta guardada, con su fecha y la acción de quitar.
#[component]
fn TarjetaGuardada(receta: RecetaGuardada, recarga: Signal<u32>) -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut pendientes = use_context::<crate::local::SenalPendientes>();
    let mut quitando = use_signal(|| false);
    let mut aviso = use_signal(|| None::<String>);
    let mut recarga = recarga;

    let resumen = &receta.receta;
    let capas = proporciones::capas(&resumen.receta);
    let graduacion = proporciones::graduacion(&resumen.receta);
    let svg = proporciones::svg(
        &resumen.receta,
        &proporciones::Opciones {
            escalas: false,
            mini: true,
        },
    );

    let mut datos = vec![format!("{:.0} ml", capas.total_ml)];
    if graduacion.abv > 0.0 {
        datos.push(format!("{:.1} % ABV", graduacion.abv));
    }
    if let Some(minutos) = resumen.minutos {
        datos.push(format!("{minutos} min"));
    }

    let id = resumen.id.clone();
    let id_quitar = id.clone();
    let nombre = resumen.receta.nombre.clone();
    let familia = familia_legible(&resumen.familia);
    let fecha = receta.guardado_en.clone();
    let ultima = receta.ultima_preparacion.clone();

    rsx! {
        li { class: "guardado-tarjeta",
            Link {
                to: Rutas::Ficha { id: id.clone() },
                class: "guardado-enlace",
                div { class: "guardado-mini", dangerous_inner_html: svg }
                div { class: "guardado-cuerpo",
                    h2 { "{nombre}" }
                    p { class: "guardado-meta", "{familia} · {datos.join(\" · \")}" }
                    p { class: "guardado-fecha", "Guardado el {fecha}" }
                    if let Some(ultima) = ultima.clone() {
                        p { class: "guardado-ultima", "Última vez: {ultima}" }
                    }
                }
            }

            button {
                class: "guardado-quitar",
                r#type: "button",
                disabled: *quitando.read(),
                aria_label: "Quitar {nombre} de guardados",
                onclick: move |_| {
                    if *quitando.read() {
                        return;
                    }

                    let Some(token) = crate::sesion::token(&sesion) else {
                        aviso.set(Some("La sesión caducó; vuelve a entrar.".to_string()));
                        return;
                    };

                    let id = id_quitar.clone();
                    quitando.set(true);
                    aviso.set(None);
                    spawn(async move {
                        match alternar_guardado(token, id.clone(), false).await {
                            Ok(_) => recarga.set(recarga() + 1),
                            // Sin red: quitarla queda en la cola
                            Err(causa)
                                if crate::local::es_sin_conexion(&causa)
                                    && crate::local::encolar(
                                        &sesion,
                                        &mut pendientes,
                                        crate::local::Cambio::Guardado {
                                            receta_id: id,
                                            guardar: false,
                                        },
                                    ) =>
                            {
                                aviso.set(Some(
                                    "Sin conexión: se quitará cuando vuelva la red.".to_string(),
                                ));
                            }
                            Err(causa) => {
                                aviso.set(Some(format!("No se pudo quitar: {causa}")));
                                quitando.set(false);
                            }
                        }
                    });
                },
                if *quitando.read() { "Quitando…" } else { "Quitar" }
            }

            if let Some(texto) = aviso() {
                p { class: "guardado-error", role: "alert", "{texto}" }
            }
        }
    }
}
