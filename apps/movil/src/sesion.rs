//! Sesión de la app: el token de Supabase y el perfil, compartidos por contexto.
//!
//! La sesión se guarda en un archivo JSON dentro de la carpeta privada de la app, para que
//! sobreviva al cerrarla, y una tarea la renueva antes de que caduque el token.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use dioxus::prelude::*;

/// Sesión iniciada: tokens de Supabase y el perfil que los acompaña.
pub type SesionApp = api::auth::SesionIniciada;

/// Señal con la sesión; `None` mientras nadie haya entrado.
pub type SenalSesion = Signal<Option<SesionApp>>;

/// Segundos antes de la caducidad en que se pide el token nuevo.
const MARGEN_RENOVAR: i64 = 300;

/// Espera máxima entre dos revisiones de la sesión, en segundos.
const REVISAR_CADA: i64 = 60;

/// Token de la sesión actual, si la hay.
pub fn token(sesion: &SenalSesion) -> Option<String> {
    sesion.read().as_ref().map(|guardada| guardada.token.clone())
}

/// Carpeta privada de la app, donde nadie más puede leer.
///
/// iOS y el escritorio la dan con `HOME`; Android no tiene `HOME`, así que se arma con el
/// nombre del paquete, que es la primera palabra de `/proc/self/cmdline`.
pub fn carpeta_datos() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        let linea = fs::read_to_string("/proc/self/cmdline").ok()?;
        let paquete = linea.split(['\0', ':']).next()?;
        Some(PathBuf::from(format!("/data/data/{paquete}/files")))
    }

    #[cfg(not(target_os = "android"))]
    {
        let casa = std::env::var_os("HOME")?;
        Some(PathBuf::from(casa).join("Library/Application Support/Ratio"))
    }
}

/// Ruta del archivo de la sesión.
fn archivo() -> Option<PathBuf> {
    carpeta_datos().map(|carpeta| carpeta.join("sesion.json"))
}

/// Lee la sesión guardada; un archivo ausente o dañado se trata como «sin sesión».
pub fn cargar() -> Option<SesionApp> {
    let texto = fs::read_to_string(archivo()?).ok()?;
    serde_json::from_str(&texto).ok()
}

/// Escribe la sesión en disco. Si falla, la sesión sigue en memoria hasta cerrar la app.
fn guardar(guardada: &SesionApp) {
    let Some(ruta) = archivo() else {
        return;
    };
    let Ok(texto) = serde_json::to_string(guardada) else {
        return;
    };

    if let Some(carpeta) = ruta.parent() {
        let _ = fs::create_dir_all(carpeta);
    }
    if fs::write(&ruta, texto).is_err() {
        dioxus::logger::tracing::warn!("could not save session");
    }
}

/// Deja la sesión iniciada en memoria y en disco.
pub fn iniciar(sesion: &mut SenalSesion, iniciada: SesionApp) {
    guardar(&iniciada);
    sesion.set(Some(iniciada));
}

/// Cierra la sesión y borra lo guardado.
pub fn salir(sesion: &mut SenalSesion) {
    if let Some(ruta) = archivo() {
        let _ = fs::remove_file(ruta);
    }
    sesion.set(None);
}

/// Segundos desde 1970 según el reloj del dispositivo.
fn ahora() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|transcurrido| transcurrido.as_secs() as i64)
        .unwrap_or(0)
}

/// Renueva el token unos minutos antes de que caduque, mientras la app siga abierta.
///
/// Revisa la sesión cada minuto como mucho, así nota también una sesión recién iniciada o
/// cerrada. Si Supabase rechaza el token de renovación, cierra la sesión; si falla la red,
/// lo vuelve a intentar en la siguiente revisión.
pub async fn mantener(mut sesion: SenalSesion) {
    loop {
        let actual = sesion.peek().clone();
        let falta = actual
            .as_ref()
            .filter(|guardada| !guardada.refresco.is_empty())
            .map(|guardada| guardada.caduca_en - MARGEN_RENOVAR - ahora());

        match (actual, falta) {
            (Some(guardada), Some(falta)) if falta <= 0 => {
                match api::auth::renovar(guardada.refresco).await {
                    Ok(Some(nueva)) => iniciar(&mut sesion, nueva),
                    Ok(None) => salir(&mut sesion),
                    Err(_) => tokio::time::sleep(Duration::from_secs(REVISAR_CADA as u64)).await,
                }
            }
            (_, falta) => {
                let espera = falta.unwrap_or(REVISAR_CADA).clamp(1, REVISAR_CADA);
                tokio::time::sleep(Duration::from_secs(espera as u64)).await;
            }
        }
    }
}
