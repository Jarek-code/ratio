//! Acciones de C1: restablecer el editor y guardar la versión privada (C2).

use api::guardar::{ComponenteAgregado, ComponenteNuevo, RecetaNueva, guardar_receta};
use api::receta::RecetaCompleta;
use api::vasos::VasoOpcion;
use dioxus::prelude::*;

use super::CapaAgregada;
use crate::rutas::Rutas;

/// Botones de restablecer y guardar, con el aviso del resultado.
///
/// `vaso` es el vaso elegido e `indice_inicial` la posición del vaso de la receta, al que
/// vuelve «Restablecer».
#[component]
pub(super) fn Acciones(
    receta: RecetaCompleta,
    vaso: VasoOpcion,
    factores: Signal<Vec<f64>>,
    agregados: Signal<Vec<CapaAgregada>>,
    seleccion_ingrediente: Signal<String>,
    comensales: Signal<u32>,
    indice_vaso: Signal<usize>,
    indice_inicial: usize,
) -> Element {
    let mut factores = factores;
    let mut agregados = agregados;
    let mut seleccion_ingrediente = seleccion_ingrediente;
    let mut comensales = comensales;
    let mut indice_vaso = indice_vaso;
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut pendientes = use_context::<crate::local::SenalPendientes>();
    let mut guardando = use_signal(|| false);
    let mut guardada = use_signal(|| None::<String>);
    let mut aviso = use_signal(|| None::<String>);
    let total_ingredientes = receta.receta.ingredientes.len();
    let receta_para_guardar = receta;
    let vaso_para_guardar = vaso;

    rsx! {
        div { class: "ajustar-acciones",
            button {
                class: "chip",
                r#type: "button",
                onclick: move |_| {
                    factores.set(vec![1.0; total_ingredientes]);
                    agregados.set(Vec::new());
                    seleccion_ingrediente.set(String::new());
                    comensales.set(1);
                    indice_vaso.set(indice_inicial);
                },
                "Restablecer"
            }

            button {
                class: "chip",
                r#type: "button",
                disabled: *guardando.read(),
                onclick: move |_| {
                    if *guardando.read() {
                        return;
                    }

                    let Some(token) = crate::sesion::token(&sesion) else {
                        aviso.set(Some("Inicia sesión para guardar una versión.".to_string()));
                        return;
                    };

                    // La version guarda las proporciones por comensal; los comensales
                    // siguen siendo un ajuste de la vista y no una propiedad de la receta.
                    let factores_guardar = factores.read().clone();
                    let mut componentes = Vec::new();

                    // C1 muestra todas las capas del motor, también las mg (1 g = 1 ml),
                    // así que se envían todas con su factor y conservan su unidad original.
                    for (indice_capa, capa) in
                        receta_para_guardar.componentes.iter().enumerate()
                    {
                        let factor = factores_guardar
                            .get(indice_capa)
                            .copied()
                            .unwrap_or(1.0);

                        // Un factor cero quita la capa; no se guarda una cantidad invalida.
                        if factor <= 0.0 {
                            continue;
                        }

                        componentes.push(ComponenteNuevo {
                            ingrediente_id: capa.ingrediente_id.clone(),
                            posicion: capa.posicion,
                            cantidad: capa.cantidad * factor,
                        });
                    }

                    // Las capas nuevas viajan aparte: el servidor valida su metadata
                    // contra el catálogo y las coloca encima de las del origen.
                    let mut agregados_guardar = Vec::new();
                    for extra in agregados.read().iter() {
                        if extra.factor <= 0.0 {
                            continue;
                        }

                        agregados_guardar.push(ComponenteAgregado {
                            ingrediente_id: extra.ingrediente_id.clone(),
                            cantidad: extra.cantidad * extra.factor,
                        });
                    }

                    let nombre_version = format!(
                        "{} · versión",
                        receta_para_guardar.receta.nombre
                    )
                    .chars()
                    .take(120)
                    .collect::<String>();

                    let datos = RecetaNueva {
                        origen_id: receta_para_guardar.id.clone(),
                        nombre: nombre_version,
                        vaso_id: vaso_para_guardar.id.clone(),
                        capacidad_ml: vaso_para_guardar.vaso.capacidad_ml,
                        componentes,
                        agregados: agregados_guardar,
                        id: crate::local::nuevo_id(),
                    };

                    guardando.set(true);
                    guardada.set(None);
                    aviso.set(None);

                    spawn(async move {
                        match crate::red::con_plazo(guardar_receta(token, datos.clone())).await {
                            Ok(resultado) => {
                                let mensaje = if resultado.advertencias.is_empty() {
                                    "Versión guardada. Ya aparece en Mi recetario.".to_string()
                                } else {
                                    format!(
                                        "Versión guardada. {}",
                                        resultado.advertencias.join(" ")
                                    )
                                };
                                aviso.set(Some(mensaje));
                                guardada.set(Some(resultado.receta_id));
                            }
                            // Sin red: la versión queda en la cola con su mismo id
                            Err(causa)
                                if crate::local::es_sin_conexion(&causa)
                                    && crate::local::encolar(
                                        &sesion,
                                        &mut pendientes,
                                        crate::local::Cambio::Version(datos),
                                    ) =>
                            {
                                aviso.set(Some(
                                    "Sin conexión: la versión se enviará cuando vuelva la red y aparecerá en Mi recetario."
                                        .to_string(),
                                ));
                            }
                            Err(causa) => {
                                aviso.set(Some(format!(
                                    "No pudimos guardar la versión: {}",
                                    crate::red::mensaje(&causa)
                                )));
                            }
                        }

                        guardando.set(false);
                    });
                },
                if *guardando.read() {
                    "Guardando…"
                } else {
                    "Guardar versión privada"
                }
            }

            if let Some(id) = guardada.read().as_ref() {
                Link {
                    to: Rutas::Ficha { id: id.clone() },
                    class: "mini",
                    "Abrir versión"
                }
            }
        }

        if let Some(mensaje) = aviso.read().as_ref() {
            p {
                class: "ajustar-aviso",
                role: "status",
                "{mensaje}"
            }
        }
    }
}
