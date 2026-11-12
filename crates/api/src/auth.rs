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
use crate::db;

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

/// Dirección y clave anónima, o el motivo por el que no están.
#[cfg(feature = "server")]
fn supabase() -> Result<&'static (String, String), String> {
    SUPABASE.get().ok_or_else(|| {
        "faltan SUPABASE_URL y SUPABASE_ANON_KEY en el servidor".to_string()
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
#[cfg(feature = "server")]
async fn token_con_contrasena(
    correo: &str,
    contrasena: &str,
) -> Result<serde_json::Value, String> {
    let (url, anon) = supabase()?;

    let respuesta = reqwest::Client::new()
        .post(format!("{url}/auth/v1/token?grant_type=password"))
        .header("apikey", anon)
        .json(&serde_json::json!({ "email": correo, "password": contrasena }))
        .send()
        .await
        .map_err(|causa| format!("no se pudo hablar con Supabase Auth: {causa}"))?;

    let (exito, cuerpo) = leer_respuesta(respuesta).await;

    if !exito {
        let motivo = motivo_de(&cuerpo);
        let codigo = cuerpo["error_code"].as_str().unwrap_or("");

        return Err(if codigo == "invalid_credentials"
            || motivo.contains("Invalid login credentials")
            || motivo.contains("invalid_grant")
        {
            "Correo o contraseña incorrectos".to_string()
        } else if codigo == "email_not_confirmed" || motivo.contains("Email not confirmed") {
            "Confirma el correo que te enviamos y vuelve a entrar".to_string()
        } else if motivo.is_empty() {
            format!("Supabase Auth rechazó el acceso ({})", cuerpo["code"].as_u64().unwrap_or(0))
        } else {
            format!("Supabase Auth: {motivo}")
        });
    }

    Ok(cuerpo)
}

/// Arma la sesión con la respuesta de Supabase Auth: verifica el token y lee el perfil.
#[cfg(feature = "server")]
async fn iniciada_de(cuerpo: &serde_json::Value) -> Result<SesionIniciada, String> {
    let token = cuerpo["access_token"]
        .as_str()
        .ok_or_else(|| "Supabase Auth no devolvió un token".to_string())?;
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
async fn clave_de(kid: &str) -> Result<jsonwebtoken::DecodingKey, String> {
    if let Some(clave) = clave_en_cache(kid) {
        return Ok(clave);
    }

    let recientes = match CLAVES.read() {
        Ok(claves) => claves
            .as_ref()
            .is_some_and(|(_, descarga)| descarga.elapsed() < ESPERA_ENTRE_DESCARGAS),
        Err(_) => false,
    };

    if recientes {
        return Err("La sesión caducó; vuelve a entrar".to_string());
    }

    let (url, anon) = supabase()?;

    let descargadas: jsonwebtoken::jwk::JwkSet = reqwest::Client::new()
        .get(format!("{url}/auth/v1/.well-known/jwks.json"))
        .header("apikey", anon)
        .send()
        .await
        .map_err(|causa| format!("no se pudo verificar la sesión: {causa}"))?
        .json()
        .await
        .map_err(|causa| format!("claves ilegibles de Supabase Auth: {causa}"))?;

    if let Ok(mut claves) = CLAVES.write() {
        *claves = Some((descargadas, std::time::Instant::now()));
    }

    clave_en_cache(kid).ok_or_else(|| "La sesión caducó; vuelve a entrar".to_string())
}

/// Verifica la firma, la caducidad y el emisor de un token y devuelve el id de su cuenta.
///
/// La comprobación es local: Supabase solo se consulta para descargar sus claves públicas.
#[cfg(feature = "server")]
async fn cuenta_del_token(token: &str) -> Result<String, String> {
    let caducada = || "La sesión caducó; vuelve a entrar".to_string();
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
async fn perfil_de(cuenta: &str) -> Result<Sesion, String> {
    let pool = db::pool().ok_or_else(|| "la base de datos no está disponible".to_string())?;

    let fila = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT alias, rol, mayor_de_edad FROM usuario WHERE id = $1::uuid",
    )
    .bind(cuenta)
    .fetch_optional(pool)
    .await
    .map_err(|_| "no se pudo leer el perfil".to_string())?;

    let Some((alias, rol, mayor_de_edad)) = fila else {
        return Err("La cuenta no tiene perfil en Ratio".to_string());
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
    let cuerpo = token_con_contrasena(&credenciales.correo, &credenciales.contrasena)
        .await
        .map_err(ServerFnError::new)?;

    iniciada_de(&cuerpo).await.map_err(ServerFnError::new)
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
    let (url, anon) = supabase().map_err(ServerFnError::new)?;

    let respuesta = reqwest::Client::new()
        .post(format!("{url}/auth/v1/token?grant_type=refresh_token"))
        .header("apikey", anon)
        .json(&serde_json::json!({ "refresh_token": refresco }))
        .send()
        .await
        .map_err(|_| ServerFnError::new("Auth service unreachable"))?;

    // Un 4xx es rechazo del token; un 5xx es fallo de Supabase y se reintenta
    let rechazado = respuesta.status().is_client_error();
    let (exito, cuerpo) = leer_respuesta(respuesta).await;

    if rechazado {
        return Ok(None);
    }
    if !exito {
        return Err(ServerFnError::new("Auth service error"));
    }

    iniciada_de(&cuerpo).await.map(Some).map_err(ServerFnError::new)
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
    let (url, anon) = supabase().map_err(ServerFnError::new)?;

    let respuesta = reqwest::Client::new()
        .post(format!("{url}/auth/v1/signup"))
        .header("apikey", anon)
        .json(&serde_json::json!({
            "email": datos.correo,
            "password": datos.contrasena,
            "data": {
                "alias": datos.alias,
                "mayor_de_edad": if datos.mayor_de_edad { "true" } else { "false" },
            },
        }))
        .send()
        .await
        .map_err(|causa| ServerFnError::new(format!("no se pudo hablar con Supabase Auth: {causa}")))?;

    let (exito, cuerpo) = leer_respuesta(respuesta).await;

    if !exito {
        let motivo = motivo_de(&cuerpo);
        let codigo = cuerpo["error_code"].as_str().unwrap_or("");

        return Err(ServerFnError::new(
            if codigo == "user_already_exists"
                || motivo.contains("already registered")
                || motivo.contains("already been registered")
            {
                "Ese correo ya tiene cuenta: entra con tu contraseña".to_string()
            } else if codigo == "weak_password" || motivo.contains("at least 6 characters") {
                "La contraseña necesita al menos 6 caracteres".to_string()
            } else if codigo == "over_email_send_rate_limit" || motivo.contains("rate limit") {
                "Demasiados correos seguidos: espera unos minutos y vuelve a probar".to_string()
            } else if codigo == "signup_disabled" || motivo.contains("Signups not allowed") {
                "El alta de cuentas está desactivada en el proyecto".to_string()
            } else if codigo == "email_address_invalid" || motivo.contains("is invalid") {
                "Ese correo no parece válido".to_string()
            } else if motivo.is_empty() {
                "No se pudo crear la cuenta".to_string()
            } else {
                format!("No se pudo crear la cuenta: {motivo}")
            },
        ));
    }

    if cuerpo["access_token"].as_str().is_none() {
        return Err(ServerFnError::new(
            "Cuenta creada. Confirma el correo que te enviamos y luego entra",
        ));
    }

    iniciada_de(&cuerpo).await.map_err(ServerFnError::new)
}

/// Revalida un token guardado y devuelve el perfil que le corresponde.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el token caducó o si la cuenta ya no tiene perfil.
#[server(endpoint = "sesion")]
pub async fn sesion(token: String) -> Result<crate::Sesion, ServerFnError> {
    let cuenta = cuenta_del_token(&token)
        .await
        .map_err(ServerFnError::new)?;

    perfil_de(&cuenta).await.map_err(ServerFnError::new)
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
