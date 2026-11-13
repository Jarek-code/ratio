//! Acceso y alta contra Supabase Auth, y verificación del token de sesión.
//!
//! Las contraseñas nunca pasan por este código más allá del reenvío: Supabase Auth guarda su
//! hash en `auth.users.encrypted_password` y devuelve un token. Aquí solo se pide el token, se
//! verifica su firma con las claves públicas del proyecto y se lee el perfil de `usuario`.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
use crate::Sesion;
#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta
#[cfg(feature = "server")]
use crate::base;
#[cfg(feature = "server")]
use crate::fallos; // Fallos con código: dato inválido, reintento o sesión

/// Credenciales del formulario de acceso.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credenciales {
    /// Correo de la cuenta.
    pub correo: String,
    /// Contraseña en claro solo hasta Supabase, que guarda su hash.
    pub contrasena: String,
}

/// Datos del formulario de alta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Registro {
    /// Correo de la cuenta nueva.
    pub correo: String,
    /// Contraseña elegida.
    pub contrasena: String,
    /// Alias visible en La Barra (obligatorio).
    pub alias: String,
    /// Declaración de mayoría de edad, que habilita las recetas con alcohol.
    pub mayor_de_edad: bool,
}

/// Largo mínimo de la contraseña, igual que el mínimo que acepta Supabase Auth.
pub const CONTRASENA_MINIMA: usize = 6;

/// Largo del alias: entre estos dos valores, igual que el CHECK de `usuario`.
pub const ALIAS_MINIMO: usize = 3;
pub const ALIAS_MAXIMO: usize = 24;

/// Revisa un correo antes de mandarlo a Supabase: vacío, con espacios o sin dominio.
///
/// Devuelve el aviso que se le muestra a la persona, o `None` si tiene forma de correo. No
/// sustituye la comprobación de Supabase: solo evita un viaje que ya se sabe que falla.
pub fn revisar_correo(correo: &str) -> Option<&'static str> {
    let correo = correo.trim();

    if correo.is_empty() {
        return Some("Escribe tu correo.");
    }
    if correo.contains(char::is_whitespace) {
        return Some("El correo no lleva espacios.");
    }

    let Some((usuario, dominio)) = correo.split_once('@') else {
        return Some("Escribe un correo válido, con arroba y dominio.");
    };

    let dominio_valido = dominio.contains('.')
        && !dominio.starts_with('.')
        && !dominio.ends_with('.')
        && !dominio.contains('@');
    if usuario.is_empty() || !dominio_valido {
        return Some("Escribe un correo válido, con arroba y dominio.");
    }

    None
}

/// Revisa una contraseña antes de mandarla a Supabase.
pub fn revisar_contrasena(contrasena: &str) -> Option<&'static str> {
    if contrasena.is_empty() {
        return Some("Escribe tu contraseña.");
    }
    if contrasena.chars().count() < CONTRASENA_MINIMA {
        return Some("La contraseña necesita al menos 6 caracteres.");
    }

    None
}

/// Revisa el alias de un alta: obligatorio, con largo acotado y sin arroba.
pub fn revisar_alias(alias: &str) -> Option<&'static str> {
    let largo = alias.trim().chars().count();

    if largo == 0 {
        return Some("Escribe un alias.");
    }
    if largo < ALIAS_MINIMO {
        return Some("El alias necesita 3 caracteres o más.");
    }
    if largo > ALIAS_MAXIMO {
        return Some("El alias no puede pasar de 24 caracteres.");
    }
    if alias.contains('@') {
        return Some("El alias no lleva arroba: es el nombre que verán los demás.");
    }

    None
}

/// Revisa las credenciales del acceso y devuelve el primer aviso que encuentre.
pub fn revisar_credenciales(credenciales: &Credenciales) -> Option<&'static str> {
    revisar_correo(&credenciales.correo).or_else(|| revisar_contrasena(&credenciales.contrasena))
}

/// Lo que devuelve un acceso correcto: el token y el perfil que lo acompaña.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SesionIniciada {
    /// Token de acceso de Supabase, que la app guarda y reenvía.
    pub token: String,
    /// Token de renovación: con él se pide un token de acceso nuevo antes de que caduque.
    #[serde(default)]
    pub refresco: String,
    /// Momento en que caduca `token`, en segundos desde 1970 (0 si no se conoce).
    #[serde(default)]
    pub caduca_en: i64,
    /// Perfil del usuario que corresponde al token.
    pub perfil: crate::Sesion,
}

/// Dirección del proyecto y clave pública de Supabase (solo servidor).
#[cfg(feature = "server")]
static SUPABASE: OnceLock<(String, String)> = OnceLock::new();

/// Guarda la dirección y la clave anónima de Supabase. La llama el binario al arrancar.
#[cfg(feature = "server")]
pub fn configurar(url: String, anon: String) {
    let _ = SUPABASE.set((url.trim_end_matches('/').to_string(), anon));
}

/// ¿Hay credenciales de Supabase configuradas?
#[cfg(feature = "server")]
pub fn configurado() -> bool {
    SUPABASE.get().is_some()
}

/// Dirección y clave anónima, o el fallo que se le cuenta al cliente.
#[cfg(feature = "server")]
fn supabase() -> Result<&'static (String, String), ServerFnError> {
    SUPABASE.get().ok_or_else(|| {
        dioxus::logger::tracing::error!("faltan SUPABASE_URL y SUPABASE_ANON_KEY");

        fallos::reintento(
            "El servidor no tiene configurado el acceso (Supabase). Avisa a quien lo administra.",
        )
    })
}

/// Estado y cuerpo de una respuesta de Supabase Auth.
#[cfg(feature = "server")]
async fn leer_respuesta(respuesta: reqwest::Response) -> (bool, serde_json::Value) {
    let exito = respuesta.status().is_success();
    let cuerpo = respuesta.json().await.unwrap_or(serde_json::Value::Null);

    (exito, cuerpo)
}

/// Motivo que manda Supabase en el cuerpo, si lo trae.
#[cfg(feature = "server")]
fn motivo_de(cuerpo: &serde_json::Value) -> String {
    for clave in ["error_description", "msg", "message", "error"] {
        if let Some(texto) = cuerpo[clave].as_str() {
            return texto.to_string();
        }
    }

    String::new()
}

/// Pide el token a Supabase con correo y contraseña; devuelve el cuerpo de la respuesta.
///
/// # Errores
///
/// Devuelve un fallo con código [`fallos::CODIGO_DATOS`] si el correo o la contraseña no son
/// válidos, y [`fallos::CODIGO_REINTENTO`] si Supabase no contesta a tiempo.
#[cfg(feature = "server")]
async fn token_con_contrasena(
    correo: &str,
    contrasena: &str,
) -> Result<serde_json::Value, ServerFnError> {
    if let Some(aviso) = revisar_correo(correo).or_else(|| revisar_contrasena(contrasena)) {
        return Err(fallos::datos(aviso));
    }

    let (url, anon) = supabase()?;

    let respuesta = base::cliente_http()?
        .post(format!("{url}/auth/v1/token?grant_type=password"))
        .header("apikey", anon)
        .json(&serde_json::json!({ "email": correo.trim(), "password": contrasena }))
        .send()
        .await
        .map_err(|causa| {
            dioxus::logger::tracing::error!("Supabase Auth no contestó: {causa}");

            fallos::reintento(
                "No se pudo hablar con el servicio de cuentas. Revisa la conexión y reintenta.",
            )
        })?;

    let (exito, cuerpo) = leer_respuesta(respuesta).await;

    if !exito {
        let motivo = motivo_de(&cuerpo);
        let codigo = cuerpo["error_code"].as_str().unwrap_or("");

        return Err(if codigo == "invalid_credentials"
            || motivo.contains("Invalid login credentials")
            || motivo.contains("invalid_grant")
        {
            fallos::datos("Correo o contraseña incorrectos.")
        } else if codigo == "email_not_confirmed" || motivo.contains("Email not confirmed") {
            fallos::datos("Confirma el correo que te enviamos y vuelve a entrar.")
        } else if codigo == "over_request_rate_limit" || motivo.contains("rate limit") {
            fallos::datos("Demasiados intentos seguidos: espera unos minutos y vuelve a probar.")
        } else if motivo.is_empty() {
            // Supabase contestó con un fallo suyo: reintentar sí sirve
            fallos::reintento(format!(
                "El servicio de cuentas falló ({}). Vuelve a intentarlo en un momento.",
                cuerpo["code"].as_u64().unwrap_or(0)
            ))
        } else {
            fallos::datos(format!("No se pudo entrar: {motivo}"))
        });
    }

    Ok(cuerpo)
}

/// Arma la sesión con la respuesta de Supabase Auth: verifica el token y lee el perfil.
#[cfg(feature = "server")]
async fn iniciada_de(cuerpo: &serde_json::Value) -> Result<SesionIniciada, ServerFnError> {
    let token = cuerpo["access_token"].as_str().ok_or_else(|| {
        fallos::reintento(
            "El servicio de cuentas no devolvió la sesión. Vuelve a intentarlo en un momento.",
        )
    })?;
    let cuenta = cuenta_del_token(token).await?;
    let perfil = perfil_de(&cuenta).await?;

    Ok(SesionIniciada {
        token: token.to_string(),
        refresco: cuerpo["refresh_token"].as_str().unwrap_or("").to_string(),
        caduca_en: cuerpo["expires_at"].as_i64().unwrap_or(0),
        perfil,
    })
}

/// Claves públicas con las que Supabase firma los tokens; se descargan una vez y se
/// renuevan cuando llega un token firmado con una clave que aún no está aquí.
#[cfg(feature = "server")]
static CLAVES: std::sync::RwLock<Option<(jsonwebtoken::jwk::JwkSet, std::time::Instant)>> =
    std::sync::RwLock::new(None);

/// Espera mínima entre dos descargas de claves: un token con una clave inventada no puede
/// obligar al servidor a llamar a Supabase en cada petición.
#[cfg(feature = "server")]
const ESPERA_ENTRE_DESCARGAS: std::time::Duration = std::time::Duration::from_secs(60);

/// Lo único que se lee del token: la cuenta a la que pertenece.
#[cfg(feature = "server")]
#[derive(Deserialize)]
struct Declaraciones {
    /// Id de la cuenta en `auth.users`.
    sub: String,
}

/// Clave de verificación con ese identificador entre las ya descargadas, si está.
#[cfg(feature = "server")]
fn clave_en_cache(kid: &str) -> Option<jsonwebtoken::DecodingKey> {
    let claves = CLAVES.read().ok()?;
    let clave = claves.as_ref()?.0.find(kid)?;

    jsonwebtoken::DecodingKey::from_jwk(clave).ok()
}

/// Clave pública que firmó el token: la de la caché o, si no está, la que publique Supabase.
#[cfg(feature = "server")]
async fn clave_de(kid: &str) -> Result<jsonwebtoken::DecodingKey, ServerFnError> {
    if let Some(clave) = clave_en_cache(kid) {
        return Ok(clave);
    }

    let recientes = match CLAVES.read() {
        Ok(claves) => claves
            .as_ref()
            .is_some_and(|(_, descarga)| descarga.elapsed() < ESPERA_ENTRE_DESCARGAS),
        Err(_) => false,
    };

    if recientes
    {
        return Err(fallos::sesion("La sesión caducó: vuelve a entrar."));
    }

    let (url, anon) = supabase()?;

    let descargadas: jsonwebtoken::jwk::JwkSet = base::cliente_http()?
        .get(format!("{url}/auth/v1/.well-known/jwks.json"))
        .header("apikey", anon)
        .send()
        .await
        .map_err(|causa| {
            dioxus::logger::tracing::error!("no se pudieron descargar las claves de Supabase: {causa}");

            fallos::reintento("No se pudo comprobar tu sesión: el servidor de cuentas no contestó. Reintenta.")
        })?
        .json()
        .await
        .map_err(|causa| {
            dioxus::logger::tracing::error!("claves ilegibles de Supabase: {causa}");

            fallos::reintento("No se pudo comprobar tu sesión. Vuelve a intentarlo en un momento.")
        })?;

    if let Ok(mut claves) = CLAVES.write() {
        *claves = Some((descargadas, std::time::Instant::now()));
    }

    clave_en_cache(kid).ok_or_else(|| fallos::sesion("La sesión caducó: vuelve a entrar."))
}

/// Verifica la firma, la caducidad y el emisor de un token y devuelve el id de su cuenta.
///
/// La comprobación es local: Supabase solo se consulta para descargar sus claves públicas.
#[cfg(feature = "server")]
async fn cuenta_del_token(token: &str) -> Result<String, ServerFnError> {
    let caducada = || fallos::sesion("La sesión caducó: vuelve a entrar.");
    if token.trim().is_empty()
    {
        return Err(caducada());
    }
    let (url, _) = supabase()?;

    let cabecera = jsonwebtoken::decode_header(token).map_err(|_| caducada())?;
    let kid = cabecera.kid.ok_or_else(caducada)?;
    let clave = clave_de(&kid).await?;

    let mut reglas = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::ES256);
    reglas.set_audience(&["authenticated"]);
    reglas.set_issuer(&[format!("{url}/auth/v1")]);

    jsonwebtoken::decode::<Declaraciones>(token, &clave, &reglas)
        .map(|verificado| verificado.claims.sub)
        .map_err(|_| caducada())
}

/// Lee el perfil de la cuenta en `usuario`.
#[cfg(feature = "server")]
async fn perfil_de(cuenta: &str) -> Result<Sesion, ServerFnError> {
    let pool = base::pool()?;

    let fila = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT alias, rol, mayor_de_edad FROM usuario WHERE id = $1::uuid",
    )
    .bind(cuenta)
    .fetch_optional(pool)
    .con_plazo("el perfil")
    .await?;

    let Some((alias, rol, mayor_de_edad)) = fila else {
        return Err(fallos::no_encontrado(
            "Tu cuenta todavía no tiene perfil en Ratio. Sal y vuelve a entrar.",
        ));
    };

    Ok(Sesion {
        usuario_id: cuenta.to_string(),
        alias,
        rol: if rol == "moderador" {
            crate::Rol::Moderador
        } else {
            crate::Rol::Usuario
        },
        mayor_de_edad,
    })
}

/// Entra con correo y contraseña contra Supabase Auth.
///
/// # Errores
///
/// Devuelve `ServerFnError` si faltan las variables de Supabase, si las credenciales no son
/// válidas o si la cuenta todavía no tiene perfil en Ratio.
#[server(endpoint = "entrar")]
pub async fn entrar(credenciales: Credenciales) -> Result<SesionIniciada, ServerFnError> {
    // Revisar los campos aquí también: la app puede quedarse vieja y mandar otra cosa
    if let Some(aviso) = revisar_credenciales(&credenciales) {
        return Err(fallos::datos(aviso));
    }

    let cuerpo = token_con_contrasena(&credenciales.correo, &credenciales.contrasena).await?;

    iniciada_de(&cuerpo).await
}

/// Pide a Supabase Auth un token de acceso nuevo con el token de renovación.
///
/// Devuelve `Ok(None)` si Supabase rechaza el token de renovación (caducado o revocado): la
/// app debe pedir que se vuelva a entrar. Un fallo de red llega como `Err` y se reintenta.
///
/// # Errores
///
/// Devuelve `ServerFnError` si faltan las variables de Supabase o si no se pudo hablar con él.
#[server(endpoint = "renovar")]
pub async fn renovar(refresco: String) -> Result<Option<SesionIniciada>, ServerFnError> {
    if refresco.trim().is_empty()
    {
        return Err(fallos::sesion("La sesión caducó: vuelve a entrar."));
    }

    let (url, anon) = supabase()?;

    let respuesta = base::cliente_http()?
        .post(format!("{url}/auth/v1/token?grant_type=refresh_token"))
        .header("apikey", anon)
        .json(&serde_json::json!({ "refresh_token": refresco }))
        .send()
        .await
        .map_err(|causa| {
            dioxus::logger::tracing::error!("Supabase Auth no contestó al renovar: {causa}");

            fallos::reintento("No se pudo renovar la sesión: el servicio de cuentas no contestó.")
        })?;

    // Un 4xx es rechazo del token; un 5xx es fallo de Supabase y se reintenta
    let rechazado = respuesta.status().is_client_error();
    let (exito, cuerpo) = leer_respuesta(respuesta).await;

    if rechazado {
        return Ok(None);
    }
    if !exito {
        return Err(fallos::reintento(
            "El servicio de cuentas falló al renovar la sesión. Reintenta en un momento.",
        ));
    }

    iniciada_de(&cuerpo).await.map(Some)
}

/// Da de alta una cuenta y, si Supabase la deja entrar de inmediato, inicia la sesión.
///
/// El alias y la mayoría de edad viajan como metadatos: el disparador `crear_perfil` de la
/// base crea con ellos la fila de `usuario`.
///
/// # Errores
///
/// Devuelve `ServerFnError` si faltan las variables de Supabase, si el correo ya existe, si la
/// contraseña es débil o si el proyecto exige confirmar el correo antes de entrar.
#[server(endpoint = "registrar")]
pub async fn registrar(datos: Registro) -> Result<SesionIniciada, ServerFnError> {
    // Los mismos avisos que da la app, por si llega una versión vieja del formulario
    if let Some(aviso) = revisar_correo(&datos.correo) {
        return Err(fallos::datos(aviso));
    }
    if let Some(aviso) = revisar_contrasena(&datos.contrasena) {
        return Err(fallos::datos(aviso));
    }
    if let Some(aviso) = revisar_alias(&datos.alias) {
        return Err(fallos::datos(aviso));
    }

    let (url, anon) = supabase()?;

    let respuesta = base::cliente_http()?
        .post(format!("{url}/auth/v1/signup"))
        .header("apikey", anon)
        .json(&serde_json::json!({
            "email": datos.correo.trim(),
            "password": datos.contrasena,
            "data": {
                "alias": datos.alias.trim(),
                "mayor_de_edad": if datos.mayor_de_edad { "true" } else { "false" },
            },
        }))
        .send()
        .await
        .map_err(|causa| {
            dioxus::logger::tracing::error!("Supabase Auth no contestó al registrar: {causa}");

            fallos::reintento(
                "No se pudo hablar con el servicio de cuentas. Revisa la conexión y reintenta.",
            )
        })?;

    let (exito, cuerpo) = leer_respuesta(respuesta).await;

    if !exito {
        let motivo = motivo_de(&cuerpo);
        let codigo = cuerpo["error_code"].as_str().unwrap_or("");

        return Err(if codigo == "user_already_exists"
            || motivo.contains("already registered")
            || motivo.contains("already been registered")
        {
            fallos::datos("Ese correo ya tiene cuenta: entra con tu contraseña.")
        } else if codigo == "weak_password" || motivo.contains("at least 6 characters") {
            fallos::datos("La contraseña necesita al menos 6 caracteres.")
        } else if codigo == "over_email_send_rate_limit" || motivo.contains("rate limit") {
            fallos::datos("Demasiados correos seguidos: espera unos minutos y vuelve a probar.")
        } else if codigo == "signup_disabled" || motivo.contains("Signups not allowed") {
            fallos::datos("El alta de cuentas está desactivada: avisa a quien administra el proyecto.")
        } else if codigo == "email_address_invalid" || motivo.contains("is invalid") {
            fallos::datos("Ese correo no parece válido: revísalo y vuelve a probar.")
        } else if motivo.is_empty() {
            fallos::reintento(
                "El servicio de cuentas no pudo crear la cuenta. Vuelve a intentarlo en un momento.",
            )
        } else {
            fallos::datos(format!("No se pudo crear la cuenta: {motivo}"))
        });
    }

    if cuerpo["access_token"].as_str().is_none() {
        return Err(fallos::datos(
            "Cuenta creada. Confirma el correo que te enviamos y luego entra.",
        ));
    }

    iniciada_de(&cuerpo).await
}

/// Revalida un token guardado y devuelve el perfil que le corresponde.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el token caducó o si la cuenta ya no tiene perfil.
#[server(endpoint = "sesion")]
pub async fn sesion(token: String) -> Result<crate::Sesion, ServerFnError> {
    let cuenta = cuenta_del_token(&token).await?;

    perfil_de(&cuenta).await
}

/// Cuenta a la que pertenece el token, si el token sigue siendo válido.
#[cfg(feature = "server")]
pub async fn cuenta(token: Option<&str>) -> Option<String> {
    match token {
        Some(token) => cuenta_del_token(token).await.ok(),
        None => None,
    }
}

/// ¿La sesión corresponde a una persona mayor de edad? Sin token, no.
#[cfg(feature = "server")]
pub async fn es_adulto(token: Option<&str>) -> bool {
    let Some(token) = token else {
        return false;
    };

    match cuenta_del_token(token).await {
        Ok(cuenta) => perfil_de(&cuenta).await.map(|p| p.mayor_de_edad).unwrap_or(false),
        Err(_) => false,
    }
}
