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
}

impl Config {
    /// Lee el entorno. Devuelve el motivo del fallo en lugar de adivinarlo.
    ///
    /// Si la cadena no trae `sslmode` y el host no es local, se añade `sslmode=require`:
    /// Supabase rechaza las conexiones sin cifrar.
    pub fn desde_entorno() -> Result<Self, String> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            "falta DATABASE_URL; copia .env.example a .env en local, o configúrala en Render".to_string()
        })?;

        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err("DATABASE_URL debe empezar por postgres:// o postgresql://".to_string());
        }

        let es_local = database_url.contains("localhost") || database_url.contains("127.0.0.1");
        let database_url = if database_url.contains("sslmode=") || es_local {
            database_url
        } else {
            let separador = if database_url.contains('?') { '&' } else { '?' };
            format!("{database_url}{separador}sslmode=require")
        };

        let supabase_url = std::env::var("SUPABASE_URL").ok().filter(|v| !v.trim().is_empty());
        let supabase_anon = std::env::var("SUPABASE_ANON_KEY").ok().filter(|v| !v.trim().is_empty());

        Ok(Self {
            database_url,
            supabase_url,
            supabase_anon,
            migrar: std::env::var("MIGRACIONES_AL_ARRANCAR")
                .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "si" | "sí"))
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
