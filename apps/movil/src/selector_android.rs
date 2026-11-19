//! Selector de fotos nativo de Android, parte de la app.
//!
//! Dioxus no expone el callback de resultados de actividad en Android, así que la foto no se
//! puede pedir con `startActivityForResult` desde Rust. En su lugar se lanza una actividad
//! auxiliar propia (`SelectorFotos`, en Kotlin y compilada dentro del APK) que abre el
//! selector del sistema, escribe los bytes elegidos en la carpeta privada de la app y se
//! cierra. Esta parte solo la lanza y espera el archivo, con plazo.

// LIBRERÍAS

use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use crate::i18n::Texto;
use jni::JavaVM;
use jni::objects::JObject;
use jni::objects::JValue; // Textos traducibles

// CONSTANTES Y VARIABLES GLOBALES

/// Clase de la actividad auxiliar dentro del APK; la escribe la app de Android.
const CLASE: &str = "com.ratio.app.SelectorFotos";

/// Archivo donde la actividad deja los bytes de la foto.
const ARCHIVO_BYTES: &str = "foto-pendiente";

/// Archivo donde la actividad deja el tipo MIME.
const ARCHIVO_TIPO: &str = "foto-pendiente-tipo";

/// Archivo donde la actividad deja si hubo foto o la persona canceló.
const ARCHIVO_ESTADO: &str = "foto-pendiente-estado";

/// Contenido de [`ARCHIVO_ESTADO`] cuando hay foto.
const ESTADO_OK: &str = "ok";

/// Contenido de [`ARCHIVO_ESTADO`] cuando la persona cerró el selector sin elegir.
const ESTADO_VACIO: &str = "vacio";

/// Cuánto se espera a que la persona elija la foto antes de rendirse.
const ESPERA_MAXIMA: Duration = Duration::from_secs(180);

/// Espera entre dos revisiones de la carpeta.
const REVISAR_CADA: Duration = Duration::from_millis(250);

// FUNCIONES

/// Abre la galería del sistema y devuelve los bytes y el tipo de la foto elegida.
///
/// `Ok(None)` si la persona cerró el selector sin elegir nada.
///
/// # Errores
///
/// Devuelve el motivo en texto cuando no se puede hablar con Android, cuando la actividad
/// auxiliar no está en el APK o cuando la foto no llega a tiempo.
pub async fn elegir_de_galeria() -> Result<Option<(Vec<u8>, String)>, String> {
    let carpeta = crate::sesion::carpeta_datos()
        .ok_or_else(|| crate::i18n::ahora(Texto::FotoSinCarpeta).to_string())?;
    let bytes_ruta = carpeta.join(ARCHIVO_BYTES);
    let tipo_ruta = carpeta.join(ARCHIVO_TIPO);
    let estado_ruta = carpeta.join(ARCHIVO_ESTADO);

    limpiar(&[&bytes_ruta, &tipo_ruta, &estado_ruta]);

    abrir_selector()?;

    let inicio = Instant::now();
    while inicio.elapsed() < ESPERA_MAXIMA {
        tokio::time::sleep(REVISAR_CADA).await;

        let estado = std::fs::read_to_string(&estado_ruta).ok();
        match estado.as_deref().map(str::trim) {
            // La persona cerró el selector sin elegir: no es un error
            Some(ESTADO_VACIO) => {
                limpiar(&[&bytes_ruta, &tipo_ruta, &estado_ruta]);

                return Ok(None);
            }
            Some(ESTADO_OK) => {
                let bytes = std::fs::read(&bytes_ruta)
                    .map_err(|_| crate::i18n::ahora(Texto::FotoNoLeidaAndroid).to_string())?;
                let tipo = std::fs::read_to_string(&tipo_ruta).unwrap_or_default();
                let tipo = match tipo.trim() {
                    "" => "image/jpeg".to_string(), // Tipo que la galería no declaró
                    declarado => declarado.to_string(),
                };

                limpiar(&[&bytes_ruta, &tipo_ruta, &estado_ruta]);

                return Ok(Some((bytes, tipo)));
            }
            _ => {} // Todavía no hay nada: seguir esperando
        }
    }

    limpiar(&[&bytes_ruta, &tipo_ruta, &estado_ruta]);

    Err(crate::i18n::ahora(Texto::FotoSinElegir).to_string())
}

/// Borra los archivos de una elección anterior o terminada.
fn limpiar(rutas: &[&Path]) {
    for ruta in rutas {
        let _ = std::fs::remove_file(ruta);
    }
}

/// Lanza la actividad auxiliar que abre el selector del sistema.
///
/// Se busca la clase con el cargador de la app: `find_class` no sirve desde un hilo pegado
/// por Rust, porque busca en el cargador del sistema.
fn abrir_selector() -> Result<(), String> {
    let contexto = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(contexto.vm().cast()) }
        .map_err(|causa| format!("No se pudo hablar con Android: {causa}"))?;
    let mut env = vm
        .get_env()
        .map_err(|causa| format!("No se pudo entrar al entorno de Android: {causa}"))?;
    let actividad = unsafe { JObject::from_raw(contexto.context().cast()) };

    let cargador = env
        .call_method(
            &actividad,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )
        .and_then(|valor| valor.l())
        .map_err(|causa| format!("Android no dejó leer el cargador de clases: {causa}"))?;
    let nombre = env
        .new_string(CLASE)
        .map_err(|causa| format!("No se pudo nombrar la clase del selector: {causa}"))?;
    let clase = env
        .call_method(
            &cargador,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::from(&nombre)],
        )
        .and_then(|valor| valor.l())
        .map_err(|causa| format!("No se encontró la actividad del selector ({CLASE}): {causa}"))?;
    let intencion = env
        .new_object(
            "android/content/Intent",
            "(Landroid/content/Context;Ljava/lang/Class;)V",
            &[JValue::Object(&actividad), JValue::Object(&clase)],
        )
        .map_err(|causa| format!("No se pudo preparar el selector: {causa}"))?;

    env.call_method(
        &actividad,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intencion)],
    )
    .map_err(|causa| format!("No se pudo abrir la galería: {causa}"))?;

    Ok(())
}
