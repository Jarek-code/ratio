//! Sesión de la app: el token de Supabase y el perfil, compartidos por contexto.
//!
//! La sesión se guarda en un archivo JSON dentro de la carpeta privada de la app, para que
//! sobreviva al cerrarla, y una tarea la renueva antes de que caduque el token.

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
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
    sesion
        .read()
        .as_ref()
        .map(|guardada| guardada.token.clone())
}

/// Alias de la sesión actual, si la hay.
pub fn alias(sesion: &SenalSesion) -> Option<String> {
    sesion
        .read()
        .as_ref()
        .map(|guardada| guardada.perfil.alias.clone())
}

/// Si la cuenta declaró mayoría de edad.
pub fn es_adulto(sesion: &SenalSesion) -> bool {
    sesion
        .read()
        .as_ref()
        .map(|guardada| guardada.perfil.mayor_de_edad)
        .unwrap_or(false)
}

/// Cambia el alias y la declaración de edad de la sesión guardada.
///
/// Se usa después de que el servidor acepte el cambio, para que la interfaz muestre el alias
/// nuevo sin volver a entrar.
pub fn renombrar(sesion: &mut SenalSesion, nuevo_alias: &str, mayor_de_edad: bool) {
    let actualizada = sesion.read().as_ref().map(|guardada| {
        let mut copia = guardada.clone();
        copia.perfil.alias = nuevo_alias.to_string();
        copia.perfil.mayor_de_edad = mayor_de_edad;
        copia
    });

    if let Some(copia) = actualizada {
        sesion.set(Some(copia.clone()));
        guardar(&copia);
    }
}

/// Cuenta (`usuario_id`) de la sesión actual, si la hay.
pub fn cuenta(sesion: &SenalSesion) -> Option<String> {
    sesion
        .read()
        .as_ref()
        .map(|guardada| guardada.perfil.usuario_id.clone())
}

/// Carpeta privada de la app, donde nadie más puede leer.
///
/// Se prueba una lista de candidatas y se queda con la primera que se deje crear: la carpeta
/// del paquete en Android —que no tiene `HOME`—, la de siempre bajo `HOME` en iOS y
/// escritorio, y la carpeta temporal como último recurso. Así una plataforma sin `HOME` o sin
/// permiso en esa ruta no pierde la sesión ni la cola de pendientes sin decir nada.
pub fn carpeta_datos() -> Option<PathBuf> {
    static CARPETA: OnceLock<Option<PathBuf>> = OnceLock::new();

    CARPETA.get_or_init(elegir_carpeta).clone()
}

/// Nombre del paquete en Android, que da la primera palabra de `/proc/self/cmdline`.
fn paquete_android() -> Option<String> {
    #[cfg(target_os = "android")]
    {
        let linea = fs::read_to_string("/proc/self/cmdline").ok()?;
        let paquete = linea.split(['\0', ':']).next()?.trim().to_string();

        return (!paquete.is_empty()).then_some(paquete);
    }

    #[cfg(not(target_os = "android"))]
    {
        None // En iOS y escritorio la carpeta sale de HOME
    }
}

/// Rutas donde puede vivir la carpeta privada, en orden de preferencia.
fn candidatas() -> Vec<PathBuf> {
    let mut lista = Vec::new();

    if let Some(paquete) = paquete_android() {
        lista.push(
            PathBuf::from("/data/data")
                .join(paquete)
                .join("files")
                .join("Ratio"),
        );
    }
    if let Some(casa) = std::env::var_os("HOME") {
        lista.push(PathBuf::from(casa).join("Library/Application Support/Ratio"));
    }
    lista.push(std::env::temp_dir().join("Ratio")); // Último recurso, sigue siendo privada

    lista
}

/// Primera candidata que se puede crear y escribir.
fn elegir_carpeta() -> Option<PathBuf> {
    for candidata in candidatas() {
        if fs::create_dir_all(&candidata).is_ok() {
            return Some(candidata);
        }
    }

    dioxus::logger::tracing::error!("no hay carpeta de datos escribible: la sesión no se guardará");

    None
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
    // Avisar al servidor de que este aparato ya no recibe push (lee el token antes de borrar)
    crate::push::olvidar(sesion);

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
                // Renovar con plazo: si el servidor no contesta, se reintenta en la vuelta siguiente
                match crate::red::con_plazo(api::auth::renovar(guardada.refresco)).await {
                    Ok(Some(nueva)) => iniciar(&mut sesion, nueva),
                    Ok(None) => salir(&mut sesion), // Supabase rechazó el token: hay que volver a entrar
                    Err(causa) => {
                        dioxus::logger::tracing::warn!(
                            "no se pudo renovar la sesión: {}",
                            crate::red::mensaje(&causa)
                        );
                        tokio::time::sleep(Duration::from_secs(REVISAR_CADA as u64)).await
                    }
                }
            }
            (_, falta) => {
                let espera = falta.unwrap_or(REVISAR_CADA).clamp(1, REVISAR_CADA);
                tokio::time::sleep(Duration::from_secs(espera as u64)).await;
            }
        }
    }
}
