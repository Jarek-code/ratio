//! Acceso y alta contra Supabase Auth, y verificación del token de sesión.
//!
//! Las contraseñas nunca pasan por este código más allá del reenvío: Supabase Auth guarda su
//! hash en `auth.users.encrypted_password` y devuelve un token. Aquí solo se pide el token y,
//! con él, se lee el perfil de `usuario` para armar la `Sesion`.

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

/// Pide el token a Supabase con correo y contraseña.
#[cfg(feature = "server")]
async fn token_con_contrasena(correo: &str, contrasena: &str) -> Result<String, String> {
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

        return Err(if motivo.contains("Invalid login credentials") || motivo.contains("invalid_grant") {
            "Correo o contraseña incorrectos".to_string()
        } else if motivo.contains("Email not confirmed") {
            "Confirma el correo que te enviamos y vuelve a entrar".to_string()
        } else if motivo.is_empty() {
            "Supabase Auth rechazó el acceso".to_string()
        } else {
            format!("Supabase Auth: {motivo}")
        });
    }

    cuerpo["access_token"]
        .as_str()
        .map(|token| token.to_string())
        .ok_or_else(|| "Supabase Auth no devolvió un token".to_string())
}

/// Comprueba un token contra Supabase y devuelve el id de su cuenta.
#[cfg(feature = "server")]
async fn cuenta_del_token(token: &str) -> Result<String, String> {
    let (url, anon) = supabase()?;

    let respuesta = reqwest::Client::new()
        .get(format!("{url}/auth/v1/user"))
        .header("apikey", anon)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|causa| format!("no se pudo verificar la sesión: {causa}"))?;

    if !respuesta.status().is_success() {
        return Err("La sesión caducó; vuelve a entrar".to_string());
    }

    let cuerpo: serde_json::Value = respuesta
        .json()
        .await
        .map_err(|causa| format!("respuesta ilegible de Supabase Auth: {causa}"))?;

    cuerpo["id"]
        .as_str()
        .map(|id| id.to_string())
        .ok_or_else(|| "la cuenta del token no tiene id".to_string())
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
    let token = token_con_contrasena(&credenciales.correo, &credenciales.contrasena)
        .await
        .map_err(ServerFnError::new)?;
    let cuenta = cuenta_del_token(&token).await.map_err(ServerFnError::new)?;
    let perfil = perfil_de(&cuenta).await.map_err(ServerFnError::new)?;

    Ok(SesionIniciada { token, perfil })
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

        return Err(ServerFnError::new(
            if motivo.contains("already registered") || motivo.contains("already been registered") {
                "Ese correo ya tiene cuenta: entra con tu contraseña".to_string()
            } else if motivo.contains("at least 6 characters") || motivo.contains("weak") {
                "La contraseña necesita al menos 6 caracteres".to_string()
            } else if motivo.contains("invalid format") || motivo.contains("is invalid") {
                "Ese correo no parece válido".to_string()
            } else if motivo.is_empty() {
                "No se pudo crear la cuenta".to_string()
            } else {
                format!("No se pudo crear la cuenta: {motivo}")
            },
        ));
    }

    let Some(token) = cuerpo["access_token"].as_str() else {
        return Err(ServerFnError::new(
            "Cuenta creada. Confirma el correo que te enviamos y luego entra",
        ));
    };

    let cuenta = cuenta_del_token(token)
        .await
        .map_err(ServerFnError::new)?;
    let perfil = perfil_de(&cuenta).await.map_err(ServerFnError::new)?;

    Ok(SesionIniciada {
        token: token.to_string(),
        perfil,
    })
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
