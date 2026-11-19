//! La Barra en la app (D1-D4 y D7): publicar, valorar, hilo, comentario anclado y reporte,
//! parte de la app.
//!
//! La sección de la ficha muestra el recuento de «me salió / no me salió», el hilo con las
//! respuestas bajo su comentario y el formulario para comentar, anclado o no a una capa o a
//! un paso y con una foto opcional. Las escrituras pasan por la cola sin conexión: sin red
//! quedan pendientes, y los comentarios en cola se ven en el hilo como «pendiente de enviar».

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

use crate::i18n::Texto;
use crate::local::Cambio; // Cambio que se encola sin conexión
use crate::local::SenalPendientes; // Contador de cambios pendientes
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Ruta del perfil de quien comenta
use crate::sesion::SenalSesion; // Sesión con cuenta, alias y token // Textos traducibles

// FUNCIONES

/// Indica si la receta es de la cuenta que tiene la sesión abierta.
///
/// Compara la cuenta del autor; una copia guardada por una versión anterior de la app no la
/// trae, y entonces se compara el alias, que no distingue mayúsculas.
pub fn es_propia(sesion: &SenalSesion, receta: &RecetaCompleta) -> bool {
    let sesion_leida = sesion.read();
    let Some(guardada) = sesion_leida.as_ref() else {
        return false; // Sin sesión nada es propio
    };

    if receta.autor_id.is_empty() {
        guardada.perfil.alias.to_lowercase() == receta.autor.to_lowercase()
    } else {
        guardada
            .perfil
            .usuario_id
            .eq_ignore_ascii_case(&receta.autor_id)
    }
}

/// Convierte el valor del desplegable de ancla («c3», «p1» o vacío) en un ancla.
fn ancla_de(valor: &str) -> Option<Ancla> {
    let posicion = valor.get(1..)?.parse::<i16>().ok()?; // Posición tras la letra
    match valor.chars().next() {
        Some('c') => Some(Ancla::Capa(posicion)), // Capa de la receta
        Some('p') => Some(Ancla::Paso(posicion)), // Paso de la preparación
        _ => None,                                // Comentario general
    }
}

/// Ordena el hilo: cada comentario raíz seguido de todas sus respuestas, en orden de fecha.
///
/// Una respuesta a otra respuesta cuelga del mismo comentario raíz.
fn ordenar_hilo(comentarios: &[Comentario]) -> Vec<(Comentario, bool)> {
    let padres_map: HashMap<&str, Option<&str>> = comentarios
        .iter()
        .map(|comentario| (comentario.id.as_str(), comentario.respuesta_a.as_deref()))
        .collect(); // Padre directo de cada comentario

    // Subir por la cadena de respuestas hasta el comentario raíz
    let raiz_de = |id: &str| -> String {
        let mut actual = id;
        let mut contador_saltos = 0; // Corta una cadena rota o circular
        while let Some(Some(padre)) = padres_map.get(actual) {
            if !padres_map.contains_key(padre) || contador_saltos > comentarios.len() {
                break; // Padre ausente u oculto: queda como raíz
            }
            actual = padre;
            contador_saltos += 1;
        }
        actual.to_string() // Comentario raíz de la cadena
    };

    let mut hilo_vec = Vec::new(); // Comentarios en orden de lectura
    for raiz in comentarios
        .iter()
        .filter(|comentario| raiz_de(&comentario.id) == comentario.id)
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
pub fn BotonPublicar(receta_id: String, publicada: bool, al_cambiar: EventHandler<()>) -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para publicar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut enviando = use_signal(|| false); // Petición en curso
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    rsx! {
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
                        aviso.set(Some(crate::i18n::ahora(Texto::VuelveAEntrarPublicar).to_string()));
                        return; // Sin sesión no se publica
                    };
                    let id = receta_id.clone();
                    let publica = !publicada;
                    enviando.set(true);
                    aviso.set(None);

                    spawn(async move
                    {
                        match crate::red::con_plazo(api::foro::publicar(token, id.clone(), publica)).await
                        {
                            Ok(_) => al_cambiar.call(()), // Recargar la lista con el estado nuevo
                            Err(causa) if crate::red::sin_conexion(&causa)
                                && crate::local::encolar(&sesion, &mut pendientes, Cambio::Publicacion { receta_id: id, publica }) =>
                            {
                                aviso.set(Some(crate::i18n::ahora(Texto::SinConexionPublicacion).to_string()));
                            }
                            Err(causa) => aviso.set(Some(crate::i18n::con(crate::i18n::actual(), Texto::NoSePudoCambiarPublicacion, &[&crate::red::mensaje(&causa)]))),
                        }
                        enviando.set(false);
                    });
                },

                if enviando() { "{crate::i18n::ahora(Texto::UnMomento)}" } else if publicada { "{crate::i18n::ahora(Texto::RetirarDeLaBarra)}" } else { "{crate::i18n::ahora(Texto::PublicarEnLaBarra)}" }
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
/// `propia` oculta los botones: quien publicó la receta no la valora. `duracion_s` es lo que
/// tomó la preparación guiada, si se midió; se guarda junto con la valoración. Después de
/// valorar se pregunta, aparte y de forma opcional, cuánto gustó la bebida.
#[component]
pub fn PanelValoracion(
    receta_id: String,
    propia: bool,
    #[props(default)] duracion_s: Option<i32>,
) -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para leer y valorar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let id_recurso = receta_id.clone();
    let mut recurso = use_resource(move || {
        let token = crate::sesion::token(&sesion);
        let id = id_recurso.clone(); // Copia propia para los reintentos
        crate::red::con_reintentos(move || api::foro::valoraciones(id.clone(), token.clone()))
    }); // Recuento del servidor
    let mut local_sel = use_signal(|| None::<Valoraciones>); // Recuento tras valorar
    let mut valorada = use_signal(|| false); // Si se valoró en esta visita
    let mut gusto_sel = use_signal(|| None::<i16>); // Gusto elegido en esta visita
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    let actual = local_sel().or_else(|| {
        recurso
            .read()
            .as_ref()
            .and_then(|res| res.as_ref().ok().copied())
    }); // Recuento a mostrar
    let con_sesion = sesion.read().is_some();

    // Guardar el gusto y la duración de la preparación, o encolarlos sin conexión
    let id_detalle = receta_id.clone();
    let detallar = use_callback(move |gusto: Option<i16>| {
        let Some(token) = crate::sesion::token(&sesion) else {
            return; // Sin sesión no hay preparación que completar
        };
        if gusto.is_none() && duracion_s.is_none() {
            return; // Nada que guardar
        }
        let id = id_detalle.clone();

        spawn(async move {
            match crate::red::con_plazo(api::diario::detallar_preparacion(
                token,
                id.clone(),
                gusto,
                duracion_s,
            ))
            .await
            {
                Ok(()) => {}
                Err(causa)
                    if crate::red::sin_conexion(&causa)
                        && crate::local::encolar(
                            &sesion,
                            &mut pendientes,
                            Cambio::Detalle {
                                receta_id: id,
                                gusto,
                                duracion_s,
                            },
                        ) => {}
                Err(causa) => aviso.set(Some(crate::i18n::con(
                    crate::i18n::actual(),
                    Texto::NoSePudoGuardarGusto,
                    &[&crate::red::mensaje(&causa)],
                ))),
            }
        });
    });

    // Enviar una valoración, o encolarla sin conexión
    let valorar = use_callback(move |me_salio: bool| {
        let Some(token) = crate::sesion::token(&sesion) else {
            return; // Sin sesión los botones no se muestran
        };
        let id = receta_id.clone();
        aviso.set(None);

        spawn(async move {
            match crate::red::con_plazo(api::foro::valorar(token, id.clone(), me_salio)).await {
                Ok(recuento) => {
                    local_sel.set(Some(recuento));
                    valorada.set(true);
                    detallar.call(gusto_sel()); // La duración viaja en cuanto existe la preparación
                }
                Err(causa)
                    if crate::red::sin_conexion(&causa)
                        && crate::local::encolar(
                            &sesion,
                            &mut pendientes,
                            Cambio::Valoracion {
                                receta_id: id,
                                me_salio,
                            },
                        ) =>
                {
                    valorada.set(true);
                    detallar.call(gusto_sel()); // Queda en la cola detrás de la valoración
                    aviso.set(Some(
                        crate::i18n::ahora(Texto::SinConexionValoracion).to_string(),
                    ));
                }
                Err(causa) => aviso.set(Some(crate::i18n::con(
                    crate::i18n::actual(),
                    Texto::NoSePudoGuardarValoracion,
                    &[&crate::red::mensaje(&causa)],
                ))),
            }
        });
    });

    rsx! {
        div
        {
            class: "barra-valoracion",

            if let Some(recuento) = actual
            {
                p
                {
                    class: "barra-recuento",
                    role: "status",
                    "{crate::i18n::con(crate::i18n::actual(), Texto::RecuentoValoracion, &[&recuento.me_salio.to_string(), &recuento.no_me_salio.to_string()])}"
                }
            }

            // Con sesión y receta ajena: mostrar los botones para valorar
            if con_sesion && !propia
            {
                div
                {
                    class: "acciones-fila",
                    role: "group",
                    "aria-label": "{crate::i18n::ahora(Texto::TeSalioEstaReceta)}",

                    button
                    {
                        class: if actual.and_then(|recuento| recuento.mia) == Some(true) { "chip chip--activo" } else { "chip" },
                        r#type: "button",
                        "aria-pressed": if actual.and_then(|recuento| recuento.mia) == Some(true) { "true" } else { "false" },
                        onclick: move |_| valorar.call(true),
                        {crate::i18n::ahora(Texto::MeSalio)}
                    }
                    button
                    {
                        class: if actual.and_then(|recuento| recuento.mia) == Some(false) { "chip chip--activo" } else { "chip" },
                        r#type: "button",
                        "aria-pressed": if actual.and_then(|recuento| recuento.mia) == Some(false) { "true" } else { "false" },
                        onclick: move |_| valorar.call(false),
                        {crate::i18n::ahora(Texto::NoMeSalio)}
                    }
                }
            }

            // Recién valorada: preguntar aparte cuánto gustó
            if valorada()
            {
                div
                {
                    class: "barra-gusto",
                    role: "group",
                    "aria-label": crate::i18n::ahora(Texto::CuantoTeGusto),

                    p { class: "barra-recuento", "{crate::i18n::ahora(Texto::TeGustoOpcional)}" }
                    div
                    {
                        class: "acciones-fila",
                        for valor in 1..=5_i16
                        {
                            button
                            {
                                key: "{valor}",
                                class: if gusto_sel() == Some(valor) { "chip chip--activo" } else { "chip" },
                                r#type: "button",
                                "aria-pressed": if gusto_sel() == Some(valor) { "true" } else { "false" },
                                "aria-label": "{crate::i18n::con(crate::i18n::actual(), Texto::GustoDeCinco, &[&valor.to_string()])}",
                                onclick: move |_|
                                {
                                    gusto_sel.set(Some(valor));
                                    detallar.call(Some(valor));
                                },
                                "{valor}"
                            }
                        }
                    }
                    p { class: "nota", "{crate::i18n::ahora(Texto::GustoNoCambia)}" }
                }
            }

            if propia
            {
                p { class: "nota", "{crate::i18n::ahora(Texto::EsTuReceta)}" }
            }

            // El recuento no llegó: avisar sin dejar la pantalla muda
            if let Some(Err(causa)) = &*recurso.read()
            {
                AvisoRed
                {
                    causa: causa.clone(),
                    reintentar: move |_| recurso.restart(),
                }
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
) -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para comentar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut respuesta_a = respuesta_a;
    let mut texto_sel = use_signal(String::new); // Texto escrito
    let mut ancla_sel = use_signal(String::new); // Valor del desplegable de ancla
    let mut propuesta_sel = use_signal(String::new); // Cantidad propuesta escrita
    let mut enviando = use_signal(|| false); // Petición en curso
    let mut aviso = use_signal(|| None::<String>); // Resultado visible
    let mut id_sel = use_signal(|| None::<String>); // Id del comentario, fijado al adjuntar una foto
    let mut foto_sel = use_signal(|| None::<String>); // Ruta de la foto ya subida
    let mut subiendo = use_signal(|| false); // Subida de la foto en curso

    let receta_id = receta.id.clone();
    let con_ancla = !ancla_sel().is_empty();

    // Elegir una foto y subirla a la carpeta de este comentario
    let adjuntar = use_callback(move |elegida: Result<Option<(Vec<u8>, String)>, String>| {
        let (Some(token), Some(cuenta)) = (
            crate::sesion::token(&sesion),
            crate::sesion::cuenta(&sesion),
        ) else {
            aviso.set(Some(
                crate::i18n::ahora(Texto::VuelveAEntrarFoto).to_string(),
            ));
            return; // Sin sesión no se sube
        };
        let Some(id) = id_sel().or_else(crate::local::nuevo_id) else {
            aviso.set(Some(crate::i18n::ahora(Texto::FotoNoPreparada).to_string()));
            return; // Sin identificador no hay carpeta donde subirla
        };
        id_sel.set(Some(id.clone()));
        subiendo.set(true);
        aviso.set(None);

        spawn(async move {
            match crate::fotos::subir_foto_comentario(token, cuenta, id, elegida).await {
                Ok(Some(ruta)) => foto_sel.set(Some(ruta)),
                Ok(None) => {} // Selección cancelada
                Err(mensaje) => aviso.set(Some(mensaje)),
            }
            subiendo.set(false);
        });
    });

    rsx! {
        form
        {
            class: "barra-formulario",
            onsubmit: move |evento|
            {
                evento.prevent_default();
                let texto = texto_sel().trim().to_string();
                if enviando() || subiendo()
                {
                    return; // Ya hay un envío en curso o la foto sigue subiendo
                }
                if texto.is_empty()
                {
                    // Campo vacío: se dice qué falta en vez de ignorar el toque
                    aviso.set(Some({crate::i18n::ahora(Texto::EscribeTuComentario)}.to_string()));
                    return;
                }
                if crate::sesion::token(&sesion).is_none()
                {
                    aviso.set(Some(crate::i18n::ahora(Texto::VuelveAEntrarComentar).to_string()));
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
                            aviso.set(Some({crate::i18n::ahora(Texto::PropuestaInvalida)}.to_string()));
                            return; // Propuesta inválida
                        }
                    },
                };

                let nuevo = ComentarioNuevo
                {
                    id: id_sel().or_else(crate::local::nuevo_id),
                    receta_id: receta_id.clone(),
                    texto,
                    respuesta_a: respuesta_a().map(|comentario| comentario.id),
                    ancla,
                    propuesta,
                    foto: foto_sel(),
                };
                let token = crate::sesion::token(&sesion).unwrap_or_default(); // Comprobado arriba
                enviando.set(true);
                aviso.set(None);

                spawn(async move
                {
                    match crate::red::con_plazo(api::foro::comentar(token, nuevo.clone())).await
                    {
                        Ok(_) =>
                        {
                            // Publicado: limpiar el formulario y recargar el hilo
                            texto_sel.set(String::new());
                            propuesta_sel.set(String::new());
                            ancla_sel.set(String::new());
                            respuesta_a.set(None);
                            id_sel.set(None);
                            foto_sel.set(None);
                            al_publicar.call(());
                        }
                        Err(causa) if crate::red::sin_conexion(&causa)
                            && crate::local::encolar(&sesion, &mut pendientes, Cambio::Comentario(nuevo)) =>
                        {
                            texto_sel.set(String::new());
                            propuesta_sel.set(String::new());
                            ancla_sel.set(String::new());
                            respuesta_a.set(None);
                            id_sel.set(None);
                            foto_sel.set(None);
                            aviso.set(Some(crate::i18n::ahora(Texto::SinConexionComentario).to_string()));
                        }
                        Err(causa) => aviso.set(Some(crate::i18n::con(crate::i18n::actual(), Texto::NoSePudoComentar, &[&crate::red::mensaje(&causa)]))),
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
                    button { class: "enlace", r#type: "button", onclick: move |_| respuesta_a.set(None), "{crate::i18n::ahora(Texto::Cancelar)}" }
                }
            }

            label { class: "solo-lector", r#for: "comentario-texto", "{crate::i18n::ahora(Texto::TuComentario)}" }
            textarea
            {
                id: "comentario-texto",
                class: "barra-texto",
                rows: "3",
                maxlength: "2000",
                placeholder: "{crate::i18n::ahora(Texto::QueCambiarias)}",
                value: "{texto_sel}",
                oninput: move |evento| texto_sel.set(evento.value()),
            }

            div
            {
                class: "barra-ancla",

                label { r#for: "comentario-ancla", "{crate::i18n::ahora(Texto::Sobre)}" }
                select
                {
                    id: "comentario-ancla",
                    class: "barra-select",
                    value: "{ancla_sel}",
                    onchange: move |evento| ancla_sel.set(evento.value()),

                    option { value: "", "{crate::i18n::ahora(Texto::TodaLaReceta)}" }
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
                    label { r#for: "comentario-propuesta", "{crate::i18n::ahora(Texto::PropuestaOpcional)}" }
                    input
                    {
                        id: "comentario-propuesta",
                        class: "barra-propuesta",
                        r#type: "text",
                        inputmode: "decimal",
                        placeholder: if ancla_sel().starts_with('p') { crate::i18n::ahora(Texto::SegundosPlaceholder) } else { crate::i18n::ahora(Texto::CantidadPlaceholder) },
                        value: "{propuesta_sel}",
                        oninput: move |evento| propuesta_sel.set(evento.value()),
                    }
                }
            }

            // Foto opcional: se sube antes de publicar y queda en la carpeta del comentario
            if let Some(motivo) = crate::fotos::aviso_sin_fotos()
            {
                p { class: "nota", "{motivo}" }
            }
            if crate::fotos::puede_adjuntar()
            {
                if let Some(url) = foto_sel().as_deref().and_then(crate::fotos::url_publica)
                {
                    div
                    {
                        class: "barra-adjunto",
                        img { class: "foto-comentario", src: "{url}", alt: "{crate::i18n::ahora(Texto::FotoQueVasAAjuntar)}" }
                        button { class: "enlace", r#type: "button", onclick: move |_| foto_sel.set(None), "{crate::i18n::ahora(Texto::QuitarFoto)}" }
                    }
                }
                else
                {
                    crate::fotos::BotonFoto
                    {
                        id: "comentario-foto",
                        ocupado: subiendo(),
                        etiqueta: crate::i18n::ahora(Texto::AdjuntarFoto),
                        al_elegir: move |elegida| adjuntar.call(elegida),
                    }
                }
            }

            button
            {
                class: "primario",
                r#type: "submit",
                disabled: enviando() || subiendo() || texto_sel().trim().is_empty(),
                if enviando() { {crate::i18n::ahora(Texto::Publicando)} } else if respuesta_a().is_some() { {crate::i18n::ahora(Texto::Responder)} } else { {crate::i18n::ahora(Texto::Comentar)} }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "status", "{texto}" }
            }
        }
    }
}

/// Enlace «Reportar» que despliega el formulario de reporte de un comentario (D7).
///
/// Se pasa `receta_id` para reportar la receta o `comentario_id` para reportar un comentario.
#[component]
fn Reportar(receta_id: Option<String>, comentario_id: Option<String>) -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para reportar
    let mut pendientes = use_context::<SenalPendientes>(); // Cola sin conexión
    let mut abierto = use_signal(|| false); // Si el formulario está desplegado
    let mut motivo_sel = use_signal(|| Motivo::Spam); // Causa elegida
    let mut detalle_sel = use_signal(String::new); // Explicación escrita
    let mut aviso = use_signal(|| None::<String>); // Resultado visible
    let que = if comentario_id.is_some() {
        "comentario"
    } else {
        "receta"
    }; // Qué se reporta
    let sufijo = comentario_id
        .clone()
        .or(receta_id.clone())
        .unwrap_or_default(); // Ids únicos en la página

    // Ya enviado: solo el aviso
    if let Some(texto) = aviso() {
        return rsx! { p { class: "barra-aviso", role: "status", "{texto}" } };
    }

    rsx! {
        if !abierto()
        {
            button
            {
                class: "enlace",
                r#type: "button",
                "aria-label": "{crate::i18n::con(crate::i18n::actual(), Texto::ReportarEste, &[&que])}",
                onclick: move |_| abierto.set(true),
                {crate::i18n::ahora(Texto::Reportar)}
            }
        }
        else
        {
            form
            {
                class: "barra-formulario",
                "aria-label": "{crate::i18n::con(crate::i18n::actual(), Texto::ReportarEste, &[&que])}",
                onsubmit: move |evento|
                {
                    evento.prevent_default();
                    let Some(token) = crate::sesion::token(&sesion) else
                    {
                        aviso.set(Some(crate::i18n::ahora(Texto::VuelveAEntrarReportar).to_string()));
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
                        match crate::red::con_plazo(api::foro::reportar(token, reporte.clone())).await
                        {
                            Ok(true) => aviso.set(Some({crate::i18n::ahora(Texto::GraciasModeracion)}.to_string())),
                            Ok(false) => aviso.set(Some({crate::i18n::ahora(Texto::YaHabiasReportado)}.to_string())),
                            Err(causa) if crate::red::sin_conexion(&causa)
                                && crate::local::encolar(&sesion, &mut pendientes, Cambio::Reporte(reporte)) =>
                            {
                                aviso.set(Some(crate::i18n::ahora(Texto::SinConexionReporte).to_string()));
                            }
                            Err(causa) => aviso.set(Some(crate::i18n::con(crate::i18n::actual(), Texto::NoSePudoReportar, &[&crate::red::mensaje(&causa)]))),
                        }
                    });
                },

                label { r#for: "reporte-motivo-{sufijo}", "{crate::i18n::ahora(Texto::Motivo)}" }
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
                        option { key: "{motivo.clave()}", value: "{motivo.clave()}", selected: motivo == motivo_sel(), "{crate::api_textos::motivo(motivo)}" }
                    }
                }

                label { class: "solo-lector", r#for: "reporte-detalle-{sufijo}", "{crate::i18n::ahora(Texto::Detalle)}" }
                textarea
                {
                    id: "reporte-detalle-{sufijo}",
                    class: "barra-texto",
                    rows: "2",
                    maxlength: "2000",
                    placeholder: crate::i18n::ahora(Texto::CuentanosQuePasaOpcional),
                    value: "{detalle_sel}",
                    oninput: move |evento| detalle_sel.set(evento.value()),
                }

                div
                {
                    class: "acciones-fila",
                    button { class: "primario", r#type: "submit", "{crate::i18n::ahora(Texto::EnviarReporte)}" }
                    button { class: "chip", r#type: "button", onclick: move |_| abierto.set(false), "{crate::i18n::ahora(Texto::Cancelar)}" }
                }
            }
        }
    }
}

/// Un comentario del hilo con su autor, su ancla, su foto y sus acciones.
///
/// `al_responder` recibe el comentario cuando la persona toca «Responder».
#[component]
fn TarjetaComentario(
    comentario: Comentario,
    es_respuesta: bool,
    al_responder: EventHandler<Comentario>,
) -> Element {
    let sesion = use_context::<SenalSesion>(); // Con sesión se puede responder y reportar
    let foto_url = comentario
        .foto
        .as_deref()
        .and_then(crate::fotos::url_publica); // Foto adjunta, si se puede mostrar

    rsx! {
        li
        {
            class: if es_respuesta { "barra-comentario barra-comentario--respuesta" } else { "barra-comentario" },

            p
            {
                class: "barra-meta",
                Link
                {
                    to: Rutas::PerfilPublico { alias: comentario.autor.clone() },
                    class: "barra-autor",
                    "aria-label": "{crate::i18n::con(crate::i18n::actual(), Texto::VerPerfilDe, &[&comentario.autor])}",
                    "{comentario.autor}"
                }
                " · {crate::i18n::fecha_iso(&comentario.fecha)}"
            }
            if let Some(ancla) = crate::api_textos::ancla(&comentario.capa, comentario.paso, &comentario.paso_titulo)
            {
                p { class: "barra-ancla-texto", "{ancla}" }
            }
            p { "{comentario.texto}" }
            if let Some(propuesta) = comentario.propuesta
            {
                p { class: "barra-propuesta-texto", "{crate::i18n::con(crate::i18n::actual(), Texto::Propone, &[&crate::api_textos::propuesta(propuesta, comentario.propuesta_unidad.as_deref())])}" }
            }
            if let Some(url) = foto_url
            {
                img { class: "foto-comentario", src: "{url}", alt: "{crate::i18n::con(crate::i18n::actual(), Texto::FotoDe, &[&comentario.autor])}", loading: "lazy" }
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
                            move |_| al_responder.call(comentario.clone())
                        },
                        {crate::i18n::ahora(Texto::Responder)}
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

/// Resumen de la conversación para la ficha: valoración, últimos comentarios y el acceso al
/// hilo completo (D2 y D4).
///
/// La ficha no repite el formulario ni la lista entera: se comenta en `Rutas::Hilo`, que es la
/// única pantalla donde vive la conversación.
#[component]
pub fn ResumenHilo(receta: RecetaCompleta) -> Element {
    let sesion = use_context::<SenalSesion>(); // Token para leer los comentarios
    let propia = es_propia(&sesion, &receta); // Quien publicó no se valora a sí mismo
    let id_recurso = receta.id.clone();
    let mut recurso = use_resource(use_reactive!(|(id_recurso,)| {
        let token = crate::sesion::token(&sesion);
        crate::red::con_reintentos(move || api::foro::hilo(id_recurso.clone(), token.clone()))
    })); // Comentarios del hilo

    rsx! {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-barra",

            p { id: "titulo-barra", class: "eyebrow", "{crate::i18n::ahora(Texto::LaBarra)}" }

            PanelValoracion { receta_id: receta.id.clone(), propia }

            // Elegir contenido según el estado de la petición
            match &*recurso.read()
            {
                None => rsx! { p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoComentarios)}" } },
                Some(Err(causa)) => rsx!
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso.restart(),
                    }
                },
                Some(Ok(comentarios_vec)) if comentarios_vec.is_empty() => rsx!
                {
                    p { class: "estado", "{crate::i18n::ahora(Texto::SinComentariosAun)}" }
                },
                Some(Ok(comentarios_vec)) =>
                {
                    let total = comentarios_vec.len();
                    let ultimos_vec: Vec<Comentario> =
                        comentarios_vec.iter().rev().take(2).rev().cloned().collect();

                    rsx!
                    {
                        ul
                        {
                            class: "barra-hilo",

                            for comentario in ultimos_vec
                            {
                                li
                                {
                                    key: "{comentario.id}",
                                    class: "barra-comentario",

                                    p { class: "barra-meta", "{comentario.autor} · {crate::i18n::fecha_iso(&comentario.fecha)}" }
                                    p { "{comentario.texto}" }
                                }
                            }
                        }

                        Link
                        {
                            to: Rutas::Hilo { id: receta.id.clone() },
                            class: "primario",

                            if total == 1 { "{crate::i18n::ahora(Texto::VerComentarioYResponder)}" } else { "{crate::i18n::con(crate::i18n::actual(), Texto::VerLosComentarios, &[&total.to_string()])}" }
                        }
                    }
                },
            }
        }
    }
}

/// Hilo completo de una receta: valoración, comentarios, formulario y reporte (D2, D3, D4 y D7).
///
/// Vive en la pantalla del hilo (`Rutas::Hilo`). La receta se reporta con el modal de la ficha
/// (`reportar.rs`). Los comentarios que esperan en la cola sin conexión se pintan al final,
/// marcados como pendientes, y el hilo se recarga cuando la cola los envía.
#[component]
pub fn HiloReceta(receta: RecetaCompleta) -> Element {
    let sesion = use_context::<SenalSesion>(); // Sesión para saber si es propia y si comenta
    let pendientes = use_context::<SenalPendientes>(); // Cambios que esperan conexión
    let id_recurso = receta.id.clone();
    let mut hilo_res = use_resource(move || {
        let token = crate::sesion::token(&sesion);
        let id = id_recurso.clone(); // Copia propia para los reintentos
        crate::red::con_reintentos(move || api::foro::hilo(id.clone(), token.clone()))
    }); // Hilo del servidor
    let mut respuesta_a = use_signal(|| None::<Comentario>); // Comentario al que se responde
    let mut cola_previa = use_signal(|| *pendientes.peek()); // Tamaño de la cola en el pintado anterior

    // Cuando la cola envía algo, recargar el hilo para ver el comentario ya publicado
    use_effect(move || {
        let cola_actual = pendientes();
        if cola_actual < *cola_previa.peek() {
            hilo_res.restart();
        }
        cola_previa.set(cola_actual);
    });

    let propia = es_propia(&sesion, &receta);
    let alias_propio = sesion
        .read()
        .as_ref()
        .map(|guardada| guardada.perfil.alias.clone())
        .unwrap_or_default();
    let en_cola_vec = if pendientes() > 0 {
        crate::local::comentarios_pendientes(&sesion, &receta.id)
    } else {
        Vec::new()
    }; // Comentarios sin enviar

    rsx! {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-barra",

            p { id: "titulo-barra", class: "eyebrow", "{crate::i18n::ahora(Texto::LaBarra)}" }

            PanelValoracion { receta_id: receta.id.clone(), propia }

            // Elegir contenido según el estado del hilo
            match &*hilo_res.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoComentarios)}" }
                },
                Some(Err(causa)) => rsx!
                {
                    // Fallo de red o del servidor: explicar qué pasó y ofrecer reintentar
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| hilo_res.restart(),
                    }
                },
                Some(Ok(comentarios_vec)) if comentarios_vec.is_empty() && en_cola_vec.is_empty() => rsx!
                {
                    p { class: "estado", "{crate::i18n::ahora(Texto::SinComentariosAun)}" }
                },
                Some(Ok(comentarios_vec)) => rsx!
                {
                    ul
                    {
                        class: "barra-hilo",

                        for (comentario, es_respuesta) in ordenar_hilo(comentarios_vec)
                        {
                            TarjetaComentario
                            {
                                key: "{comentario.id}",
                                comentario,
                                es_respuesta,
                                al_responder: move |comentario| respuesta_a.set(Some(comentario)),
                            }
                        }
                    }
                },
            }

            // Comentarios en la cola sin conexión: visibles, pero marcados como pendientes
            if !en_cola_vec.is_empty()
            {
                ul
                {
                    class: "barra-hilo",
                    "aria-label": crate::i18n::ahora(Texto::ComentariosPendientes),

                    for (indice, nuevo) in en_cola_vec.iter().enumerate()
                    {
                        li
                        {
                            key: "pendiente-{indice}",
                            class: if nuevo.respuesta_a.is_some() { "barra-comentario barra-comentario--respuesta barra-comentario--pendiente" } else { "barra-comentario barra-comentario--pendiente" },

                            p
                            {
                                class: "barra-meta",
                                strong { "{alias_propio}" }
                                "{crate::i18n::ahora(Texto::PendienteDeEnviar)}"
                            }
                            p { "{nuevo.texto}" }
                        }
                    }
                }
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
                p { class: "nota", "{crate::i18n::ahora(Texto::EntraParaComentar)}" }
            }

        }
    }
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas {
    // LIBRERÍAS

    use super::ancla_de; // Lectura del desplegable de ancla
    use super::ordenar_hilo; // Orden del hilo
    use api::foro::Ancla; // Ancla esperada
    use api::foro::Comentario; // Comentario de prueba

    // FUNCIONES

    /// Arma un comentario de prueba con su padre.
    fn comentario(id: &str, padre: Option<&str>) -> Comentario {
        Comentario {
            id: id.to_string(),
            autor: "ana".to_string(),
            texto: id.to_string(),
            fecha: String::new(),
            respuesta_a: padre.map(|valor| valor.to_string()),
            capa: None,
            paso: None,
            paso_titulo: None,
            propuesta: None,
            propuesta_unidad: None,
            propio: false,
            foto: None,
        }
    }

    /// Comprueba que el desplegable se traduce al ancla correcta.
    #[test]
    fn ancla_desde_el_desplegable() {
        assert_eq!(ancla_de("c2"), Some(Ancla::Capa(2)));
        assert_eq!(ancla_de("p0"), Some(Ancla::Paso(0)));
        assert_eq!(ancla_de(""), None);
        assert_eq!(ancla_de("x1"), None);
    }

    /// Comprueba que las respuestas, incluso a otra respuesta, quedan bajo su raíz.
    #[test]
    fn respuestas_bajo_su_raiz() {
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
