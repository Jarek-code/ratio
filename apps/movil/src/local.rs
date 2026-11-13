//! Repositorio local (A2) y cola de cambios sin conexión, parte de la app.
//!
//! Guarda en la carpeta privada de la app las últimas recetas abiertas, para verlas sin
//! red, y los cambios que no pudieron enviarse, para mandarlos cuando vuelva la conexión.
//! Todo se separa por cuenta: una cuenta nunca ve la copia ni los pendientes de otra, así
//! que el control de edad que aplicó el servidor sigue valiendo sin conexión.

// LIBRERÍAS

use std::fs; // Lectura y escritura de los archivos JSON
use std::io::Read; // Lectura de bytes de /dev/urandom
use std::path::PathBuf; // Rutas de los archivos locales
use std::time::Duration; // Espera entre dos intentos de envío

use api::foro::ComentarioNuevo; // Comentario que espera a ser enviado
use api::foro::ReporteNuevo; // Reporte que espera a ser enviado
use api::guardados::alternar_guardado; // Envío de un guardado pendiente
use api::guardar::guardar_receta; // Envío de una versión pendiente
use api::guardar::RecetaNueva; // Versión que espera a ser enviada
use api::receta::completa; // Ficha completa desde el servidor
use api::receta::RecetaCompleta; // Ficha que se guarda en A2
use dioxus::prelude::*; // Señales, componentes y ServerFnError
use serde::de::DeserializeOwned; // Lectura genérica de los archivos JSON
use serde::Deserialize; // Lectura de las entradas guardadas
use serde::Serialize; // Escritura de las entradas guardadas

use crate::sesion::SenalSesion; // Sesión de la que salen cuenta y token

// CONSTANTES Y VARIABLES GLOBALES

/// Archivo con las recetas vistas.
const ARCHIVO_VISTAS: &str = "vistas.json";

/// Archivo con los cambios pendientes de enviar.
const ARCHIVO_COLA: &str = "cola.json";

/// Recetas vistas que se conservan por cuenta; las más antiguas se descartan.
const MAX_VISTAS: usize = 30;

/// Segundos entre dos intentos de vaciar la cola.
const REINTENTAR_CADA: u64 = 20;

// ESTRUCTURAS

/// Receta guardada en A2 junto con la cuenta que la vio (`None` sin sesión).
#[derive(Clone, Serialize, Deserialize)]
struct RecetaVista
{
    /// Cuenta que la pidió; solo esa cuenta puede leer la copia.
    cuenta: Option<String>,
    /// Ficha tal como la devolvió el servidor.
    receta: RecetaCompleta,
}

/// Cambio que espera a que vuelva la conexión.
#[derive(Clone, Serialize, Deserialize)]
pub enum Cambio
{
    /// Guardar una versión privada desde C1 y C2.
    Version(RecetaNueva),
    /// Guardar o quitar una receta de Guardados (C5).
    Guardado
    {
        /// Receta que se guarda o se quita.
        receta_id: String,
        /// `true` para guardar, `false` para quitar.
        guardar: bool,
    },
    /// Comentar en el hilo de una receta (D3).
    Comentario(ComentarioNuevo),
    /// Valorar si la receta salió (D4).
    Valoracion
    {
        /// Receta valorada.
        receta_id: String,
        /// `true` si salió igual a la ficha.
        me_salio: bool,
    },
    /// Reportar una receta o un comentario (D7).
    Reporte(ReporteNuevo),
    /// Seguir o dejar de seguir un perfil (E3).
    Seguimiento
    {
        /// Alias del perfil.
        alias: String,
        /// `true` para seguir, `false` para dejar de seguir.
        sigue: bool,
    },
    /// Publicar o retirar una receta propia de La Barra (D1).
    Publicacion
    {
        /// Receta propia.
        receta_id: String,
        /// `true` para publicar, `false` para retirar.
        publica: bool,
    },
}

/// Entrada de la cola: un cambio con su cuenta y un identificador propio.
#[derive(Clone, Serialize, Deserialize)]
struct Pendiente
{
    /// Identifica la entrada para quitarla sin tocar las que lleguen mientras se envía.
    id: String,
    /// Cuenta dueña del cambio; solo se envía con su sesión.
    cuenta: String,
    /// Cambio por enviar.
    cambio: Cambio,
}

/// Señal con el número de cambios pendientes de la cuenta actual.
pub type SenalPendientes = Signal<usize>;

// FUNCIONES

/// Ruta de un archivo dentro de la carpeta privada de la app.
fn ruta(nombre: &str) -> Option<PathBuf>
{
    crate::sesion::carpeta_datos().map(|carpeta| carpeta.join(nombre))
}

/// Lee un valor guardado en un archivo JSON de la carpeta privada, si existe y es válido.
pub fn leer_valor<T: DeserializeOwned>(nombre: &str) -> Option<T>
{
    let texto = fs::read_to_string(ruta(nombre)?).ok()?; // Sin carpeta o sin archivo: nada guardado
    serde_json::from_str(&texto).ok() // Archivo dañado: nada guardado
}

/// Lee una lista guardada; un archivo ausente o dañado se trata como lista vacía.
fn leer<T: DeserializeOwned>(nombre: &str) -> Vec<T>
{
    leer_valor(nombre).unwrap_or_default()
}

/// Escribe un valor completo en un archivo JSON; si falla, lo anota y la app sigue en memoria.
pub fn escribir<T: Serialize + ?Sized>(nombre: &str, valor: &T)
{
    let Some(archivo) = ruta(nombre) else
    {
        return; // Sin carpeta de datos no se puede guardar
    };

    if let Some(carpeta) = archivo.parent()
    {
        let _ = fs::create_dir_all(carpeta);
    }

    let escrito = serde_json::to_string(valor)
        .ok()
        .map(|texto| fs::write(&archivo, texto).is_ok());

    if escrito != Some(true)
    {
        dioxus::logger::tracing::warn!("could not write {nombre}");
    }
}

/// Genera un UUID v4 con bytes de `/dev/urandom` (iOS, Android y macOS lo tienen).
///
/// Devuelve `None` si no se pueden leer; entonces el servidor genera el identificador.
pub fn nuevo_id() -> Option<String>
{
    let mut bytes = [0u8; 16]; // Bytes aleatorios del identificador
    fs::File::open("/dev/urandom")
        .and_then(|mut fuente| fuente.read_exact(&mut bytes))
        .ok()?;

    // Marcar la versión 4 y la variante RFC 4122
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect(); // 32 dígitos
    Some(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

/// Identificador de una entrada de la cola cuando no hay `/dev/urandom`: los nanosegundos
/// del reloj, que no se repiten dentro del mismo dispositivo.
fn respaldo_id() -> String
{
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|transcurrido| transcurrido.as_nanos())
        .unwrap_or(0); // Reloj anterior a 1970: imposible en la práctica
    format!("local-{nanos}")
}

/// Indica si el fallo se arregla reintentando: sin red, plazo agotado o servidor caído.
///
/// Con `true` la app usa la copia guardada (A2) y deja el cambio en la cola, en vez de
/// dar la petición por perdida.
pub fn es_sin_conexion(causa: &ServerFnError) -> bool
{
    crate::red::es_red(causa) || api::fallos::es_reintentable(causa)
}

/// Cuenta de la sesión actual, si la hay.
fn cuenta_de(sesion: &SenalSesion) -> Option<String>
{
    sesion.peek().as_ref().map(|guardada| guardada.perfil.usuario_id.clone())
}

/// Quita de A2 la copia de una receta para una cuenta.
fn olvidar(cuenta: &Option<String>, id: &str)
{
    let mut vistas_vec: Vec<RecetaVista> = leer(ARCHIVO_VISTAS); // Copias guardadas
    vistas_vec.retain(|vista| !(&vista.cuenta == cuenta && vista.receta.id == id));
    escribir(ARCHIVO_VISTAS, &vistas_vec);
}

/// Guarda una ficha recién traída del servidor como la más reciente de su cuenta.
fn recordar(cuenta: Option<String>, receta: &RecetaCompleta)
{
    olvidar(&cuenta, &receta.id);
    let mut vistas_vec: Vec<RecetaVista> = leer(ARCHIVO_VISTAS); // Copias sin la anterior
    vistas_vec.insert(
        0,
        RecetaVista
        {
            cuenta: cuenta.clone(),
            receta: receta.clone(),
        },
    );

    // Conservar solo las más recientes de esta cuenta
    let mut contador_cuenta = 0; // Copias de esta cuenta ya conservadas
    vistas_vec.retain(|vista|
    {
        if vista.cuenta != cuenta
        {
            return true; // Otra cuenta: no se toca
        }
        contador_cuenta += 1;
        contador_cuenta <= MAX_VISTAS // Más allá del límite se descarta
    });

    escribir(ARCHIVO_VISTAS, &vistas_vec);
}

/// Copia guardada de una receta para la cuenta, si la vio antes.
fn copia(cuenta: &Option<String>, id: &str) -> Option<RecetaCompleta>
{
    leer::<RecetaVista>(ARCHIVO_VISTAS)
        .into_iter()
        .find(|vista| &vista.cuenta == cuenta && vista.receta.id == id)
        .map(|vista| vista.receta) // Ficha tal como se vio la última vez
}

/// Últimas recetas vistas por la cuenta, de la más reciente a la más antigua.
pub fn recientes(cuenta: &Option<String>, cuantas: usize) -> Vec<RecetaCompleta>
{
    leer::<RecetaVista>(ARCHIVO_VISTAS)
        .into_iter()
        .filter(|vista| &vista.cuenta == cuenta)
        .take(cuantas)
        .map(|vista| vista.receta)
        .collect() // Fichas en el orden en que se vieron
}

/// Pide la ficha al servidor y la guarda en A2; sin conexión devuelve la copia guardada.
///
/// `cuenta` es el `usuario_id` de la sesión (`None` sin sesión) y `token` el que se reenvía.
///
/// # Errores
///
/// Devuelve el `ServerFnError` del servidor, o el de red si tampoco hay copia guardada.
pub async fn receta(
    id: String,
    token: Option<String>,
    cuenta: Option<String>,
) -> Result<Option<RecetaCompleta>, ServerFnError>
{
    match completa(id.clone(), token).await
    {
        Ok(Some(receta)) =>
        {
            // Con red: guardar la copia y devolver la ficha
            recordar(cuenta, &receta);
            Ok(Some(receta))
        }
        Ok(None) =>
        {
            // Ya no visible para esta cuenta: borrar la copia
            olvidar(&cuenta, &id);
            Ok(None)
        }
        Err(causa) if es_sin_conexion(&causa) =>
        {
            // Sin red: la copia guardada, o el error si no hay
            copia(&cuenta, &id).map(Some).ok_or(causa)
        }
        Err(causa) => Err(causa), // Error del servidor
    }
}

/// Número de cambios pendientes de una cuenta.
fn contar(cuenta: &Option<String>) -> usize
{
    leer::<Pendiente>(ARCHIVO_COLA)
        .iter()
        .filter(|pendiente| Some(&pendiente.cuenta) == cuenta.as_ref())
        .count()
}

/// Deja un cambio en la cola de la cuenta actual. Devuelve `false` si no hay sesión.
pub fn encolar(sesion: &SenalSesion, pendientes: &mut SenalPendientes, cambio: Cambio) -> bool
{
    let Some(cuenta) = cuenta_de(sesion) else
    {
        return false; // Sin sesión no hay a quién atribuir el cambio
    };

    let mut cola_vec: Vec<Pendiente> = leer(ARCHIVO_COLA); // Cola guardada
    cola_vec.push(Pendiente
    {
        id: nuevo_id().unwrap_or_else(respaldo_id),
        cuenta: cuenta.clone(),
        cambio,
    });
    escribir(ARCHIVO_COLA, &cola_vec);
    pendientes.set(contar(&Some(cuenta)));

    true // Cambio guardado en la cola
}

/// Envía un cambio pendiente al servidor.
async fn enviar(token: String, cambio: Cambio) -> Result<(), ServerFnError>
{
    match cambio
    {
        Cambio::Version(datos) => guardar_receta(token, datos).await.map(|_| ()),
        Cambio::Guardado { receta_id, guardar } =>
        {
            alternar_guardado(token, receta_id, guardar).await.map(|_| ())
        }
        Cambio::Comentario(nuevo) => api::foro::comentar(token, nuevo).await.map(|_| ()),
        Cambio::Valoracion { receta_id, me_salio } =>
        {
            api::foro::valorar(token, receta_id, me_salio).await.map(|_| ())
        }
        Cambio::Reporte(reporte) => api::foro::reportar(token, reporte).await,
        Cambio::Seguimiento { alias, sigue } => api::perfil::seguir(token, alias, sigue).await.map(|_| ()),
        Cambio::Publicacion { receta_id, publica } =>
        {
            api::foro::publicar(token, receta_id, publica).await.map(|_| ())
        }
    }
}

/// Envía en orden los cambios pendientes de la cuenta actual mientras la app siga abierta.
///
/// Se detiene al primer fallo de red y lo reintenta más tarde. Un cambio que el servidor
/// rechaza (por ejemplo, una receta que ya no existe) se descarta para no bloquear la cola.
pub async fn vaciar(sesion: SenalSesion, mut pendientes: SenalPendientes)
{
    loop
    {
        let cuenta = cuenta_de(&sesion);
        let token = crate::sesion::token(&sesion);

        if let (Some(cuenta_val), Some(token_val)) = (cuenta.clone(), token)
        {
            let propios_vec: Vec<Pendiente> = leer::<Pendiente>(ARCHIVO_COLA)
                .into_iter()
                .filter(|pendiente| pendiente.cuenta == cuenta_val)
                .collect(); // Cambios de esta cuenta, en orden

            // Enviar uno por uno hasta vaciar o perder la red
            for pendiente in propios_vec
            {
                // Cada envío lleva plazo: un servidor que no contesta no frena la cola
                let resultado_res =
                    crate::red::con_plazo(enviar(token_val.clone(), pendiente.cambio.clone())).await;

                if let Err(causa) = &resultado_res
                {
                    if es_sin_conexion(causa)
                    {
                        break; // Sin red: se reintenta en la siguiente vuelta
                    }
                    dioxus::logger::tracing::warn!("queued change rejected: {causa}");
                }

                // Quitar la entrada releyendo la cola, por si llegó otra mientras se enviaba
                let mut cola_vec: Vec<Pendiente> = leer(ARCHIVO_COLA);
                cola_vec.retain(|otra| otra.id != pendiente.id);
                escribir(ARCHIVO_COLA, &cola_vec);
            }
        }

        pendientes.set(contar(&cuenta));
        tokio::time::sleep(Duration::from_secs(REINTENTAR_CADA)).await;
    }
}

/// Aviso con los cambios que esperan conexión; no pinta nada si no hay ninguno.
#[component]
pub fn AvisoPendientes() -> Element
{
    let pendientes = use_context::<SenalPendientes>(); // Cambios por enviar
    let total_pendientes = pendientes();

    rsx!
    {
        if total_pendientes > 0
        {
            p
            {
                class: "aviso-pendientes",
                role: "status",

                if total_pendientes == 1
                {
                    "1 cambio pendiente de enviar: se enviará cuando vuelva la conexión."
                }
                else
                {
                    "{total_pendientes} cambios pendientes de enviar: se enviarán cuando vuelva la conexión."
                }
            }
        }
    }
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas
{
    // LIBRERÍAS

    use super::nuevo_id; // Generador de identificadores

    // FUNCIONES

    /// Comprueba que el identificador tiene forma de UUID v4.
    #[test]
    fn nuevo_id_es_uuid_v4()
    {
        let id_val = nuevo_id().unwrap_or_default(); // Identificador generado
        let partes_vec: Vec<&str> = id_val.split('-').collect(); // Grupos del UUID

        assert_eq!(partes_vec.iter().map(|parte| parte.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
        assert!(partes_vec[2].starts_with('4'), "version must be 4");
        assert!(matches!(partes_vec[3].chars().next(), Some('8' | '9' | 'a' | 'b')), "variant must be RFC 4122");
        assert_ne!(nuevo_id(), nuevo_id(), "ids must differ");
    }
}
