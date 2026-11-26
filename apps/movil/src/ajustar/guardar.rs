//! Acciones de C1: restablecer el editor y guardar la versión privada (C2).

use api::guardar::{ComponenteAgregado, ComponenteNuevo, RecetaNueva, guardar_receta};
use api::receta::RecetaCompleta;
use api::vasos::VasoOpcion;
use dioxus::prelude::*;

use super::CapaAgregada;
use crate::i18n::Texto;
use crate::rutas::Rutas; // Textos traducibles

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
    escala: Signal<f64>,
    seleccion_ingrediente: Signal<String>,
    indice_vaso: Signal<usize>,
    indice_inicial: usize,
) -> Element {
    // Nombre de la versión: se propone uno y se puede cambiar antes de guardar
    let nombre_propuesto = crate::i18n::con(
        crate::i18n::actual(),
        Texto::NombreVersionPropuesto,
        &[&receta.receta.nombre],
    )
    .chars()
    .take(120)
    .collect::<String>();
    let mut nombre = use_signal(|| nombre_propuesto.clone()); // Editable antes de guardar
    let nombre_para_reset = nombre_propuesto.clone(); // Copia para el botón de restablecer
    let mut factores = factores;
    let mut agregados = agregados;
    let mut seleccion_ingrediente = seleccion_ingrediente;
    let mut indice_vaso = indice_vaso;
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut pendientes = use_context::<crate::local::SenalPendientes>();
    let mut guardando = use_signal(|| false);
    let mut guardada = use_signal(|| None::<String>);
    let mut aviso = use_signal(|| None::<String>);
    let total_ingredientes = receta.receta.ingredientes.len();
    let receta_para_guardar = receta;
    let vaso_para_guardar = vaso;
    let propuesto_para_guardar = nombre_propuesto.clone(); // Copia para el callback

    // Si la receta es de la cuenta, los cambios se pueden guardar en ella misma
    let propia = crate::foro::es_propia(&sesion, &receta_para_guardar);
    let receta_propia_id = receta_para_guardar.id.clone();

    // Guardar la versión; con `publicar` además se publica en La Barra (D1)
    let guardar_version = use_callback(move |accion: u8| {
        let publicar = accion == 1;
        if *guardando.read() {
            return;
        }

        let Some(token) = crate::sesion::token(&sesion) else {
            aviso.set(Some(
                { crate::i18n::ahora(Texto::IniciaSesionGuardarVersion) }.to_string(),
            ));
            return;
        };

        let factores_guardar = factores.read().clone();
        let escala_guardar = *escala.read();
        let mut componentes = Vec::new();

        // C1 muestra todas las capas del motor, también las mg (1 g = 1 ml),
        // así que se envían todas con su factor y conservan su unidad original.
        for (indice_capa, capa) in receta_para_guardar.componentes.iter().enumerate() {
            let factor = factores_guardar.get(indice_capa).copied().unwrap_or(1.0);

            // Un factor cero quita la capa; no se guarda una cantidad invalida.
            if factor <= 0.0 {
                continue;
            }

            componentes.push(ComponenteNuevo {
                ingrediente_id: capa.ingrediente_id.clone(),
                posicion: capa.posicion,
                // La escala global también se guarda, para que la versión
                // quede con el volumen que se ve en el vaso.
                cantidad: capa.cantidad * factor * escala_guardar,
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
                cantidad: extra.cantidad * extra.factor * escala_guardar,
            });
        }

        // El nombre lo escribe la persona; si lo dejó vacío se usa el propuesto
        let escrito = nombre.read().trim().to_string();
        let nombre_version = if escrito.is_empty() {
            propuesto_para_guardar.clone()
        } else {
            escrito.chars().take(120).collect::<String>()
        };

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

        let receta_propia_id = receta_propia_id.clone(); // Copia para esta llamada

        spawn(async move {
            let guardado = if accion == 2 {
                crate::red::con_plazo(api::guardar::actualizar_receta(
                    token.clone(),
                    receta_propia_id.clone(),
                    datos.clone(),
                ))
                .await
            } else {
                crate::red::con_plazo(guardar_receta(token.clone(), datos.clone())).await
            };

            match guardado {
                Ok(resultado) => {
                    if accion == 2 {
                        aviso.set(Some(
                            crate::i18n::ahora(Texto::RecetaActualizada).to_string(),
                        ));
                        guardada.set(Some(resultado.receta_id.clone()));
                        guardando.set(false);
                        return;
                    }

                    let mensaje = if resultado.advertencias.is_empty() {
                        crate::i18n::ahora(Texto::VersionGuardada).to_string()
                    } else {
                        let avisos = resultado
                            .advertencias
                            .iter()
                            .map(|aviso| crate::api_textos::aviso(*aviso))
                            .collect::<Vec<_>>()
                            .join(" ");

                        crate::i18n::con(
                            crate::i18n::actual(),
                            Texto::VersionGuardadaConAviso,
                            &[&avisos],
                        )
                    };
                    aviso.set(Some(mensaje));
                    guardada.set(Some(resultado.receta_id.clone()));

                    // Publicar en La Barra solo si se pidió y la versión se guardó
                    if publicar {
                        aviso.set(Some(publicar_version(&token, &resultado.receta_id).await));
                    }
                }
                // Sin red: la versión queda en la cola con su mismo id
                Err(causa)
                    if accion != 2
                        && crate::red::sin_conexion(&causa)
                        && crate::local::encolar(
                            &sesion,
                            &mut pendientes,
                            crate::local::Cambio::Version(datos),
                        ) =>
                {
                    aviso.set(Some(if publicar {
                        crate::i18n::ahora(Texto::VersionGuardadaSinConexionPublicar).to_string()
                    } else {
                        crate::i18n::ahora(Texto::VersionSinConexion).to_string()
                    }));
                }
                Err(causa) => {
                    aviso.set(Some(crate::i18n::con(
                        crate::i18n::actual(),
                        Texto::NoSeGuardoVersion,
                        &[&crate::red::mensaje(&causa)],
                    )));
                }
            }

            guardando.set(false);
        });
    });

    rsx! {
        div { class: "ajustar-acciones",
            // Nombre de la versión: se propone uno y se cambia tocándolo
            label { class: "ajustar-nombre", r#for: "nombre-version",
                span { "{crate::i18n::ahora(Texto::NombreDeLaVersion)}" }
                input {
                    id: "nombre-version",
                    class: "ajustar-select",
                    r#type: "text",
                    maxlength: "120",
                    placeholder: "{nombre_propuesto}",
                    value: "{nombre}",
                    oninput: move |evento| nombre.set(evento.value()),
                }
            }

            button {
                class: "chip",
                r#type: "button",
                onclick: move |_| {
                    factores.set(vec![1.0; total_ingredientes]);
                    agregados.set(Vec::new());
                    seleccion_ingrediente.set(String::new());
                    indice_vaso.set(indice_inicial);
                    escala.set(1.0);
                    nombre.set(nombre_para_reset.clone());
                },
                {crate::i18n::ahora(Texto::Restablecer)}
            }

            button {
                class: "chip",
                r#type: "button",
                disabled: *guardando.read(),
                onclick: move |_| guardar_version.call(0),
                if *guardando.read() {
                    {crate::i18n::ahora(Texto::Guardando)}
                } else {
                    {crate::i18n::ahora(Texto::GuardarVersionPrivada)}
                }
            }

            // La misma versión, publicada en La Barra al terminar de guardarla
            button {
                class: "chip chip--activo",
                r#type: "button",
                disabled: *guardando.read(),
                onclick: move |_| guardar_version.call(1),
                if *guardando.read() {
                    {crate::i18n::ahora(Texto::Guardando)}
                } else {
                    {crate::i18n::ahora(Texto::GuardarYPublicar)}
                }
            }

            // Receta propia: los cambios se guardan en ella, sin crear otra versión
            if propia {
                button {
                    class: "chip",
                    r#type: "button",
                    disabled: *guardando.read(),
                    onclick: move |_| guardar_version.call(2),
                    if *guardando.read() {
                        {crate::i18n::ahora(Texto::Guardando)}
                    } else {
                        {crate::i18n::ahora(Texto::GuardarCambios)}
                    }
                }
            }

            if let Some(id) = guardada.read().as_ref() {
                Link {
                    to: Rutas::Ficha { id: id.clone() },
                    class: "mini",
                    {crate::i18n::ahora(Texto::AbrirVersion)}
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

/// Publica una versión recién guardada y devuelve el aviso que se muestra.
async fn publicar_version(token: &str, receta_id: &str) -> String {
    match crate::red::con_plazo(api::foro::publicar(
        token.to_string(),
        receta_id.to_string(),
        true,
    ))
    .await
    {
        Ok(_) => crate::i18n::ahora(Texto::VersionGuardadaYPublicada).to_string(),
        Err(causa) => crate::i18n::con(
            crate::i18n::actual(),
            Texto::VersionGuardadaSinPublicar,
            &[&crate::red::mensaje(&causa)],
        ),
    }
}
