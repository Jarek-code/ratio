//! Red del cliente: plazo, reintentos y mensajes de las llamadas al servidor.
//!
//! Una server function no trae plazo propio: si el servidor no contesta, la petición puede
//! quedarse esperando para siempre y la pantalla se queda en «Cargando…». Aquí cada llamada
//! se corta a los [`SEGUNDOS_ESPERA`] segundos, se repite hasta [`REINTENTOS`] veces
//! esperando [`ESPERAS`] entre intentos y, si aun así falla, devuelve un texto que la persona
//! puede entender y un aviso con su botón de reintentar.

use std::future::Future;
use std::time::Duration;

use api::fallos;
use dioxus::fullstack::RequestError;
use dioxus::prelude::*;

// CONSTANTES

/// Segundos que se espera cada intento antes de cortarlo.
pub const SEGUNDOS_ESPERA: u64 = 20;

/// Segundos que se espera a abrir la conexión (Render tarda en despertar).
pub const SEGUNDOS_CONEXION: u64 = 10;

/// Reintentos automáticos después del primer intento.
pub const REINTENTOS: usize = 2;

/// Espera entre un intento y el siguiente, en segundos: primero 2 s y después 5 s.
pub const ESPERAS: [u64; REINTENTOS] = [2, 5];

// FUNCIONES

/// Deja el cliente HTTP de Dioxus con plazo de conexión y de respuesta.
///
/// Sin esto, una petición que el servidor nunca contesta se queda abierta: el plazo propio
/// de cada llamada ([`con_reintentos`]) la corta igual, pero el cliente HTTP se encarga
/// además de las que no pasan por aquí, como la subida de fotos.
pub fn configurar() {
    let cliente = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(SEGUNDOS_CONEXION))
        .timeout(Duration::from_secs(SEGUNDOS_ESPERA + 5))
        .build();

    match cliente {
        Ok(cliente) => {
            // Si alguien ya lo había fijado, manda el primero: da igual, los dos llevan plazo
            let _ = dioxus::fullstack::GLOBAL_REQUEST_CLIENT.set(cliente);
        }
        Err(causa) => dioxus::logger::tracing::error!("no se pudo armar el cliente HTTP: {causa}"),
    }
}

/// Fallo con el que se cierra un intento que se pasó de plazo.
fn tiempo_agotado() -> ServerFnError {
    ServerFnError::Request(RequestError::Timeout(format!(
        "sin respuesta en {SEGUNDOS_ESPERA} s ({} intentos)",
        REINTENTOS + 1
    )))
}

/// Llama al servidor y repite el envío si el fallo es temporal.
///
/// Es la forma de pedir datos: lecturas que se pueden repetir sin efectos dobles.
///
/// # Errores
///
/// Devuelve el fallo del servidor tal cual si no tiene sentido reintentarlo (un dato
/// inválido, una sesión caducada), o el plazo agotado si se acabaron los intentos.
pub async fn con_reintentos<T, F, Fut>(mut llamada: F) -> Result<T, ServerFnError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ServerFnError>>,
{
    let mut intento = 0_usize; // Intentos ya gastados

    loop {
        let resultado_res =
            tokio::time::timeout(Duration::from_secs(SEGUNDOS_ESPERA), llamada()).await;

        match resultado_res {
            Ok(Ok(valor)) => return Ok(valor), // Contesto: listo
            Ok(Err(causa)) if intento < REINTENTOS && fallos::es_reintentable(&causa) => {
                dioxus::logger::tracing::warn!("fallo temporal ({causa}); se reintenta");
            }
            Ok(Err(causa)) => return Err(causa), // Fallo que no se arregla repitiendo
            Err(_) if intento >= REINTENTOS => return Err(tiempo_agotado()), // Sin respuesta
            Err(_) => dioxus::logger::tracing::warn!("sin respuesta; se reintenta"),
        }

        // Esperar antes del siguiente intento, cada vez un poco más
        let espera_s = ESPERAS[intento.min(REINTENTOS - 1)];
        tokio::time::sleep(Duration::from_secs(espera_s)).await;
        intento += 1;
    }
}

/// Llama al servidor una sola vez, con plazo pero sin reintentos.
///
/// Es la forma de enviar cambios: repetir a ciegas un envío que sí llegó podría duplicarlo.
///
/// # Errores
///
/// Devuelve el fallo del servidor, o el plazo agotado si se pasó de [`SEGUNDOS_ESPERA`].
pub async fn con_plazo<T, F>(llamada: F) -> Result<T, ServerFnError>
where
    F: Future<Output = Result<T, ServerFnError>>,
{
    match tokio::time::timeout(Duration::from_secs(SEGUNDOS_ESPERA), llamada).await {
        Ok(resultado_res) => resultado_res,
        Err(_) => Err(tiempo_agotado()),
    }
}

/// ¿El fallo es de red (sin conexión o plazo agotado) y no una respuesta del servidor?
///
/// Un error de estado (4xx, 5xx) sí es una respuesta: el servidor contestó y hay que leerlo.
pub fn es_red(causa: &ServerFnError) -> bool {
    matches!(
        causa,
        ServerFnError::Request(
            RequestError::Timeout(_) | RequestError::Connect(_) | RequestError::Request(_)
        )
    )
}

/// Texto del fallo listo para mostrar: qué pasó, qué puede hacer la persona y qué se intentó.
///
/// Los mensajes del servidor ya vienen escritos para leerse; los de red se arman aquí.
pub fn mensaje(causa: &ServerFnError) -> String {
    match causa {
        ServerFnError::Request(RequestError::Timeout(_)) => format!(
            "El servidor no respondió en {SEGUNDOS_ESPERA} s. Se intentó {} veces: revisa tu conexión y vuelve a probar.",
            REINTENTOS + 1
        ),
        ServerFnError::Request(RequestError::Connect(_)) => format!(
            "No se pudo abrir la conexión con el servidor ({} s de espera). Revisa tu red o los datos móviles.",
            SEGUNDOS_CONEXION
        ),
        ServerFnError::Request(RequestError::Status(_, codigo)) => format!(
            "El servidor respondió con un error {codigo}. Vuelve a intentarlo en un momento."
        ),
        ServerFnError::Request(_) => {
            "Sin conexión con el servidor. Revisa tu red o los datos móviles y vuelve a intentarlo."
                .to_string()
        }
        ServerFnError::ServerError { message, .. } if !message.trim().is_empty() => {
            message.clone()
        }
        ServerFnError::ServerError { .. } => {
            "El servidor no pudo completar la petición. Vuelve a intentarlo.".to_string()
        }
        ServerFnError::Deserialization(_) | ServerFnError::Serialization(_) => {
            "La respuesta del servidor llegó incompleta. Vuelve a intentarlo.".to_string()
        }
        ServerFnError::StreamError(_) => {
            "La conexión se cortó a media respuesta. Vuelve a intentarlo.".to_string()
        }
        otra => format!("No se pudo completar la petición: {otra}"),
    }
}

/// Aviso de un fallo de red o del servidor, con su botón de reintentar.
///
/// Se pinta donde iría el contenido: el mensaje dice qué pasó y, si la pantalla sabe cómo
/// repetir la petición, ofrece el botón.
#[component]
pub fn AvisoRed(
    /// El fallo tal como llegó.
    causa: ServerFnError,
    /// Clase del bloque; por defecto la del estado de error de las pantallas.
    #[props(default = None)]
    clase: Option<String>,
    /// Qué hacer para repetir; sin esto no se pinta el botón.
    #[props(default = None)]
    reintentar: Option<EventHandler<MouseEvent>>,
) -> Element {
    let es_aviso = es_red(&causa) || fallos::es_reintentable(&causa);
    let clase = clase.unwrap_or_else(|| {
        if es_aviso {
            "estado estado-aviso".to_string()
        } else {
            "estado estado-error".to_string()
        }
    });

    rsx! {
        div { class: clase, role: "alert",
            p { "{mensaje(&causa)}" }
            if let Some(reintentar) = reintentar {
                button {
                    class: "enlace",
                    r#type: "button",
                    onclick: move |evento| reintentar.call(evento),
                    "Reintentar"
                }
            }
        }
    }
}
