//! Preparación guiada (C3) con temporizador por paso, parte de la app.
//!
//! Muestra un paso a la vez con su temporizador, si lo tiene, y al terminar pregunta si la
//! receta salió (D4). Lee la receta con A2, así que se puede preparar sin conexión.

// LIBRERÍAS

use std::time::Duration; // Tic del temporizador

use dioxus::prelude::*; // Componentes, señales y recursos

use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Ruta para volver a la ficha
use crate::sesion::SenalSesion; // Cuenta, alias y token

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Tiempo restante en «m:ss».
fn reloj(segundos: u32) -> String
{
    format!("{}:{:02}", segundos / 60, segundos % 60)
}

/// Pantalla C3: los pasos de la receta uno a uno, con temporizador y valoración al final.
#[component]
pub fn PantallaPreparar(id: String) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let id_recurso = id.clone();
    let mut recurso = use_resource(use_reactive!(|(id_recurso,)|
    {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        crate::red::con_reintentos(move || crate::local::receta(id_recurso.clone(), token.clone(), cuenta.clone()))
    })); // Receta de la ficha
    let mut paso_sel = use_signal(|| 0_usize); // Paso visible (el último + 1 es «terminado»)
    let mut restante_sel = use_signal(|| None::<u32>); // Segundos que faltan del temporizador
    let mut corriendo = use_signal(|| false); // Si el temporizador avanza

    // Al cargar la receta, dejar listo el temporizador del primer paso
    use_effect(move ||
    {
        if let Some(Ok(Some(receta))) = &*recurso.read()
            && *paso_sel.peek() == 0
            && restante_sel.peek().is_none()
            && let Some(segundos) = receta.pasos.first().and_then(|paso| paso.temporizador_s)
        {
            restante_sel.set(Some(segundos.max(0) as u32));
        }
    });

    // Avanzar el temporizador un segundo a la vez mientras la pantalla esté abierta
    use_future(move || async move
    {
        loop
        {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if !corriendo()
            {
                continue; // En pausa: esperar
            }
            match restante_sel()
            {
                Some(segundos) if segundos > 1 => restante_sel.set(Some(segundos - 1)),
                Some(_) =>
                {
                    // Tiempo cumplido: dejarlo en cero y detenerlo
                    restante_sel.set(Some(0));
                    corriendo.set(false);
                }
                None => corriendo.set(false), // Paso sin temporizador
            }
        }
    });

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::Ficha { id: id.clone() }, class: "enlace", aria_label: "Salir de la preparación", "‹ Receta" }
            }

            match &*recurso.read()
            {
                None => rsx! { p { class: "estado", role: "status", "Cargando la receta…" } },
                Some(Err(causa)) => rsx!
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso.restart(),
                    }
                },
                Some(Ok(None)) => rsx! { p { class: "estado", "Esta receta no existe o ya no está disponible." } },
                Some(Ok(Some(receta))) if receta.pasos.is_empty() => rsx!
                {
                    h1 { "{receta.receta.nombre}" }
                    p { class: "estado", "Esta receta todavía no tiene pasos de preparación." }
                },
                Some(Ok(Some(receta))) =>
                {
                    let total = receta.pasos.len();
                    let paso = paso_sel().min(total);
                    let alias = sesion.read().as_ref().map(|guardada| guardada.perfil.alias.to_lowercase());
                    let propia = alias.as_deref() == Some(receta.autor.to_lowercase().as_str()); // Los alias no distinguen mayúsculas

                    // Ir a otro paso y dejar listo su temporizador, sin arrancarlo
                    let pasos_vec = receta.pasos.clone();
                    let mut ir_a = move |destino: usize|
                    {
                        paso_sel.set(destino);
                        corriendo.set(false);
                        restante_sel.set(pasos_vec.get(destino).and_then(|paso| paso.temporizador_s).map(|s| s.max(0) as u32));
                    };

                    rsx!
                    {
                        p { class: "eyebrow", "Preparar · {receta.receta.nombre}" }

                        if paso < total
                        {
                            {
                                let actual = receta.pasos[paso].clone();
                                let mut ir_atras = ir_a.clone();
                                rsx!
                                {
                                    p { class: "barra-meta", "Paso {paso + 1} de {total}" }
                                    h1 { "{actual.titulo}" }
                                    p { class: "bienvenida-texto", "{actual.instruccion}" }

                                    // Paso con temporizador: reloj y control
                                    if let Some(segundos) = restante_sel()
                                    {
                                        div
                                        {
                                            class: "preparar-reloj",
                                            p { class: "preparar-tiempo", "aria-live": "polite", role: "timer", "{reloj(segundos)}" }
                                            if segundos == 0
                                            {
                                                p { class: "aviso-pendientes", role: "alert", "¡Tiempo!" }
                                            }
                                            else
                                            {
                                                button
                                                {
                                                    class: "chip",
                                                    r#type: "button",
                                                    onclick: move |_| corriendo.set(!corriendo()),
                                                    if corriendo() { "Pausar" } else { "Iniciar temporizador" }
                                                }
                                            }
                                        }
                                    }

                                    div
                                    {
                                        class: "acciones-fila",
                                        if paso > 0
                                        {
                                            button { class: "chip", r#type: "button", onclick: move |_| ir_atras(paso - 1), "Anterior" }
                                        }
                                        button
                                        {
                                            class: "primario",
                                            r#type: "button",
                                            onclick: move |_| ir_a(paso + 1),
                                            if paso + 1 == total { "Terminar" } else { "Siguiente paso" }
                                        }
                                    }
                                }
                            }
                        }
                        else
                        {
                            // Terminado: preguntar si salió (D4)
                            h1 { "¡Listo!" }
                            p { class: "bienvenida-texto", "¿Te salió igual que en la ficha?" }
                            crate::foro::PanelValoracion { receta_id: receta.id.clone(), propia }
                            div
                            {
                                class: "acciones-fila",
                                button { class: "chip", r#type: "button", onclick: move |_| ir_a(0), "Preparar otra vez" }
                                Link { to: Rutas::Ficha { id: receta.id.clone() }, class: "chip", "Volver a la receta" }
                            }
                        }
                    }
                },
            }
        }
    }
}
