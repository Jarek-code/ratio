//! Token push del aparato: Android lo deja en un archivo y aquí se registra en el servidor.
//!
//! El servicio de FCM corre fuera del runtime de Rust (incluso con la app cerrada), así que
//! no puede llamar a una server function. Guarda el token en la carpeta privada y esta tarea,
//! mientras la app vive, lo envía a `registrar_dispositivo`. Al cerrar sesión se da de baja.

use std::path::PathBuf;
use std::time::Duration;

use crate::sesion::SenalSesion;

/// Cada cuánto se revisa si hay token o sesión nuevos, en segundos.
const REVISAR_CADA: u64 = 15;

/// Archivo donde `MainActivity.kt` deja el token de FCM.
#[cfg(target_os = "android")]
const ARCHIVO: &str = "push-token.txt";

/// Archivo donde `MainActivity.kt` deja la ruta que abrió una notificación.
#[cfg(target_os = "android")]
const ARCHIVO_RUTA: &str = "abrir-ruta.txt";

/// Token push guardado por el lado nativo, si ya existe.
///
/// Solo Android lo escribe; en iOS y escritorio no hay archivo y no se registra nada.
pub fn token_push() -> Option<String> {
    let texto = std::fs::read_to_string(archivo()?).ok()?;
    let token = texto.trim();

    (!token.is_empty()).then(|| token.to_string())
}

/// Ruta del archivo del token en la carpeta privada de la app.
fn archivo() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        crate::sesion::carpeta_datos().map(|carpeta| carpeta.join(ARCHIVO))
    }

    #[cfg(not(target_os = "android"))]
    {
        None // Sin push nativo todavía: no hay archivo que leer
    }
}

/// Ruta del archivo donde se deja el destino de una notificación tocada.
fn archivo_ruta() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        crate::sesion::carpeta_datos().map(|carpeta| carpeta.join(ARCHIVO_RUTA))
    }

    #[cfg(not(target_os = "android"))]
    {
        None
    }
}

/// Consume la ruta que dejó una notificación tocada, si hay sesión para abrirla.
///
/// Devuelve `true` una sola vez por notificación: al leerla, borra el archivo. Sin sesión no
/// consume nada, para que la ruta siga esperando a que la persona entre.
pub fn tomar_atajo(sesion: &SenalSesion) -> bool {
    if crate::sesion::token(sesion).is_none() {
        return false;
    }

    let Some(ruta) = archivo_ruta() else {
        return false;
    };
    if !ruta.exists() {
        return false;
    }

    let _ = std::fs::remove_file(&ruta);
    true
}

/// Plataforma que se declara al servidor. Solo se registra en Android.
fn plataforma() -> &'static str {
    "android"
}

/// Registra el token del aparato cuando hay sesión, y lo reintenta si falla.
///
/// Se ejecuta una sola vez al abrir la app y no termina: una sesión nueva o un token rotado
/// se registran en la siguiente vuelta. El servidor hace UPSERT, así que repetir no duplica.
pub async fn registrar(sesion: SenalSesion) {
    let mut ultimo: Option<(String, String)> = None;

    loop {
        let cuenta = crate::sesion::cuenta(&sesion);
        let token_sesion = crate::sesion::token(&sesion);
        let push_token = token_push();

        if let (Some(cuenta), Some(token_sesion), Some(push_token)) =
            (cuenta, token_sesion, push_token)
        {
            let actual = (cuenta, push_token.clone());
            if ultimo.as_ref() != Some(&actual) {
                let llamada = api::push::registrar_dispositivo(
                    token_sesion,
                    push_token,
                    plataforma().to_string(),
                );

                match crate::red::con_plazo(llamada).await {
                    Ok(()) => {
                        ultimo = Some(actual);
                        dioxus::logger::tracing::info!("dispositivo registrado para push");
                    }
                    Err(causa) => dioxus::logger::tracing::warn!(
                        "no se pudo registrar el push: {}",
                        crate::red::mensaje(&causa)
                    ),
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(REVISAR_CADA)).await;
    }
}

/// Da de baja el token del aparato al cerrar sesión (mejor esfuerzo, sin bloquear).
///
/// Debe llamarse **antes** de borrar la sesión: necesita su token para autenticarse.
pub fn olvidar(sesion: &SenalSesion) {
    let Some(token_sesion) = crate::sesion::token(sesion) else {
        return;
    };
    let Some(push_token) = token_push() else {
        return;
    };

    // En segundo plano: el cierre de sesión no espera a la red
    dioxus::prelude::spawn(async move {
        if let Err(causa) =
            crate::red::con_plazo(api::push::baja_dispositivo(token_sesion, push_token)).await
        {
            dioxus::logger::tracing::warn!(
                "no se pudo dar de baja el push: {}",
                crate::red::mensaje(&causa)
            );
        }
    });
}
