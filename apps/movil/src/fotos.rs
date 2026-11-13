//! Fotos de una receta (D1): galería y subida directa a Supabase Storage, parte de la app.
//!
//! La foto viaja de la app al bucket `fotos-recetas` con el token de la sesión; la política
//! del bucket solo deja subir a la carpeta `{cuenta}/…`. Después el servidor registra la ruta
//! en la receta. Las fotos se ven por la URL pública de la CDN de Storage.

// LIBRERÍAS

use api::receta::RecetaCompleta; // Receta con sus rutas de fotos
use dioxus::prelude::*; // Componentes, señales y FileData

use crate::sesion::SenalSesion; // Sesión con cuenta y token

// CONSTANTES Y VARIABLES GLOBALES

/// Dirección del proyecto de Supabase; se fija al compilar con `RATIO_SUPABASE_URL`.
const SUPABASE_URL: Option<&str> = option_env!("RATIO_SUPABASE_URL");

/// Clave pública (anon) de Supabase; se fija al compilar con `RATIO_SUPABASE_ANON_KEY`.
const SUPABASE_ANON: Option<&str> = option_env!("RATIO_SUPABASE_ANON_KEY");

/// Bucket de Storage con las fotos de las recetas.
const BUCKET: &str = "fotos-recetas";

/// Tamaño máximo de una foto, igual que el límite del bucket (5 MB).
const BYTES_MAXIMOS: u64 = 5 * 1024 * 1024;

/// Si la plataforma tiene cómo elegir una foto.
///
/// Dioxus 0.7 intercepta `<input type="file">` y en móvil no abre nada: iOS usa su galería
/// nativa (`selector_ios`) y en Android el botón se oculta en vez de quedar sin efecto.
const HAY_SELECTOR: bool = !cfg!(target_os = "android");

/// Si se usa la galería nativa de iOS en vez del selector de archivos.
const GALERIA_NATIVA: bool = cfg!(target_os = "ios");

// FUNCIONES

/// Dirección del proyecto sin la barra final, si la app se compiló con ella.
fn base() -> Option<&'static str>
{
    SUPABASE_URL.map(|url| url.trim_end_matches('/')).filter(|url| !url.is_empty())
}

/// URL pública de una foto en la CDN de Storage.
fn url_publica(ruta: &str) -> Option<String>
{
    base().map(|url| format!("{url}/storage/v1/object/public/{BUCKET}/{ruta}"))
}

/// Extensión del archivo según su tipo; `None` si no es JPEG, PNG ni WebP.
fn extension(tipo: &str) -> Option<&'static str>
{
    match tipo
    {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/webp" => Some("webp"),
        _ => None, // Tipo que el bucket rechaza
    }
}

/// Sube los bytes de una foto al bucket y registra su ruta en la receta; devuelve las rutas.
///
/// `tipo` es el tipo MIME de la imagen. Devuelve un mensaje en español listo para mostrar.
async fn subir_bytes(
    token: String,
    cuenta: String,
    receta_id: String,
    bytes: Vec<u8>,
    tipo: &str,
) -> Result<Vec<String>, String>
{
    let (Some(url), Some(anon)) = (base(), SUPABASE_ANON) else
    {
        return Err("Esta versión de la app no tiene configuradas las fotos.".to_string());
    };
    let Some(ext) = extension(tipo) else
    {
        return Err("Elige una foto JPG, PNG o WebP.".to_string());
    };
    if bytes.len() as u64 > BYTES_MAXIMOS
    {
        return Err("La foto pesa más de 5 MB.".to_string());
    }

    let nombre = crate::local::nuevo_id().ok_or_else(|| "No se pudo nombrar la foto.".to_string())?;
    let ruta = format!("{cuenta}/{receta_id}/{nombre}.{ext}"); // Carpeta propia de la cuenta

    // Subir directo a Storage con el token de la sesión
    let respuesta = reqwest::Client::new()
        .post(format!("{url}/storage/v1/object/{BUCKET}/{ruta}"))
        .bearer_auth(&token)
        .header("apikey", anon)
        .header("content-type", tipo)
        .body(bytes)
        .send()
        .await
        .map_err(|_| "Sin conexión: intenta subir la foto cuando vuelva la red.".to_string())?;
    if !respuesta.status().is_success()
    {
        return Err("Storage rechazó la foto.".to_string());
    }

    // Registrar la ruta en la receta
    api::foro::agregar_foto(token, receta_id, ruta)
        .await
        .map_err(|_| "La foto se subió, pero no se pudo agregar a la receta.".to_string())
}

/// Obtiene la foto que elige la persona: la galería nativa en iOS, el selector de archivos
/// en escritorio. Devuelve los bytes y su tipo MIME, o `None` si canceló.
#[cfg(target_os = "ios")]
async fn elegir(_archivo: Option<dioxus::html::FileData>) -> Result<Option<(Vec<u8>, String)>, String>
{
    let foto = crate::selector_ios::elegir_de_galeria().await?;
    Ok(foto.map(|bytes| (bytes, "image/jpeg".to_string()))) // La galería entrega JPEG
}

/// Obtiene la foto que elige la persona: la galería nativa en iOS, el selector de archivos
/// en escritorio. Devuelve los bytes y su tipo MIME, o `None` si canceló.
#[cfg(not(target_os = "ios"))]
async fn elegir(archivo: Option<dioxus::html::FileData>) -> Result<Option<(Vec<u8>, String)>, String>
{
    let Some(archivo) = archivo else
    {
        return Ok(None); // Selección cancelada
    };
    if archivo.size() > BYTES_MAXIMOS
    {
        return Err("La foto pesa más de 5 MB.".to_string());
    }
    let tipo = archivo.content_type().unwrap_or_default();
    let bytes = archivo
        .read_bytes()
        .await
        .map_err(|_| "No se pudo leer la foto.".to_string())?;

    Ok(Some((bytes.to_vec(), tipo))) // Bytes y tipo del archivo elegido
}

/// Galería de fotos de la receta y, para quien la creó, el botón para agregar una (D1).
#[component]
pub fn FotosReceta(receta: RecetaCompleta) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Cuenta y token para subir
    let mut fotos_sel = use_signal(|| receta.fotos.clone()); // Rutas que se muestran
    let mut subiendo = use_signal(|| false); // Subida en curso
    let mut aviso = use_signal(|| None::<String>); // Resultado visible

    let alias = sesion.read().as_ref().map(|guardada| guardada.perfil.alias.to_lowercase());
    let propia = alias.as_deref() == Some(receta.autor.to_lowercase().as_str()); // Los alias no distinguen mayúsculas
    let puede_subir = HAY_SELECTOR && propia && base().is_some() && fotos_sel().len() < api::foro::FOTOS_MAXIMAS;
    let receta_id = receta.id.clone();
    let nombre = receta.receta.nombre.clone();

    // Elegir la foto y subirla; en iOS el archivo llega vacío y se abre la galería nativa
    let subir_foto = use_callback(move |archivo: Option<dioxus::html::FileData>|
    {
        let (Some(token), Some(cuenta)) = (crate::sesion::token(&sesion), crate::sesion::cuenta(&sesion)) else
        {
            aviso.set(Some("Vuelve a entrar para subir fotos.".to_string()));
            return; // Sin sesión no se sube
        };
        let id = receta_id.clone();
        subiendo.set(true);
        aviso.set(None);

        spawn(async move
        {
            let resultado = match elegir(archivo).await
            {
                Ok(Some((bytes, tipo))) => subir_bytes(token, cuenta, id, bytes, &tipo).await.map(Some), // Foto elegida: subirla
                Ok(None) => Ok(None), // Selección cancelada
                Err(mensaje) => Err(mensaje), // No se pudo elegir
            };
            match resultado
            {
                Ok(Some(rutas)) => fotos_sel.set(rutas),
                Ok(None) => {}
                Err(mensaje) => aviso.set(Some(mensaje)),
            }
            subiendo.set(false);
        });
    });

    // Sin Storage configurado, o sin fotos ni permiso para subir: no ocupar espacio
    if base().is_none() || (fotos_sel().is_empty() && !puede_subir)
    {
        return rsx! {};
    }

    rsx!
    {
        section
        {
            class: "seccion",
            "aria-labelledby": "titulo-fotos",

            p { id: "titulo-fotos", class: "eyebrow", "Fotos" }

            div
            {
                class: "fotos-galeria",
                for (indice, ruta) in fotos_sel().iter().enumerate()
                {
                    if let Some(url) = url_publica(ruta)
                    {
                        img { key: "{ruta}", class: "foto-receta", src: "{url}", alt: "Foto {indice + 1} de {nombre}", loading: "lazy" }
                    }
                }
            }

            // Receta propia con espacio: permitir agregar una foto
            if puede_subir
            {
                // iOS: botón que abre la galería nativa; escritorio: selector de archivos
                if GALERIA_NATIVA
                {
                    button
                    {
                        class: "chip foto-subir",
                        r#type: "button",
                        disabled: subiendo(),
                        onclick: move |_| subir_foto.call(None),
                        if subiendo() { "Un momento…" } else { "Elegir de la galería" }
                    }
                }
                else
                {
                    label
                    {
                        class: "chip chip--enlace foto-subir",
                        r#for: "foto-nueva",
                        if subiendo() { "Subiendo…" } else { "Agregar foto" }
                    }
                    input
                    {
                        id: "foto-nueva",
                        class: "solo-lector",
                        r#type: "file",
                        accept: "image/jpeg,image/png,image/webp",
                        disabled: subiendo(),
                        onchange: move |evento: FormEvent|
                        {
                            if let Some(archivo) = evento.files().into_iter().next()
                            {
                                subir_foto.call(Some(archivo));
                            }
                        },
                    }
                }
                p { class: "nota", "Hasta {api::foro::FOTOS_MAXIMAS} fotos JPG, PNG o WebP de 5 MB como máximo." }
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
mod pruebas
{
    // LIBRERÍAS

    use super::extension; // Extensión según el tipo de imagen

    // FUNCIONES

    /// Comprueba que solo se aceptan los tipos que admite el bucket.
    #[test]
    fn solo_tipos_del_bucket()
    {
        assert_eq!(extension("image/jpeg"), Some("jpg"));
        assert_eq!(extension("image/png"), Some("png"));
        assert_eq!(extension("image/webp"), Some("webp"));
        assert_eq!(extension("image/gif"), None);
        assert_eq!(extension(""), None);
    }
}
