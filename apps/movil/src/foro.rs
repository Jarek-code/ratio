//! La Barra en la app (D1-D4): publicar, valorar, hilo y comentario anclado, parte de la app.
//!
//! La sección de la ficha muestra el recuento de «me salió / no me salió», el hilo con las
//! respuestas bajo su comentario y el formulario para comentar, anclado o no a una capa o a
//! un paso. Las escrituras pasan por la cola sin conexión: sin red quedan pendientes.

// LIBRERÍAS

use std::collections::HashMap; // Agrupación de respuestas por comentario raíz

use api::foro::Ancla; // Capa o paso al que se ancla un comentario
use api::foro::Comentario; // Comentario del hilo
use api::foro::ComentarioNuevo; // Comentario que se envía
use api::foro::Motivo; // Causa de un reporte
use api::foro::ReporteNuevo; // Reporte que se envía
use api::foro::Valoraciones; // Recuento de valoraciones
use api::receta::RecetaCompleta; // Ficha con capas y pasos para elegir el ancla
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::local::Cambio; // Cambio que se encola sin conexión
use crate::local::SenalPendientes; // Contador de cambios pendientes
use crate::sesion::SenalSesion; // Sesión con cuenta, alias y token

// FUNCIONES

/// Convierte el valor del desplegable de ancla («c3», «p1» o vacío) en un ancla.
fn ancla_de(valor: &str) -> Option<Ancla>
{
    let posicion = valor.get(1..)?.parse::<i16>().ok()?; // Posición tras la letra
    match valor.chars().next()
    {
        Some('c') => Some(Ancla::Capa(posicion)), // Capa de la receta
        Some('p') => Some(Ancla::Paso(posicion)), // Paso de la preparación
        _ => None, // Comentario general
    }
}

/// Ordena el hilo: cada comentario raíz seguido de todas sus respuestas, en orden de fecha.
///
/// Una respuesta a otra respuesta cuelga del mismo comentario raíz.
fn ordenar_hilo(comentarios: &[Comentario]) -> Vec<(Comentario, bool)>
{
    let padres_map: HashMap<&str, Option<&str>> = comentarios
        .iter()
        .map(|comentario| (comentario.id.as_str(), comentario.respuesta_a.as_deref()))
        .collect(); // Padre directo de cada comentario

    // Subir por la cadena de respuestas hasta el comentario raíz
    let raiz_de = |id: &str| -> String
    {
        let mut actual = id;
        let mut contador_saltos = 0; // Corta una cadena rota o circular
        while let Some(Some(padre)) = padres_map.get(actual)
        {
            if !padres_map.contains_key(padre) || contador_saltos > comentarios.len()
            {
                break; // Padre ausente u oculto: queda como raíz
            }
            actual = padre;
            contador_saltos += 1;
        }
        actual.to_string() // Comentario raíz de la cadena
    };

    let mut hilo_vec = Vec::new(); // Comentarios en orden de lectura
    for raiz in comentarios.iter().filter(|comentario| raiz_de(&comentario.id) == comentario.id)
    {
        hilo_vec.push((raiz.clone(), false));
        for respuesta in comentarios
            .iter()
            .filter(|otro| otro.id != raiz.id && raiz_de(&otro.id) == raiz.id)
        {
            hilo_vec.push((respuesta.clone(), true));
        }
    }

    hilo_vec // Raíces con sus respuestas debajo
}

/// Botón de Mi recetario que publica una receta propia en La Barra o la retira (D1).
///
/// `publicada` es el estado actual y `al_cambiar` avisa para recargar la lista.
#[component]
pub fn BotonPublicar(receta_id: String, publicada: bool, al_cambiar: EventHandler<()>) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para publicar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut enviando = use_signal(|| false); // Petición en curso
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    rsx!
    {
        div
        {
            class: "propia-acciones",

            button
            {
                class: "chip",
                r#type: "button",
                disabled: enviando(),
                onclick: move |_|
                {
                    let Some(token) = crate::sesion::token(&sesion) else
                    {
                        aviso.set(Some("Vuelve a entrar para publicar.".to_string()));
                        return; // Sin sesión no se publica
                    };
                    let id = receta_id.clone();
                    let publica = !publicada;
                    enviando.set(true);
                    aviso.set(None);

                    spawn(async move
                    {
                        match api::foro::publicar(token, id.clone(), publica).await
                        {
                            Ok(_) => al_cambiar.call(()), // Recargar la lista con el estado nuevo
                            Err(causa) if crate::local::es_sin_conexion(&causa)
                                && crate::local::encolar(&sesion, &mut pendientes, Cambio::Publicacion { receta_id: id, publica }) =>
                            {
                                aviso.set(Some("Sin conexión: se aplicará cuando vuelva la red.".to_string()));
                            }
                            Err(_) => aviso.set(Some("No se pudo cambiar la publicación.".to_string())),
                        }
                        enviando.set(false);
                    });
                },

                if enviando() { "Un momento…" } else if publicada { "Retirar de La Barra" } else { "Publicar en La Barra" }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "status", "{texto}" }
            }
        }
    }
}

/// Recuento de «me salió / no me salió» con los botones para valorar (D4).
///
/// `propia` oculta los botones: quien publicó la receta no la valora.
#[component]
fn PanelValoracion(receta_id: String, propia: bool) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para leer y valorar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let id_recurso = receta_id.clone();
    let recurso = use_resource(move ||
    {
        let token = crate::sesion::token(&sesion);
        api::foro::valoraciones(id_recurso.clone(), token)
    }); // Recuento del servidor
    let mut local_sel = use_signal(|| None::<Valoraciones>); // Recuento tras valorar
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    let actual = local_sel().or_else(|| recurso.read().as_ref().and_then(|res| res.as_ref().ok().copied())); // Recuento a mostrar
    let con_sesion = sesion.read().is_some();

    // Enviar una valoración, o encolarla sin conexión
    let valorar = use_callback(move |me_salio: bool|
    {
        let Some(token) = crate::sesion::token(&sesion) else
        {
            return; // Sin sesión los botones no se muestran
        };
        let id = receta_id.clone();
        aviso.set(None);

        spawn(async move
        {
            match api::foro::valorar(token, id.clone(), me_salio).await
            {
                Ok(recuento) => local_sel.set(Some(recuento)),
                Err(causa) if crate::local::es_sin_conexion(&causa)
                    && crate::local::encolar(&sesion, &mut pendientes, Cambio::Valoracion { receta_id: id, me_salio }) =>
                {
                    aviso.set(Some("Sin conexión: tu valoración se enviará cuando vuelva la red.".to_string()));
                }
                Err(_) => aviso.set(Some("No se pudo guardar tu valoración.".to_string())),
            }
        });
    });

    rsx!
    {
        div
        {
            class: "barra-valoracion",

            if let Some(recuento) = actual
            {
                p
                {
                    class: "barra-recuento",
                    role: "status",
                    "A {recuento.me_salio} les salió · a {recuento.no_me_salio} no les salió"
                }
            }

            // Con sesión y receta ajena: mostrar los botones para valorar
            if con_sesion && !propia
            {
                div
                {
                    class: "acciones-fila",
                    role: "group",
                    "aria-label": "¿Te salió esta receta?",

                    button
                    {
                        class: if actual.and_then(|recuento| recuento.mia) == Some(true) { "chip chip--activo" } else { "chip" },
                        r#type: "button",
                        "aria-pressed": if actual.and_then(|recuento| recuento.mia) == Some(true) { "true" } else { "false" },
                        onclick: move |_| valorar.call(true),
                        "Me salió"
                    }
                    button
                    {
                        class: if actual.and_then(|recuento| recuento.mia) == Some(false) { "chip chip--activo" } else { "chip" },
                        r#type: "button",
                        "aria-pressed": if actual.and_then(|recuento| recuento.mia) == Some(false) { "true" } else { "false" },
                        onclick: move |_| valorar.call(false),
                        "No me salió"
                    }
                }
            }

            if propia
            {
                p { class: "mini", "Es tu receta: quienes la preparen dirán si les salió." }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "status", "{texto}" }
            }
        }
    }
}

/// Formulario para comentar, anclado o no, o para responder a un comentario (D3).
///
/// `respuesta_a` guarda el comentario al que se responde y `al_publicar` recarga el hilo.
#[component]
fn FormularioComentario(
    receta: RecetaCompleta,
    respuesta_a: Signal<Option<Comentario>>,
    al_publicar: EventHandler<()>,
) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para comentar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut respuesta_a = respuesta_a;
    let mut texto_sel = use_signal(String::new); // Texto escrito
    let mut ancla_sel = use_signal(String::new); // Valor del desplegable de ancla
    let mut propuesta_sel = use_signal(String::new); // Cantidad propuesta escrita
    let mut enviando = use_signal(|| false); // Petición en curso
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    let receta_id = receta.id.clone();
    let con_ancla = !ancla_sel().is_empty();

    rsx!
    {
        form
        {
            class: "barra-formulario",
            onsubmit: move |evento|
            {
                evento.prevent_default();
                let texto = texto_sel().trim().to_string();
                if texto.is_empty() || enviando()
                {
                    return; // Nada que enviar
                }
                if crate::sesion::token(&sesion).is_none()
                {
                    aviso.set(Some("Vuelve a entrar para comentar.".to_string()));
                    return; // Sin sesión no se comenta
                }

                // Validar la propuesta: un número positivo, solo con ancla
                let ancla = ancla_de(&ancla_sel());
                let propuesta = match propuesta_sel().trim()
                {
                    "" => None, // Sin propuesta
                    valor => match valor.replace(',', ".").parse::<f64>()
                    {
                        Ok(cantidad) if cantidad > 0.0 && ancla.is_some() => Some(cantidad),
                        _ =>
                        {
                            aviso.set(Some("La propuesta debe ser un número mayor que cero y necesita una capa o un paso.".to_string()));
                            return; // Propuesta inválida
                        }
                    },
                };

                let nuevo = ComentarioNuevo
                {
                    id: crate::local::nuevo_id(),
                    receta_id: receta_id.clone(),
                    texto,
                    respuesta_a: respuesta_a().map(|comentario| comentario.id),
                    ancla,
                    propuesta,
                };
                let token = crate::sesion::token(&sesion).unwrap_or_default(); // Comprobado arriba
                enviando.set(true);
                aviso.set(None);

                spawn(async move
                {
                    match api::foro::comentar(token, nuevo.clone()).await
                    {
                        Ok(_) =>
                        {
                            // Publicado: limpiar el formulario y recargar el hilo
                            texto_sel.set(String::new());
                            propuesta_sel.set(String::new());
                            ancla_sel.set(String::new());
                            respuesta_a.set(None);
                            al_publicar.call(());
                        }
                        Err(causa) if crate::local::es_sin_conexion(&causa)
                            && crate::local::encolar(&sesion, &mut pendientes, Cambio::Comentario(nuevo)) =>
                        {
                            texto_sel.set(String::new());
                            respuesta_a.set(None);
                            aviso.set(Some("Sin conexión: tu comentario se publicará cuando vuelva la red.".to_string()));
                        }
                        Err(_) => aviso.set(Some("No se pudo publicar el comentario.".to_string())),
                    }
                    enviando.set(false);
                });
            },

            if let Some(comentario) = respuesta_a()
            {
                p
                {
                    class: "barra-respondiendo",
                    "Respondiendo a {comentario.autor} · "
                    button { class: "enlace", r#type: "button", onclick: move |_| respuesta_a.set(None), "Cancelar" }
                }
            }

            label { class: "solo-lector", r#for: "comentario-texto", "Tu comentario" }
            textarea
            {
                id: "comentario-texto",
                class: "barra-texto",
                rows: "3",
                maxlength: "2000",
                placeholder: "¿Qué cambiarías o qué te funcionó?",
                value: "{texto_sel}",
                oninput: move |evento| texto_sel.set(evento.value()),
            }

            div
            {
                class: "barra-ancla",

                label { r#for: "comentario-ancla", "Sobre" }
                select
                {
                    id: "comentario-ancla",
                    class: "barra-select",
                    value: "{ancla_sel}",
                    onchange: move |evento| ancla_sel.set(evento.value()),

                    option { value: "", "Toda la receta" }
                    for capa in receta.componentes.iter()
                    {
                        option { key: "c{capa.posicion}", value: "c{capa.posicion}", "Capa · {capa.nombre}" }
                    }
                    for (numero, paso) in receta.pasos.iter().enumerate()
                    {
                        option { key: "p{paso.posicion}", value: "p{paso.posicion}", "Paso {numero + 1} · {paso.titulo}" }
                    }
                }

                // Con ancla: permitir proponer una cantidad
                if con_ancla
                {
                    label { r#for: "comentario-propuesta", "Propuesta (opcional)" }
                    input
                    {
                        id: "comentario-propuesta",
                        class: "barra-propuesta",
                        r#type: "text",
                        inputmode: "decimal",
                        placeholder: if ancla_sel().starts_with('p') { "segundos" } else { "cantidad" },
                        value: "{propuesta_sel}",
                        oninput: move |evento| propuesta_sel.set(evento.value()),
                    }
                }
            }

            button
            {
                class: "primario",
                r#type: "submit",
                disabled: enviando() || texto_sel().trim().is_empty(),
                if enviando() { "Publicando…" } else if respuesta_a().is_some() { "Responder" } else { "Comentar" }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "status", "{texto}" }
            }
        }
    }
}

/// Enlace «Reportar» que despliega el formulario de reporte de una receta o un comentario (D7).
///
/// Se pasa `receta_id` para reportar la receta o `comentario_id` para reportar un comentario.
#[component]
fn Reportar(receta_id: Option<String>, comentario_id: Option<String>) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para reportar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut abierto = use_signal(|| false); // Si el formulario está desplegado
    let mut motivo_sel = use_signal(|| Motivo::Spam); // Causa elegida
    let mut detalle_sel = use_signal(String::new); // Explicación escrita
    let mut aviso = use_signal(|| None::<String>); // Resultado visible
    let que = if comentario_id.is_some() { "comentario" } else { "receta" }; // Qué se reporta
    let sufijo = comentario_id.clone().or(receta_id.clone()).unwrap_or_default(); // Ids únicos en la página

    // Ya enviado: solo el aviso
    if let Some(texto) = aviso()
    {
        return rsx! { p { class: "barra-aviso", role: "status", "{texto}" } };
    }

    rsx!
    {
        if !abierto()
        {
            button
            {
                class: "enlace",
                r#type: "button",
                "aria-label": "Reportar este {que}",
                onclick: move |_| abierto.set(true),
                "Reportar"
            }
        }
        else
        {
            form
            {
                class: "barra-formulario",
                "aria-label": "Reportar este {que}",
                onsubmit: move |evento|
                {
                    evento.prevent_default();
                    let Some(token) = crate::sesion::token(&sesion) else
                    {
                        aviso.set(Some("Vuelve a entrar para reportar.".to_string()));
                        return; // Sin sesión no se reporta
                    };
                    let reporte = ReporteNuevo
                    {
                        receta_id: receta_id.clone(),
                        comentario_id: comentario_id.clone(),
                        motivo: motivo_sel(),
                        detalle: Some(detalle_sel()),
                    };

                    spawn(async move
                    {
                        match api::foro::reportar(token, reporte.clone()).await
                        {
                            Ok(()) => aviso.set(Some("Gracias: el equipo de moderación lo revisará.".to_string())),
                            Err(causa) if crate::local::es_sin_conexion(&causa)
                                && crate::local::encolar(&sesion, &mut pendientes, Cambio::Reporte(reporte)) =>
                            {
                                aviso.set(Some("Sin conexión: el reporte se enviará cuando vuelva la red.".to_string()));
                            }
                            Err(_) => aviso.set(Some("No se pudo enviar el reporte.".to_string())),
                        }
                    });
                },

                label { r#for: "reporte-motivo-{sufijo}", "Motivo" }
                select
                {
                    id: "reporte-motivo-{sufijo}",
                    class: "barra-select",
                    onchange: move |evento|
                    {
                        let clave = evento.value();
                        if let Some(motivo) = Motivo::TODOS.into_iter().find(|motivo| motivo.clave() == clave)
                        {
                            motivo_sel.set(motivo);
                        }
                    },
                    for motivo in Motivo::TODOS
                    {
                        option { key: "{motivo.clave()}", value: "{motivo.clave()}", selected: motivo == motivo_sel(), "{motivo.etiqueta()}" }
                    }
                }

                label { class: "solo-lector", r#for: "reporte-detalle-{sufijo}", "Detalle" }
                textarea
                {
                    id: "reporte-detalle-{sufijo}",
                    class: "barra-texto",
                    rows: "2",
                    maxlength: "2000",
                    placeholder: "Cuéntanos qué pasa (opcional)",
                    value: "{detalle_sel}",
                    oninput: move |evento| detalle_sel.set(evento.value()),
                }

                div
                {
                    class: "acciones-fila",
                    button { class: "primario", r#type: "submit", "Enviar reporte" }
                    button { class: "chip", r#type: "button", onclick: move |_| abierto.set(false), "Cancelar" }
                }
            }
        }
    }
}

/// Sección «La Barra» de la ficha: valoración, hilo, formulario y reportes (D2, D3, D4 y D7).
#[component]
pub fn LaBarra(receta: RecetaCompleta) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Sesión para saber si es propia y si comenta
    let id_recurso = receta.id.clone();
    let mut hilo_res = use_resource(move ||
    {
        let token = crate::sesion::token(&sesion);
        api::foro::hilo(id_recurso.clone(), token)
    }); // Hilo del servidor
    let mut respuesta_a = use_signal(|| None::<Comentario>); // Comentario al que se responde

    let alias = sesion.read().as_ref().map(|guardada| guardada.perfil.alias.to_lowercase());
    let propia = alias.as_deref() == Some(receta.autor.to_lowercase().as_str()); // Los alias no distinguen mayúsculas

    rsx!
    {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-barra",

            p { id: "titulo-barra", class: "eyebrow", "La Barra" }

            PanelValoracion { receta_id: receta.id.clone(), propia }

            // Elegir contenido según el estado del hilo
            match &*hilo_res.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "Cargando comentarios…" }
                },
                Some(Err(causa)) if crate::local::es_sin_conexion(causa) => rsx!
                {
                    div
                    {
                        class: "estado estado-aviso",
                        role: "alert",
                        p { "Sin conexión: los comentarios no están disponibles." }
                        button { class: "enlace", r#type: "button", onclick: move |_| hilo_res.restart(), "Reintentar" }
                    }
                },
                Some(Err(_)) => rsx!
                {
                    div
                    {
                        class: "estado estado-error",
                        role: "alert",
                        p { "No pudimos traer los comentarios." }
                        button { class: "enlace", r#type: "button", onclick: move |_| hilo_res.restart(), "Reintentar" }
                    }
                },
                Some(Ok(comentarios_vec)) if comentarios_vec.is_empty() => rsx!
                {
                    p { class: "estado", "Nadie ha comentado todavía. ¿Cómo te salió?" }
                },
                Some(Ok(comentarios_vec)) => rsx!
                {
                    ul
                    {
                        class: "barra-hilo",

                        for (comentario, es_respuesta) in ordenar_hilo(comentarios_vec)
                        {
                            li
                            {
                                key: "{comentario.id}",
                                class: if es_respuesta { "barra-comentario barra-comentario--respuesta" } else { "barra-comentario" },

                                p
                                {
                                    class: "barra-meta",
                                    strong { "{comentario.autor}" }
                                    " · {comentario.fecha}"
                                }
                                if let Some(ancla) = comentario.ancla.clone()
                                {
                                    p { class: "barra-ancla-texto", "{ancla}" }
                                }
                                p { "{comentario.texto}" }
                                if let Some(propuesta) = comentario.propuesta.clone()
                                {
                                    p { class: "barra-propuesta-texto", "Propone: {propuesta}" }
                                }
                                if sesion.read().is_some()
                                {
                                    div
                                    {
                                        class: "acciones-fila",
                                        button
                                        {
                                            class: "enlace",
                                            r#type: "button",
                                            "aria-label": "Responder a {comentario.autor}",
                                            onclick:
                                            {
                                                let comentario = comentario.clone();
                                                move |_| respuesta_a.set(Some(comentario.clone()))
                                            },
                                            "Responder"
                                        }
                                        // Los comentarios propios no se reportan
                                        if !comentario.propio
                                        {
                                            Reportar { receta_id: None, comentario_id: Some(comentario.id.clone()) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }

            // Con sesión: formulario para comentar; sin sesión, invitar a entrar
            if sesion.read().is_some()
            {
                FormularioComentario
                {
                    receta: receta.clone(),
                    respuesta_a,
                    al_publicar: move |_| hilo_res.restart(),
                }
            }
            else
            {
                p { class: "mini", "Entra para comentar y valorar." }
            }

            // Con sesión y receta ajena: permitir reportarla
            if sesion.read().is_some() && !propia
            {
                Reportar { receta_id: Some(receta.id.clone()), comentario_id: None }
            }
        }
    }
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas
{
    // LIBRERÍAS

    use super::ancla_de; // Lectura del desplegable de ancla
    use super::ordenar_hilo; // Orden del hilo
    use api::foro::Ancla; // Ancla esperada
    use api::foro::Comentario; // Comentario de prueba

    // FUNCIONES

    /// Arma un comentario de prueba con su padre.
    fn comentario(id: &str, padre: Option<&str>) -> Comentario
    {
        Comentario
        {
            id: id.to_string(),
            autor: "ana".to_string(),
            texto: id.to_string(),
            fecha: String::new(),
            respuesta_a: padre.map(|valor| valor.to_string()),
            ancla: None,
            propuesta: None,
            propio: false,
        }
    }

    /// Comprueba que el desplegable se traduce al ancla correcta.
    #[test]
    fn ancla_desde_el_desplegable()
    {
        assert_eq!(ancla_de("c2"), Some(Ancla::Capa(2)));
        assert_eq!(ancla_de("p0"), Some(Ancla::Paso(0)));
        assert_eq!(ancla_de(""), None);
        assert_eq!(ancla_de("x1"), None);
    }

    /// Comprueba que las respuestas, incluso a otra respuesta, quedan bajo su raíz.
    #[test]
    fn respuestas_bajo_su_raiz()
    {
        let hilo_vec = vec![
            comentario("a", None),
            comentario("b", None),
            comentario("a1", Some("a")),
            comentario("a1x", Some("a1")),
            comentario("huerfano", Some("oculto")),
        ];
        let orden_vec: Vec<(String, bool)> = ordenar_hilo(&hilo_vec)
            .into_iter()
            .map(|(comentario, respuesta)| (comentario.id, respuesta))
            .collect();

        assert_eq!(
            orden_vec,
            vec![
                ("a".to_string(), false),
                ("a1".to_string(), true),
                ("a1x".to_string(), true),
                ("b".to_string(), false),
                ("huerfano".to_string(), false),
            ]
        );
    }
}
