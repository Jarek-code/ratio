//! Preparación guiada (C3) con temporizador por paso, parte de la app.
//!
//! Muestra un paso a la vez con su temporizador, si lo tiene, y al terminar pregunta si la
//! receta salió (D4) y guarda cuánto tomó la preparación. Lee la receta con A2, así que se
//! puede preparar sin conexión.

// LIBRERÍAS

use std::time::Duration; // Tic del temporizador
use std::time::Instant; // Momento en que empezó la preparación

use dioxus::prelude::*; // Componentes, señales y recursos

use crate::i18n::Texto;
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Ruta para volver a la ficha
use crate::sesion::SenalSesion; // Cuenta, alias y token // Textos traducibles

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// FUNCIONES

/// Tiempo restante en «m:ss».
fn reloj(segundos: u32) -> String {
    format!("{}:{:02}", segundos / 60, segundos % 60)
}

/// Pantalla C3: los pasos de la receta uno a uno, con temporizador y valoración al final.
#[component]
pub fn PantallaPreparar(id: String) -> Element {
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let id_recurso = id.clone();
    let mut recurso = use_resource(use_reactive!(|(id_recurso,)| {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        crate::red::con_reintentos(move || {
            crate::local::receta(id_recurso.clone(), token.clone(), cuenta.clone())
        })
    })); // Receta de la ficha
    let mut paso_sel = use_signal(|| 0_usize); // Paso visible (el último + 1 es «terminado»)
    let mut restante_sel = use_signal(|| None::<u32>); // Segundos que faltan del temporizador
    let mut corriendo = use_signal(|| false); // Si el temporizador avanza
    let mut inicio = use_signal(Instant::now); // Cuándo empezó esta preparación
    let mut duracion_sel = use_signal(|| None::<i32>); // Lo que tomó, al terminar

    // Al cargar la receta, dejar listo el temporizador del primer paso
    use_effect(move || {
        if let Some(Ok(Some(receta))) = &*recurso.read()
            && *paso_sel.peek() == 0
            && restante_sel.peek().is_none()
            && let Some(segundos) = receta.pasos.first().and_then(|paso| paso.temporizador_s)
        {
            restante_sel.set(Some(segundos.max(0) as u32));
        }
    });

    // Avanzar el temporizador un segundo a la vez mientras la pantalla esté abierta
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if !corriendo() {
                continue; // En pausa: esperar
            }
            match restante_sel() {
                Some(segundos) if segundos > 1 => restante_sel.set(Some(segundos - 1)),
                Some(_) => {
                    // Tiempo cumplido: dejarlo en cero y detenerlo
                    restante_sel.set(Some(0));
                    corriendo.set(false);
                }
                None => corriendo.set(false), // Paso sin temporizador
            }
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                crate::rutas::Volver { etiqueta: crate::i18n::ahora(Texto::SalirDeLaPreparacion).to_string(), respaldo: Rutas::Ficha { id: id.clone() } }
            }

            match &*recurso.read()
            {
                None => rsx! { p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoReceta)}" } },
                Some(Err(causa)) => rsx!
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso.restart(),
                    }
                },
                Some(Ok(None)) => rsx! { p { class: "estado", "{crate::i18n::ahora(Texto::RecetaNoDisponible)}" } },
                Some(Ok(Some(receta))) if receta.pasos.is_empty() => rsx!
                {
                    h1 { "{receta.receta.nombre}" }
                    p { class: "estado", "{crate::i18n::ahora(Texto::RecetaSinPasos)}" }
                },
                Some(Ok(Some(receta))) =>
                {
                    let total = receta.pasos.len();
                    let paso = paso_sel().min(total);
                    let propia = crate::foro::es_propia(&sesion, receta);

                    // Ir a otro paso y dejar listo su temporizador, sin arrancarlo
                    let pasos_vec = receta.pasos.clone();
                    let mut ir_a = move |destino: usize|
                    {
                        // Al terminar se fija la duración; al volver a empezar se mide de nuevo
                        if destino >= total
                        {
                            let segundos = inicio.peek().elapsed().as_secs().min(api::diario::DURACION_MAXIMA_S as u64);
                            duracion_sel.set(Some(segundos as i32));
                        }
                        else if destino == 0
                        {
                            inicio.set(Instant::now());
                            duracion_sel.set(None);
                        }
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
                                    p { class: "barra-meta", "{crate::i18n::con(crate::i18n::actual(), Texto::PasoDeTotal, &[&(paso + 1).to_string(), &total.to_string()])}" }
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
                                                p { class: "aviso-pendientes", role: "alert", "{crate::i18n::ahora(Texto::Tiempo)}" }
                                            }
                                            else
                                            {
                                                button
                                                {
                                                    class: "chip",
                                                    r#type: "button",
                                                    onclick: move |_| corriendo.set(!corriendo()),
                                                    if corriendo() { {crate::i18n::ahora(Texto::Pausar)} } else { {crate::i18n::ahora(Texto::IniciarTemporizador)} }
                                                }
                                            }
                                        }
                                    }

                                    div
                                    {
                                        class: "acciones-fila",
                                        if paso > 0
                                        {
                                            button { class: "chip", r#type: "button", onclick: move |_| ir_atras(paso - 1), "{crate::i18n::ahora(Texto::Anterior)}" }
                                        }
                                        button
                                        {
                                            class: "primario",
                                            r#type: "button",
                                            onclick: move |_| ir_a(paso + 1),
                                            if paso + 1 == total { {crate::i18n::ahora(Texto::Terminar)} } else { {crate::i18n::ahora(Texto::SiguientePaso)} }
                                        }
                                    }
                                }
                            }
                        }
                        else
                        {
                            // Terminado: preguntar si salió (D4)
                            h1 { "{crate::i18n::ahora(Texto::Listo)}" }
                            p { class: "bienvenida-texto", "{crate::i18n::ahora(Texto::TeSalioIgual)}" }
                            if let Some(segundos) = duracion_sel()
                            {
                                p { class: "barra-meta", "{crate::i18n::con(crate::i18n::actual(), Texto::TiempoDePreparacion, &[&crate::api_textos::duracion(segundos)])}" }
                            }
                            crate::foro::PanelValoracion { receta_id: receta.id.clone(), propia, duracion_s: duracion_sel() }
                            div
                            {
                                class: "acciones-fila",
                                button { class: "chip", r#type: "button", onclick: move |_| ir_a(0), "{crate::i18n::ahora(Texto::PrepararOtraVez)}" }
                                Link { to: Rutas::Ficha { id: receta.id.clone() }, class: "chip", "{crate::i18n::ahora(Texto::VolverALaReceta)}" }
                            }
                        }
                    }
                },
            }
        }
    }
}
