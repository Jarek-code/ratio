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

/// Si se elige con la galería del sistema en vez de la entrada de archivos.
const GALERIA_NATIVA: bool = cfg!(target_os = "ios");

/// Si la plataforma tiene con qué elegir una foto.
///
/// Android y escritorio usan el `<input type="file">`, que en Android abre vía wry (el
/// WebView de dioxus-desktop, que implementa `onShowFileChooser`). iOS usa su galería
/// nativa (`selector_ios`), porque el WebView de iOS necesita más permisos.
const HAY_SELECTOR: bool = true;

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
    elegida: Result<Option<(Vec<u8>, String)>, String>,
) -> Result<Option<String>, String> {
    match elegida? {
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

/// Borra el archivo del almacenamiento con el token de la cuenta.
///
/// Es un borrado de mejor esfuerzo: si falla, la foto ya no se muestra porque la receta dejó
/// de referenciarla, y el objeto queda huérfano en el contenedor.
async fn borrar_de_storage(token: &str, ruta: &str) {
    let (Some(url), Some(anon)) = (base_url(), clave()) else {
        return;
    };

    let mut peticion =
        reqwest::Client::new().delete(format!("{url}/storage/v1/object/{BUCKET}/{ruta}"));
    peticion = peticion.header("apikey", anon);
    peticion = peticion.header("Authorization", format!("Bearer {token}"));

    if let Err(causa) = peticion.send().await {
        dioxus::logger::tracing::warn!("la foto quedó en el almacenamiento: {causa}");
    }
}

/// Abre la galería del sistema, que solo existe en iOS. Devuelve los bytes y el tipo, o
/// `None` si la persona canceló.
async fn galeria_del_sistema() -> Result<Option<(Vec<u8>, String)>, String> {
    #[cfg(target_os = "ios")]
    {
        let foto = crate::selector_ios::elegir_de_galeria().await?;

        // Igual que en Android: una foto grande se reduce aquí en vez de rechazarla
        let foto = match foto {
            Some(bytes) if bytes.len() as u64 > BYTES_MAXIMOS => Some(comprimir(bytes)?),
            otra => otra,
        };

        Ok(foto.map(|bytes| (bytes, "image/jpeg".to_string()))) // La galería entrega JPEG
    }

    #[cfg(not(target_os = "ios"))]
    {
        Err({ crate::i18n::ahora(Texto::FotoNoSeAbrioElSelector) }.to_string())
    }
}

/// Lo que manda el navegador cuando la persona elige (o cancela) una foto.
#[derive(serde::Deserialize)]
struct FotoElegida {
    /// Contenido del archivo en base64, sin la cabecera del `data URL`.
    datos: String,
    /// Tipo MIME que informa el sistema; `"error"` si el navegador no pudo leer el archivo.
    #[serde(default)]
    tipo: Option<String>,
}

/// Guion que escucha la entrada de archivos y manda la foto en base64.
///
/// No se usa el evento de formulario de Dioxus: en Android, `dioxus-desktop` devuelve la
/// lista de archivos vacía, así que el archivo nunca llega a Rust. El navegador sí puede
/// leerlo con `FileReader`, y los bytes viajan como texto base64.
fn guion(id: &str) -> String {
    format!(
        r#"
        const boton = document.getElementById("{id}");
        if (boton) {{
            // La entrada vive en un iframe aparte: Dioxus intercepta los clics de los
            // `<input type="file">` de su propio documento y en Android no abre nada.
            const marco = document.createElement('iframe');
            marco.style.display = 'none';
            document.body.appendChild(marco);
            const documento = marco.contentDocument;
            documento.open();
            documento.write('<input type="file" accept="image/jpeg,image/png,image/webp">');
            documento.close();
            const entrada = documento.querySelector('input');

            entrada.addEventListener('change', () => {{
                const archivo = entrada.files && entrada.files[0];
                if (!archivo) {{
                    dioxus.send({{ datos: '', tipo: null }});
                    return;
                }}
                const lector = new FileReader();
                lector.onload = () => {{
                    const texto = String(lector.result || '');
                    const coma = texto.indexOf(',');
                    entrada.value = ''; // Permite elegir la misma foto otra vez
                    dioxus.send({{
                        datos: coma >= 0 ? texto.slice(coma + 1) : '',
                        tipo: archivo.type || 'image/jpeg',
                    }});
                }};
                lector.onerror = () => {{
                    entrada.value = '';
                    dioxus.send({{ datos: '', tipo: 'error' }});
                }};
                lector.readAsDataURL(archivo);
            }});

            boton.addEventListener('click', () => {{
                // Se abre dentro del gesto de la persona, así el sistema lo permite
                if (typeof entrada.showPicker === 'function') {{
                    try {{
                        entrada.showPicker();
                        return;
                    }} catch (error) {{
                        // Sin permiso para `showPicker` se prueba el clic normal
                    }}
                }}
                entrada.click();
            }});
        }}
        await new Promise(() => {{}}); // Queda escuchando hasta que se cierre la pantalla
        "#,
        id = id
    )
}

/// Convierte lo que manda el navegador en bytes y tipo MIME, revisando el tamaño.
fn procesar(elegida: FotoElegida) -> Result<Option<(Vec<u8>, String)>, String> {
    if elegida.datos.is_empty() {
        if elegida.tipo.as_deref() == Some("error") {
            return Err({ crate::i18n::ahora(Texto::FotoNoLeida) }.to_string());
        }
        return Ok(None); // Selección cancelada
    }

    let mut bytes = decodificar_base64(&elegida.datos)?;
    let mut tipo = elegida
        .tipo
        .filter(|tipo| !tipo.is_empty())
        .unwrap_or_else(|| "image/jpeg".to_string());

    // Una foto grande se reduce aquí; si aun así no cabe, se avisa
    if bytes.len() as u64 > BYTES_MAXIMOS {
        bytes = comprimir(std::mem::take(&mut bytes))?;
        tipo = "image/jpeg".to_string();

        if bytes.len() as u64 > BYTES_MAXIMOS {
            return Err({ crate::i18n::ahora(Texto::FotoPesaDemasiado) }.to_string());
        }
    }

    Ok(Some((bytes, tipo)))
}

/// Decodifica el base64 que manda el navegador (`btoa`, sin saltos de línea).
fn decodificar_base64(texto: &str) -> Result<Vec<u8>, String> {
    fn valor(caracter: u8) -> Option<u8> {
        match caracter {
            b'A'..=b'Z' => Some(caracter - b'A'),
            b'a'..=b'z' => Some(caracter - b'a' + 26),
            b'0'..=b'9' => Some(caracter - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None, // El relleno y los saltos de línea se ignoran
        }
    }

    let mut bytes = Vec::with_capacity(texto.len() * 3 / 4);
    let mut acumulado = 0u32;
    let mut bits = 0u32;

    for caracter in texto.bytes() {
        let Some(valor) = valor(caracter) else {
            continue;
        };
        acumulado = (acumulado << 6) | u32::from(valor);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((acumulado >> bits) as u8);
        }
    }

    if bytes.is_empty() {
        return Err({ crate::i18n::ahora(Texto::FotoNoLeida) }.to_string());
    }

    Ok(bytes)
}

/// Control para elegir una foto: la galería nativa en iOS, el selector del sistema en el resto.
///
/// `id` distingue el botón dentro de la página, `ocupado` lo desactiva mientras se sube y
/// `al_elegir` recibe el resultado: la foto, la cancelación o el motivo del fallo.
#[component]
pub fn BotonFoto(
    id: String,
    ocupado: bool,
    etiqueta: String,
    al_elegir: EventHandler<Result<Option<(Vec<u8>, String)>, String>>,
) -> Element {
    let mut esperando = use_signal(|| false); // El selector se abrió y no llegó nada aún

    // El navegador avisa cuando la persona elige una foto
    use_future({
        let id = id.clone();
        move || {
            let id = id.clone();
            async move {
                if GALERIA_NATIVA {
                    return; // En iOS se abre la galería del sistema
                }

                let mut escucha = dioxus::document::eval(&guion(&id));
                while let Ok(elegida) = escucha.recv::<FotoElegida>().await {
                    esperando.set(false);
                    al_elegir.call(procesar(elegida));
                }
            }
        }
    });

    rsx! {
        if GALERIA_NATIVA
        {
            button
            {
                class: "chip chip--enlace foto-subir",
                r#type: "button",
                disabled: ocupado || esperando(),
                onclick: move |_| {
                    esperando.set(true);
                    spawn(async move {
                        let elegida = galeria_del_sistema().await;
                        esperando.set(false);
                        al_elegir.call(elegida);
                    });
                },
                if ocupado
                {
                    {crate::i18n::ahora(Texto::SubiendoFoto)}
                }
                else if esperando()
                {
                    {crate::i18n::ahora(Texto::AbriendoElSelector)}
                }
                else
                {
                    "{etiqueta}"
                }
            }
        }
        else
        {
            button
            {
                id: "{id}",
                class: "chip chip--enlace foto-subir",
                r#type: "button",
                disabled: ocupado || esperando(),
                onclick: move |_| {
                    esperando.set(true);
                    // Sin foto en un minuto, se avisa: el selector no llegó a abrirse
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                        if esperando() {
                            esperando.set(false);
                            al_elegir.call(Err(
                                crate::i18n::ahora(Texto::FotoNoSeAbrioElSelector).to_string(),
                            ));
                        }
                    });
                },
                if ocupado
                {
                    {crate::i18n::ahora(Texto::SubiendoFoto)}
                }
                else if esperando()
                {
                    {crate::i18n::ahora(Texto::AbriendoElSelector)}
                }
                else
                {
                    "{etiqueta}"
                }
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
    let receta_id_quitar = receta.id.clone(); // Copia para el borrado
    let nombre = receta.receta.nombre.clone();

    // Elegir la foto y subirla; en iOS el archivo llega vacío y se abre la galería nativa
    let subir_foto = use_callback(move |elegida: Result<Option<(Vec<u8>, String)>, String>| {
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
            let resultado = match elegida {
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

    // Quitar una foto propia: primero de la receta y después del almacenamiento
    let quitar = use_callback(move |ruta: String| {
        let Some(token) = crate::sesion::token(&sesion) else {
            aviso.set(Some(
                { crate::i18n::ahora(Texto::FotoSinSesion) }.to_string(),
            ));
            return;
        };
        let id = receta_id_quitar.clone();
        aviso.set(None);

        spawn(async move {
            match crate::red::con_plazo(api::foro::quitar_foto(token.clone(), id, ruta.clone()))
                .await
            {
                Ok(rutas) => {
                    fotos_sel.set(rutas);
                    borrar_de_storage(&token, &ruta).await;
                }
                Err(causa) => aviso.set(Some(crate::i18n::con(
                    crate::i18n::actual(),
                    Texto::FotoNoQuitada,
                    &[&crate::red::mensaje(&causa)],
                ))),
            }
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
                        div { key: "{ruta}", class: "foto-celda",
                            img { class: "foto-receta", src: "{url}", alt: "{crate::i18n::con(crate::i18n::actual(), Texto::FotoNumeroDe, &[&(indice + 1).to_string(), &nombre])}", loading: "lazy" }

                            if propia
                            {
                                button {
                                    class: "foto-quitar",
                                    r#type: "button",
                                    aria_label: "{crate::i18n::ahora(Texto::QuitarFoto)}",
                                    onclick: {
                                        let ruta = ruta.clone();
                                        move |_| quitar.call(ruta.clone())
                                    },
                                    "✕"
                                }
                            }
                        }
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
                    al_elegir: move |elegida| subir_foto.call(elegida),
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

    use super::{decodificar_base64, extension}; // Extensión y decodificación de la foto

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

    /// El base64 que manda el navegador se decodifica tal cual, con relleno y sin él.
    #[test]
    fn base64_como_lo_manda_el_navegador() {
        assert_eq!(decodificar_base64("aG9sYQ==").unwrap(), b"hola");
        assert_eq!(decodificar_base64("aG9sYQ").unwrap(), b"hola");
        assert_eq!(decodificar_base64("AAECAw==").unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(decodificar_base64("+/8=").unwrap(), vec![0xfb, 0xff]);
        assert!(decodificar_base64("").is_err());
    }
}

/// Reduce una foto que pasa del límite del contenedor.
///
/// Se decodifica, se limita el lado mayor a 1600 px y se guarda como JPEG con calidad 80,
/// que para una foto de teléfono pesa muy por debajo de los 5 MB sin perder detalle visible.
fn comprimir(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    /// Lado mayor de la foto ya comprimida.
    const LADO_MAXIMO: u32 = 1600;
    /// Calidad del JPEG resultante.
    const CALIDAD: u8 = 80;

    let imagen = image::load_from_memory(&bytes)
        .map_err(|_| crate::i18n::ahora(Texto::FotoNoLeida).to_string())?;

    let reducida = if imagen.width() > LADO_MAXIMO || imagen.height() > LADO_MAXIMO {
        imagen.thumbnail(LADO_MAXIMO, LADO_MAXIMO)
    } else {
        imagen
    };

    let mut salida = Vec::new();
    let mut codificador = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut salida, CALIDAD);
    codificador
        .encode_image(&reducida)
        .map_err(|_| crate::i18n::ahora(Texto::FotoNoLeida).to_string())?;

    Ok(salida)
}
