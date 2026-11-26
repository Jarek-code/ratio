//! Variables de entorno del binario.
//!
//! Se leen una sola vez, al arrancar, y el proceso falla con un mensaje claro si falta algo
//! imprescindible. La cadena de conexión nunca se registra completa: solo host y base.

/// Lo que el servidor necesita para arrancar.
#[derive(Debug, Clone)]
pub struct Config {
    /// Cadena de conexión a PostgreSQL (Supabase), con su contraseña.
    pub database_url: String,
    /// Si es `true`, aplica `migraciones/` al arrancar (ver `MIGRACIONES_AL_ARRANCAR`).
    pub migrar: bool,
    /// Dirección del proyecto de Supabase, para el acceso.
    pub supabase_url: Option<String>,
    /// Clave pública de Supabase (la `anon`), que viaja en la cabecera `apikey`.
    pub supabase_anon: Option<String>,
    /// Credenciales de la cuenta de servicio de FCM, ya leídas: el JSON completo o la ruta.
    ///
    /// Se acepta `FCM_CREDENTIALS_JSON` (contenido) o `FCM_CREDENTIALS` (ruta al archivo). En
    /// ningún caso se registra: solo se pasa a `api::push::configurar`.
    pub fcm_credenciales: Option<String>,
    /// Llave de APNs (push de iOS): contenido del `.p8`, Key ID, Team ID y si es sandbox.
    ///
    /// Se arma con `APNS_KEY_P8` (contenido) o `APNS_KEY_P8_PATH` (ruta), más `APNS_KEY_ID`,
    /// `APNS_TEAM_ID` y `APNS_SANDBOX` (por defecto `true`). Nunca se registra.
    pub apns: Option<(String, String, String, bool)>,
}

/// Arma el JSON de la cuenta de servicio con los campos sueltos del entorno.
///
/// Se usa cuando no hay `FCM_CREDENTIALS_JSON` ni `FCM_CREDENTIALS`: basta con el id del
/// proyecto, el correo de la cuenta de servicio y la clave privada. La clave privada puede
/// venir con `\n` literales, que es como se pega en la mayoría de los paneles.
fn credenciales_por_campos() -> Option<String> {
    let leer = |nombre: &str| {
        std::env::var(nombre)
            .ok()
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
    };

    let project_id = leer("FCM_PROJECT_ID")?;
    let client_email = leer("FCM_CLIENT_EMAIL")?;
    let private_key = leer("FCM_PRIVATE_KEY")?.replace("\\n", "\n");

    let mut cuenta = serde_json::json!({
        "type": "service_account",
        "project_id": project_id,
        "client_email": client_email,
        "private_key": private_key,
    });

    if let Some(valor) = leer("FCM_PRIVATE_KEY_ID") {
        cuenta["private_key_id"] = serde_json::Value::String(valor);
    }
    cuenta["token_uri"] = serde_json::Value::String(
        leer("FCM_TOKEN_URI").unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string()),
    );

    Some(cuenta.to_string())
}

impl Config {
    /// Lee el entorno. Devuelve el motivo del fallo en lugar de adivinarlo.
    ///
    /// Si la cadena no trae `sslmode` y el host no es local, se añade `sslmode=require`:
    /// Supabase rechaza las conexiones sin cifrar.
    pub fn desde_entorno() -> Result<Self, String> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            "missing DATABASE_URL: copy .env.example to .env locally, or set it in Render"
                .to_string()
        })?;

        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err("DATABASE_URL must start with postgres:// or postgresql://".to_string());
        }

        let es_local = database_url.contains("localhost") || database_url.contains("127.0.0.1");
        let database_url = if database_url.contains("sslmode=") || es_local {
            database_url
        } else {
            let separador = if database_url.contains('?') { '&' } else { '?' };
            format!("{database_url}{separador}sslmode=require")
        };

        let supabase_url = std::env::var("SUPABASE_URL")
            .ok()
            .filter(|v| !v.trim().is_empty());
        let supabase_anon = std::env::var("SUPABASE_ANON_KEY")
            .ok()
            .filter(|v| !v.trim().is_empty());

        // Credenciales push, en orden de preferencia: el JSON inline, el archivo indicado o los
        // campos por separado. Con los campos se arma el mismo JSON que espera `api::push`, de
        // modo que en el servidor de despliegue bastan tres variables normales.
        let fcm_credenciales = std::env::var("FCM_CREDENTIALS_JSON")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                std::env::var("FCM_CREDENTIALS")
                    .ok()
                    .filter(|v| !v.trim().is_empty())
                    .and_then(|ruta| std::fs::read_to_string(ruta).ok())
            })
            .or_else(credenciales_por_campos);

        // Llave de APNs: como FCM, sin ella el servidor arranca y iOS no recibe push
        let variable = |nombre: &str| std::env::var(nombre).ok().filter(|v| !v.trim().is_empty());
        let apns = variable("APNS_KEY_P8")
            .or_else(|| {
                variable("APNS_KEY_P8_PATH").and_then(|ruta| std::fs::read_to_string(ruta).ok())
            })
            .map(|p8| {
                let sandbox = variable("APNS_SANDBOX").is_none_or(|v| {
                    !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no")
                });
                (
                    p8,
                    variable("APNS_KEY_ID").unwrap_or_default(),
                    variable("APNS_TEAM_ID").unwrap_or_default(),
                    sandbox,
                )
            });

        Ok(Self {
            database_url,
            supabase_url,
            supabase_anon,
            fcm_credenciales,
            apns,
            migrar: std::env::var("MIGRACIONES_AL_ARRANCAR")
                .map(|v| {
                    matches!(
                        v.trim().to_ascii_lowercase().as_str(),
                        "1" | "true" | "si" | "sí"
                    )
                })
                .unwrap_or(false),
        })
    }

    /// Host y base, sin usuario ni contraseña: lo único que puede acabar en un log.
    pub fn destino(&self) -> String {
        let sin_esquema = self
            .database_url
            .split_once("://")
            .map(|(_, resto)| resto)
            .unwrap_or(&self.database_url);
        let sin_credenciales = sin_esquema
            .rsplit_once('@')
            .map(|(_, resto)| resto)
            .unwrap_or(sin_esquema);
        sin_credenciales
            .split(['?', '#'])
            .next()
            .unwrap_or(sin_credenciales)
            .to_string()
    }
}
