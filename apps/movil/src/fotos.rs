//! Fotos de una receta (D1) y de un comentario (D3): galería y subida directa a Supabase
//! Storage, parte de la app.
//!
//! La foto viaja de la app al bucket `fotos-recetas` con el token de la sesión; la política
//! del bucket solo deja subir a la carpeta `{cuenta}/…`. Después el servidor registra la ruta
//! en la receta o en el comentario. Las fotos se ven por la URL pública de la CDN de Storage.

// LIBRERÍAS

use api::receta::RecetaCompleta; // Receta con sus rutas de fotos
use dioxus::prelude::*; // Componentes, señales y FileData

use crate::i18n::Texto;
use crate::sesion::SenalSesion; // Sesión con cuenta y token // Textos traducibles

// CONSTANTES Y VARIABLES GLOBALES

/// Dirección del proyecto de Supabase; se fija al compilar con `RATIO_SUPABASE_URL`.
const SUPABASE_URL: Option<&str> = option_env!("RATIO_SUPABASE_URL");

/// Clave pública (anon) de Supabase; se fija al compilar con `RATIO_SUPABASE_ANON_KEY`.
const SUPABASE_ANON: Option<&str> = option_env!("RATIO_SUPABASE_ANON_KEY");

/// Bucket de Storage con las fotos de las recetas.
const BUCKET: &str = "fotos-recetas";

/// Tamaño máximo de una foto, igual que el límite del bucket (5 MB).
const BYTES_MAXIMOS: u64 = 5 * 1024 * 1024;

/// Segundos que se espera a que la foto termine de subir antes de cortar el envío.
const SUBE_SEGUNDOS: u64 = 45;

/// Si la plataforma tiene con qué elegir una foto.
///
/// Dioxus 0.7 intercepta `<input type="file">` y en móvil no abre nada, así que iOS usa su
/// galería nativa (`selector_ios`) y Android una actividad auxiliar propia
/// (`selector_android`). En escritorio se usa el selector de archivos.
const HAY_SELECTOR: bool = true;

/// Si se elige con una pantalla del sistema en vez de un `<input type="file">`.
const GALERIA_NATIVA: bool = cfg!(any(target_os = "ios", target_os = "android"));

// ESTADO

/// Datos de Storage que manda el servidor cuando la app no los lleva compilados.
///
/// Así las fotos funcionan en cualquier compilación: el servidor tiene `SUPABASE_URL` y la
/// clave pública en su entorno, y la app se los pide una vez al arrancar.
static DESDE_SERVIDOR: std::sync::RwLock<Option<(String, String)>> = std::sync::RwLock::new(None);

// FUNCIONES

/// Pide al servidor los datos de Storage y los recuerda para el resto de la sesión.
pub async fn cargar_configuracion() {
    if base().is_some() {
        return; // Ya vienen compilados
    }

    let datos = match crate::red::con_reintentos(api::foro::datos_storage).await {
        Ok(datos) => datos,
        Err(causa) => {
            dioxus::logger::tracing::warn!(
                "no se pudieron pedir los datos de Storage: {}",
                crate::red::mensaje(&causa)
            );
            return;
        }
    };

    let Some(datos) = datos else {
        return; // El servidor no tiene Supabase configurado
    };

    if let Ok(mut guardado) = DESDE_SERVIDOR.write() {
        *guardado = Some((datos.url, datos.anon));
    }
}

/// Dirección del proyecto sin la barra final, compilada o pedida al servidor.
fn base_url() -> Option<String> {
    if let Some(url) = SUPABASE_URL
        .map(|url| url.trim_end_matches('/'))
        .filter(|url| !url.is_empty())
    {
        return Some(url.to_string());
    }

    DESDE_SERVIDOR
        .read()
        .ok()?
        .as_ref()
        .map(|(url, _)| url.trim_end_matches('/').to_string())
}

/// Clave pública del proyecto, compilada o pedida al servidor.
fn clave() -> Option<String> {
    if let Some(anon) = SUPABASE_ANON.filter(|anon| !anon.is_empty()) {
        return Some(anon.to_string());
    }

    DESDE_SERVIDOR
        .read()
        .ok()?
        .as_ref()
        .map(|(_, anon)| anon.clone())
}

/// Dirección del proyecto sin la barra final, si hay con qué.
fn base() -> Option<String> {
    base_url()
}

/// Si la app puede adjuntar fotos: tiene Storage configurado y cómo elegir una imagen.
pub fn puede_adjuntar() -> bool {
    HAY_SELECTOR && base().is_some()
}

/// Motivo por el que esta compilación no puede adjuntar fotos, si es el caso.
pub fn aviso_sin_fotos() -> Option<&'static str> {
    if base().is_none() {
        Some(crate::i18n::ahora(Texto::FotoSinStorage))
    } else {
        None
    }
}

/// URL pública de una foto en la CDN de Storage.
pub fn url_publica(ruta: &str) -> Option<String> {
    base().map(|url| format!("{url}/storage/v1/object/public/{BUCKET}/{ruta}"))
}

/// Extensión del archivo según su tipo; `None` si no es JPEG, PNG ni WebP.
fn extension(tipo: &str) -> Option<&'static str> {
    match tipo {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None, // Tipo que el bucket rechaza
    }
}

/// Sube los bytes de una foto a `{carpeta}/{nombre nuevo}.{ext}` del bucket; devuelve la ruta.
///
/// `carpeta` empieza por la cuenta y `tipo` es el tipo MIME de la imagen. Devuelve un mensaje
/// en español listo para mostrar.
async fn subir_bytes(
    token: &str,
    carpeta: &str,
    bytes: Vec<u8>,
    tipo: &str,
) -> Result<String, String> {
    let (Some(url), Some(anon)) = (base_url(), clave()) else {
        return Err(crate::i18n::ahora(Texto::FotoAppSinConfigurar).to_string());
    };
    let Some(ext) = extension(tipo) else {
        return Err({ crate::i18n::ahora(Texto::FotoTipoInvalido) }.to_string());
    };
    if bytes.len() as u64 > BYTES_MAXIMOS {
        return Err({ crate::i18n::ahora(Texto::FotoPesaDemasiado) }.to_string());
    }

    let nombre = crate::local::nuevo_id()
        .ok_or_else(|| crate::i18n::ahora(Texto::FotoSinNombre).to_string())?;
    let ruta = format!("{carpeta}/{nombre}.{ext}"); // Carpeta propia de la cuenta

    // Subir directo a Storage con el token de la sesión
    let respuesta = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(
            crate::red::SEGUNDOS_CONEXION,
        ))
        .timeout(std::time::Duration::from_secs(SUBE_SEGUNDOS))
        .build()
        .map_err(|_| crate::i18n::ahora(Texto::FotoSinPreparar).to_string())?
        .post(format!("{url}/storage/v1/object/{BUCKET}/{ruta}"))
        .bearer_auth(token)
        .header("apikey", anon)
        .header("content-type", tipo)
        .body(bytes)
        .send()
        .await
        .map_err(|causa| {
            if causa.is_timeout() {
                crate::i18n::con(
                    crate::i18n::actual(),
                    Texto::FotoNoTermino,
                    &[&SUBE_SEGUNDOS.to_string()],
                )
            } else {
                { crate::i18n::ahora(Texto::FotoSinConexion) }.to_string()
            }
        })?;
    if !respuesta.status().is_success() {
        return Err({ crate::i18n::ahora(Texto::FotoRechazada) }.to_string());
    }

    Ok(ruta) // Ruta de la foto dentro del bucket
}

/// Deja elegir una foto y la sube a la carpeta de un comentario; devuelve su ruta.
///
/// `archivo` es el archivo del selector de escritorio; en iOS llega vacío y se abre la
/// galería nativa. Devuelve `None` si la persona canceló.
pub async fn subir_foto_comentario(
    token: String,
    cuenta: String,
    comentario_id: String,
    archivo: Option<dioxus::html::FileData>,
) -> Result<Option<String>, String> {
    match elegir(archivo).await? {
        Some((bytes, tipo)) => subir_bytes(
            &token,
            &format!("{cuenta}/comentarios/{comentario_id}"),
            bytes,
            &tipo,
        )
        .await
        .map(Some), // Foto elegida: subirla
        None => Ok(None), // Selección cancelada
    }
}

/// Sube una foto a la carpeta de una receta y registra su ruta; devuelve las rutas de la receta.
async fn subir_a_receta(
    token: String,
    cuenta: String,
    receta_id: String,
    bytes: Vec<u8>,
    tipo: &str,
) -> Result<Vec<String>, String> {
    let ruta = subir_bytes(&token, &format!("{cuenta}/{receta_id}"), bytes, tipo).await?;

    // Registrar la ruta en la receta
    crate::red::con_plazo(api::foro::agregar_foto(token, receta_id, ruta))
        .await
        .map_err(|causa| {
            crate::i18n::con(
                crate::i18n::actual(),
                Texto::FotoNoAgregada,
                &[&crate::red::mensaje(&causa)],
            )
        })
}

/// Obtiene la foto que elige la persona: la galería nativa en iOS, la actividad auxiliar en
/// Android y el selector de archivos en escritorio. Devuelve los bytes y su tipo MIME, o
/// `None` si canceló.
#[cfg(target_os = "ios")]
async fn elegir(
    _archivo: Option<dioxus::html::FileData>,
) -> Result<Option<(Vec<u8>, String)>, String> {
    let foto = crate::selector_ios::elegir_de_galeria().await?;
    Ok(foto.map(|bytes| (bytes, "image/jpeg".to_string()))) // La galería entrega JPEG
}

/// Obtiene la foto que elige la persona con la actividad auxiliar de Android.
#[cfg(target_os = "android")]
async fn elegir(
    _archivo: Option<dioxus::html::FileData>,
) -> Result<Option<(Vec<u8>, String)>, String> {
    crate::selector_android::elegir_de_galeria().await
}

/// Obtiene la foto que elige la persona con el selector de archivos del escritorio.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
async fn elegir(
    archivo: Option<dioxus::html::FileData>,
) -> Result<Option<(Vec<u8>, String)>, String> {
    let Some(archivo) = archivo else {
        return Ok(None); // Selección cancelada
    };
    if archivo.size() > BYTES_MAXIMOS {
        return Err({ crate::i18n::ahora(Texto::FotoPesaDemasiado) }.to_string());
    }
    let tipo = archivo.content_type().unwrap_or_default();
    let bytes = archivo
        .read_bytes()
        .await
        .map_err(|_| { crate::i18n::ahora(Texto::FotoNoLeida) }.to_string())?;

    Ok(Some((bytes.to_vec(), tipo))) // Bytes y tipo del archivo elegido
}

/// Control para elegir una foto: la galería nativa en iOS, el selector de archivos en escritorio.
///
/// `id` distingue el control dentro de la página, `ocupado` lo desactiva mientras se sube y
/// `al_elegir` recibe el archivo elegido (`None` en iOS, donde la galería se abre después).
#[component]
pub fn BotonFoto(
    id: String,
    ocupado: bool,
    etiqueta: String,
    al_elegir: EventHandler<Option<dioxus::html::FileData>>,
) -> Element {
    rsx! {
        // iOS: botón que abre la galería nativa; escritorio: selector de archivos
        if GALERIA_NATIVA
        {
            button
            {
                class: "chip foto-subir",
                r#type: "button",
                disabled: ocupado,
                onclick: move |_| al_elegir.call(None),
                if ocupado { "{crate::i18n::ahora(Texto::UnMomento)}" } else { "{etiqueta}" }
            }
        }
        else
        {
            label
            {
                class: "chip chip--enlace foto-subir",
                r#for: "{id}",
                if ocupado { {crate::i18n::ahora(Texto::SubiendoFoto)} } else { "{etiqueta}" }
            }
            input
            {
                id: "{id}",
                class: "solo-lector",
                r#type: "file",
                accept: "image/jpeg,image/png,image/webp",
                disabled: ocupado,
                onchange: move |evento: FormEvent|
                {
                    if let Some(archivo) = evento.files().into_iter().next()
                    {
                        al_elegir.call(Some(archivo));
                    }
                },
            }
        }
    }
}

/// Galería de fotos de la receta y, para quien la creó, el botón para agregar una (D1).
#[component]
pub fn FotosReceta(receta: RecetaCompleta) -> Element {
    let sesion = use_context::<SenalSesion>(); // Cuenta y token para subir
    let mut fotos_sel = use_signal(|| receta.fotos.clone()); // Rutas que se muestran
    let mut subiendo = use_signal(|| false); // Subida en curso
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    let propia = crate::foro::es_propia(&sesion, &receta);
    let aviso_plataforma = aviso_sin_fotos(); // Por qué no se puede subir aquí
    let puede_subir = puede_adjuntar() && propia && fotos_sel().len() < api::foro::FOTOS_MAXIMAS;
    let receta_id = receta.id.clone();
    let nombre = receta.receta.nombre.clone();

    // Elegir la foto y subirla; en iOS el archivo llega vacío y se abre la galería nativa
    let subir_foto = use_callback(move |archivo: Option<dioxus::html::FileData>| {
        let (Some(token), Some(cuenta)) = (
            crate::sesion::token(&sesion),
            crate::sesion::cuenta(&sesion),
        ) else {
            aviso.set(Some(
                { crate::i18n::ahora(Texto::FotoSinSesion) }.to_string(),
            ));
            return; // Sin sesión no se sube
        };
        let id = receta_id.clone();
        subiendo.set(true);
        aviso.set(None);

        spawn(async move {
            let resultado = match elegir(archivo).await {
                Ok(Some((bytes, tipo))) => subir_a_receta(token, cuenta, id, bytes, &tipo)
                    .await
                    .map(Some), // Foto elegida: subirla
                Ok(None) => Ok(None),         // Selección cancelada
                Err(mensaje) => Err(mensaje), // No se pudo elegir
            };
            match resultado {
                Ok(Some(rutas)) => fotos_sel.set(rutas),
                Ok(None) => {}
                Err(mensaje) => aviso.set(Some(mensaje)),
            }
            subiendo.set(false);
        });
    });

    // Sin Storage configurado, o sin fotos ni permiso para subir: no ocupar espacio
    if base().is_none() || (fotos_sel().is_empty() && !puede_subir) {
        // En Android el motivo se dice en la ficha; si no hay nada que explicar, no se pinta
        return match aviso_plataforma {
            Some(motivo) if propia => rsx! { p { class: "nota", "{motivo}" } },
            _ => rsx! {},
        };
    }

    rsx! {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-fotos",

            p { id: "titulo-fotos", class: "eyebrow", "{crate::i18n::ahora(Texto::Fotos)}" }

            div
            {
                class: "fotos-galeria",
                for (indice, ruta) in fotos_sel().iter().enumerate()
                {
                    if let Some(url) = url_publica(ruta)
                    {
                        img { key: "{ruta}", class: "foto-receta", src: "{url}", alt: "{crate::i18n::con(crate::i18n::actual(), Texto::FotoNumeroDe, &[&(indice + 1).to_string(), &nombre])}", loading: "lazy" }
                    }
                }
            }

            // Receta propia con espacio: permitir agregar una foto
            if puede_subir
            {
                BotonFoto
                {
                    id: "foto-nueva",
                    ocupado: subiendo(),
                    etiqueta: crate::i18n::ahora(Texto::AgregarFoto),
                    al_elegir: move |archivo| subir_foto.call(archivo),
                }
                p { class: "nota", "{crate::i18n::con(crate::i18n::actual(), Texto::FotosMaximo, &[&api::foro::FOTOS_MAXIMAS.to_string()])}" }
            }

            if let Some(texto) = aviso()
            {
                p { class: "barra-aviso", role: "status", "{texto}" }
            }
        }
    }
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas {
    // LIBRERÍAS

    use super::extension; // Extensión según el tipo de imagen

    // FUNCIONES

    /// Comprueba que solo se aceptan los tipos que admite el bucket.
    #[test]
    fn solo_tipos_del_bucket() {
        assert_eq!(extension("image/jpeg"), Some("jpg"));
        assert_eq!(extension("image/png"), Some("png"));
        assert_eq!(extension("image/webp"), Some("webp"));
        assert_eq!(extension("image/gif"), None);
        assert_eq!(extension(""), None);
    }
}
