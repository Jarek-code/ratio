//! Notificaciones push con FCM HTTP v1 (fase posterior al MVP).
//!
//! El token del dispositivo lo entrega el cliente y se guarda en `dispositivo`; el envío vive
//! aquí, en el servidor, y nunca frena la escritura que lo provoca: se dispara después del
//! commit y, si FCM falla, solo queda en el log. El contenido del aviso es genérico a
//! propósito: la app vuelve a leer Actividad, que ya filtra alcohol, edad y contenido retirado.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError

#[cfg(feature = "server")]
use std::sync::Mutex; // Caché del token de acceso de Google
#[cfg(feature = "server")]
use std::sync::OnceLock; // Credenciales y caché instaladas una sola vez
#[cfg(feature = "server")]
use std::time::Duration; // Caducidad del token de OAuth
#[cfg(feature = "server")]
use std::time::Instant; // Instante de caducidad del token de OAuth
#[cfg(feature = "server")]
use std::time::SystemTime; // Reloj para los reclamos del JWT
#[cfg(feature = "server")]
use std::time::UNIX_EPOCH; // Base del reloj de los reclamos

#[cfg(feature = "server")]
use serde::Serialize; // Reclamos del JWT de la cuenta de servicio

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta

// CONSTANTES Y VARIABLES GLOBALES

/// URL de envío de FCM; se le inserta el id del proyecto.
#[cfg(feature = "server")]
const URL_ENVIO: &str = "https://fcm.googleapis.com/v1/projects/{}/messages:send";

/// Permiso que necesita el token de OAuth para enviar por FCM.
#[cfg(feature = "server")]
const PERMISO: &str = "https://www.googleapis.com/auth/firebase.messaging";

/// Vida del JWT de la cuenta de servicio, en segundos.
#[cfg(feature = "server")]
const VIDA_JWT: u64 = 3600;

/// Margen para no usar un token de acceso a punto de caducar, en segundos.
#[cfg(feature = "server")]
const MARGEN_TOKEN: u64 = 60;

/// Longitud máxima razonable de un token de dispositivo.
#[cfg(feature = "server")]
const TOKEN_MAXIMO: usize = 4096;

/// Cuenta de servicio de Firebase, ya validada.
#[cfg(feature = "server")]
#[derive(Clone)]
struct CuentaServicio {
    /// Proyecto de Firebase al que pertenece.
    project_id: String,
    /// Correo de la cuenta de servicio que firma el JWT.
    client_email: String,
    /// Clave privada RSA en formato PEM.
    private_key: String,
    /// Identificador de la clave, si el JSON lo trae.
    private_key_id: Option<String>,
    /// Endpoint de OAuth de Google.
    token_uri: String,
}

/// Credenciales instaladas al arrancar; `None` si el servidor no recibió ninguna.
#[cfg(feature = "server")]
static CUENTA: OnceLock<Option<CuentaServicio>> = OnceLock::new();

/// Token de acceso de Google vigente y el instante en que caduca.
#[cfg(feature = "server")]
static ACCESO: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();

// FUNCIONES

/// Instala las credenciales de la cuenta de servicio a partir del JSON de Firebase.
///
/// Se llama una sola vez al arrancar. Si el JSON no trae lo imprescindible, devuelve el motivo
/// y el servidor sigue en pie sin push.
///
/// # Errores
///
/// Devuelve un texto legible si el JSON está incompleto o no se puede interpretar.
#[cfg(feature = "server")]
pub fn configurar(json: &str) -> Result<(), String> {
    let valor: serde_json::Value = serde_json::from_str(json)
        .map_err(|causa| format!("FCM credentials are not valid JSON: {causa}"))?;

    let campo = |nombre: &str| {
        valor
            .get(nombre)
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| format!("FCM credentials missing field: {nombre}"))
    };

    let cuenta = CuentaServicio {
        project_id: campo("project_id")?,
        client_email: campo("client_email")?,
        private_key: campo("private_key")?,
        private_key_id: valor
            .get("private_key_id")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        token_uri: valor
            .get("token_uri")
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or("https://oauth2.googleapis.com/token")
            .to_string(),
    };

    let _ = CUENTA.set(Some(cuenta));

    Ok(())
}

/// Si el servidor ya tiene credenciales de FCM.
#[cfg(feature = "server")]
pub fn configurado() -> bool {
    CUENTA.get().is_some_and(Option::is_some)
}

/// Dispara un push para `para` sin frenar la petición que lo provocó.
///
/// Debe llamarse **después** del commit: si la escritura se deshace, no se avisa de nada.
/// Los fallos de FCM solo quedan en el log; un push perdido no rompe la operación.
#[cfg(feature = "server")]
pub fn avisar(para: String, tipo: &'static str, receta_id: String) {
    if !configurado() {
        return;
    }

    // El envío corre aparte: la respuesta no espera a Google
    tokio::spawn(async move {
        if let Err(causa) = enviar(&para, tipo, &receta_id).await {
            dioxus::logger::tracing::warn!("push a {para}: {causa}");
        }
    });
}

/// Envía el aviso a todos los dispositivos activos de la cuenta.
///
/// # Errores
///
/// Devuelve el motivo si no hay credenciales, si la base no responde o si no se logra el token
/// de OAuth. Los fallos por dispositivo se anotan y no cortan el resto.
#[cfg(feature = "server")]
async fn enviar(para: &str, tipo: &str, receta_id: &str) -> Result<(), String> {
    let cuenta = CUENTA
        .get()
        .and_then(Option::as_ref)
        .ok_or("no FCM credentials installed")?;
    let pool = crate::db::pool().ok_or("no database pool")?;

    // Un token por dispositivo activo, con el idioma de su cuenta; el aviso respeta el interruptor
    let filas = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT d.token, COALESCE(u.idioma, 'es') AS idioma
        FROM dispositivo d
        JOIN usuario u ON u.id = d.usuario_id
        WHERE d.usuario_id = $1::uuid
          AND d.activo
          AND u.notificaciones_activas
        "#,
    )
    .bind(para)
    .fetch_all(pool)
    .await
    .map_err(|causa| format!("could not read devices: {causa}"))?;

    if filas.is_empty() {
        return Ok(()); // La cuenta no tiene dispositivos o desactivó los avisos
    }

    let acceso = token_de_acceso(cuenta).await?;
    let cliente = crate::base::cliente_http().map_err(|causa| crate::fallos::texto(&causa))?;
    let url = URL_ENVIO.replace("{}", &cuenta.project_id);

    for (token, idioma) in filas {
        let (titulo, cuerpo) = texto_aviso(&idioma, tipo);
        let mensaje = serde_json::json!({
            "message": {
                "token": token,
                "notification": { "title": titulo, "body": cuerpo },
                "data": { "tipo": tipo, "receta_id": receta_id },
                "android": { "priority": "high" }
            }
        });

        let respuesta = cliente
            .post(&url)
            .bearer_auth(&acceso)
            .json(&mensaje)
            .send()
            .await
            .map_err(|causa| format!("could not reach FCM: {causa}"))?;

        if respuesta.status().is_success() {
            continue;
        }

        // Token muerto: se desactiva para no reintentar en cada aviso
        let estado = respuesta.status();
        let detalle = respuesta.text().await.unwrap_or_default();
        if estado.as_u16() == 404 || detalle.contains("UNREGISTERED") {
            marcar_inactivo(pool, &token).await;
        } else {
            dioxus::logger::tracing::warn!("FCM respondió {estado}: {detalle}");
        }
    }

    Ok(())
}

/// Desactiva un token que FCM ya no reconoce.
#[cfg(feature = "server")]
async fn marcar_inactivo(pool: &sqlx::PgPool, token: &str) {
    if let Err(causa) = sqlx::query("UPDATE dispositivo SET activo = false WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await
    {
        dioxus::logger::tracing::warn!("no se pudo desactivar un token push: {causa}");
    }
}

/// Texto del aviso según el idioma de la cuenta y el tipo de novedad.
#[cfg(feature = "server")]
fn texto_aviso(idioma: &str, tipo: &str) -> (&'static str, &'static str) {
    let ingles = idioma.eq_ignore_ascii_case("en");

    match (ingles, tipo) {
        (false, "respuesta") => ("Ratio", "Alguien respondió a tu comentario."),
        (false, "valoracion") => ("Ratio", "Alguien valoró tu receta."),
        (false, "version_derivada") => ("Ratio", "Alguien publicó una versión de tu receta."),
        (false, "reporte_resuelto") => {
            ("Ratio", "Moderación revisó un reporte sobre tu contenido.")
        }
        (true, "respuesta") => ("Ratio", "Someone replied to your comment."),
        (true, "valoracion") => ("Ratio", "Someone rated your recipe."),
        (true, "version_derivada") => ("Ratio", "Someone published a version of your recipe."),
        (true, "reporte_resuelto") => ("Ratio", "Moderation reviewed a report about your content."),
        _ => ("Ratio", "Tienes una novedad en tu actividad."),
    }
}

/// Token de acceso de Google, reutilizado mientras siga vigente.
///
/// # Errores
///
/// Devuelve el motivo si la clave no firma, si Google rechaza el JWT o si falta el token.
#[cfg(feature = "server")]
async fn token_de_acceso(cuenta: &CuentaServicio) -> Result<String, String> {
    let cache = ACCESO.get_or_init(|| Mutex::new(None));

    // Reutilizar el token si aún tiene margen
    {
        let guardado = cache.lock().map_err(|_| "token cache poisoned")?;
        if let Some((token, caduca)) = guardado.as_ref()
            && *caduca > Instant::now() + Duration::from_secs(MARGEN_TOKEN)
        {
            return Ok(token.clone());
        }
    }

    let jwt = firmar_jwt(cuenta)?;
    let cliente = crate::base::cliente_http().map_err(|causa| crate::fallos::texto(&causa))?;
    let respuesta = cliente
        .post(&cuenta.token_uri)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ])
        .send()
        .await
        .map_err(|causa| format!("could not reach Google OAuth: {causa}"))?;

    let estado = respuesta.status();
    let valor: serde_json::Value = respuesta
        .json()
        .await
        .map_err(|causa| format!("Google OAuth returned invalid JSON: {causa}"))?;
    if !estado.is_success() {
        return Err(format!("Google OAuth answered {estado}: {valor}"));
    }

    let token = valor
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Google OAuth without access_token")?
        .to_string();
    let segundos = valor
        .get("expires_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(VIDA_JWT);

    let mut guardado = cache.lock().map_err(|_| "token cache poisoned")?;
    *guardado = Some((
        token.clone(),
        Instant::now() + Duration::from_secs(segundos),
    ));

    Ok(token)
}

/// Firma el JWT que Google canjea por un token de acceso.
#[cfg(feature = "server")]
fn firmar_jwt(cuenta: &CuentaServicio) -> Result<String, String> {
    /// Reclamos que pide el flujo «service account» de Google.
    #[derive(Serialize)]
    struct Reclamos<'a> {
        /// Cuenta que firma.
        iss: &'a str,
        /// Permiso solicitado.
        scope: &'a str,
        /// Endpoint al que se canjea.
        aud: &'a str,
        /// Emitido en (segundos desde 1970).
        iat: u64,
        /// Caduca en (segundos desde 1970).
        exp: u64,
    }

    let ahora = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|causa| format!("bad system clock: {causa}"))?
        .as_secs();
    let reclamos = Reclamos {
        iss: &cuenta.client_email,
        scope: PERMISO,
        aud: &cuenta.token_uri,
        iat: ahora,
        exp: ahora + VIDA_JWT,
    };

    let mut cabecera = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    cabecera.kid = cuenta.private_key_id.clone();
    let llave = jsonwebtoken::EncodingKey::from_rsa_pem(cuenta.private_key.as_bytes())
        .map_err(|causa| format!("FCM private key is not valid RSA PEM: {causa}"))?;

    jsonwebtoken::encode(&cabecera, &reclamos, &llave)
        .map_err(|causa| format!("could not sign the FCM JWT: {causa}"))
}

// REGISTRO DEL DISPOSITIVO

/// Guarda o refresca el token push de este dispositivo para la cuenta de la sesión.
///
/// El token de FCM es único: si la misma persona entra en otro teléfono o reinstala, el mismo
/// token vuelve a apuntar a su cuenta. Así un cambio de cuenta en el mismo aparato no deja
/// avisos con el dueño anterior.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la plataforma no es válida o si la base falla.
#[server(endpoint = "registrar_dispositivo")]
pub async fn registrar_dispositivo(
    token: String,
    push_token: String,
    plataforma: String,
) -> Result<(), ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;
    let plataforma = plataforma_valida(&plataforma)?;

    let push_token = push_token.trim();
    if push_token.is_empty() || push_token.len() > TOKEN_MAXIMO {
        return Err(crate::fallos::datos(
            "El identificador del dispositivo no es válido.",
        ));
    }

    sqlx::query(
        r#"
        INSERT INTO dispositivo (usuario_id, token, plataforma, activo, visto_en)
        VALUES ($1::uuid, $2, $3, true, now())
        ON CONFLICT (token) DO UPDATE
        SET usuario_id = EXCLUDED.usuario_id,
            plataforma = EXCLUDED.plataforma,
            activo = true,
            visto_en = now()
        "#,
    )
    .bind(&cuenta)
    .bind(push_token)
    .bind(plataforma)
    .execute(pool)
    .con_plazo("tu dispositivo")
    .await?;

    Ok(())
}

/// Deja de avisar a este dispositivo (cierre de sesión o baja del permiso).
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si la escritura falla.
#[server(endpoint = "baja_dispositivo")]
pub async fn baja_dispositivo(token: String, push_token: String) -> Result<(), ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;

    sqlx::query("UPDATE dispositivo SET activo = false WHERE usuario_id = $1::uuid AND token = $2")
        .bind(&cuenta)
        .bind(push_token.trim())
        .execute(pool)
        .con_plazo("tu dispositivo")
        .await?;

    Ok(())
}

/// Normaliza la plataforma que manda el cliente (`android` o `ios`).
#[cfg(feature = "server")]
fn plataforma_valida(plataforma: &str) -> Result<&'static str, ServerFnError> {
    match plataforma.trim().to_ascii_lowercase().as_str() {
        "android" => Ok("android"),
        "ios" => Ok("ios"),
        _ => Err(crate::fallos::datos(
            "La plataforma del dispositivo no es válida.",
        )),
    }
}
