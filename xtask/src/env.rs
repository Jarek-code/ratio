//! Variables del servidor en formato `KEY=VALUE`, a partir de la cuenta de servicio de
//! Firebase que se descarga de la consola.

use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

/// Escribe (o imprime) las variables que necesita el servidor.
pub fn escribir(ruta_json: &str, salida: Option<&str>) -> Result<()> {
    let texto = std::fs::read_to_string(ruta_json)
        .with_context(|| format!("no se pudo leer {ruta_json}"))?;
    let cuenta: serde_json::Value =
        serde_json::from_str(&texto).context("el archivo no es un JSON válido")?;

    anyhow::ensure!(
        cuenta.get("type").and_then(|v| v.as_str()) == Some("service_account"),
        "el archivo no es una cuenta de servicio de Firebase"
    );

    let campo = |nombre: &str| -> Result<&str> {
        cuenta
            .get(nombre)
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .with_context(|| format!("la cuenta de servicio no trae «{nombre}»"))
    };

    // La clave privada va en una línea: los paneles manejan el `\n` escapado
    let clave = campo("private_key")?.replace('\n', "\\n");
    let token = cuenta
        .get("token_uri")
        .and_then(|v| v.as_str())
        .unwrap_or("https://oauth2.googleapis.com/token");

    let mut lineas = vec![
        "# Servidor".to_string(),
        "IP=0.0.0.0".to_string(),
        "PORT=8080".to_string(),
        "RUST_LOG=info".to_string(),
        String::new(),
        "# Base de datos (Supabase, cadena del session pooler)".to_string(),
        "DATABASE_URL=".to_string(),
        "MIGRACIONES_AL_ARRANCAR=false".to_string(),
        String::new(),
        "# Autenticación de Supabase".to_string(),
        "SUPABASE_URL=".to_string(),
        "SUPABASE_ANON_KEY=".to_string(),
        String::new(),
        "# Notificaciones push (cuenta de servicio de Firebase)".to_string(),
        format!("FCM_PROJECT_ID={}", campo("project_id")?),
        format!("FCM_CLIENT_EMAIL={}", campo("client_email")?),
    ];

    if let Some(id) = cuenta.get("private_key_id").and_then(|v| v.as_str()) {
        lineas.push(format!("FCM_PRIVATE_KEY_ID={id}"));
    }
    lineas.push(format!("FCM_TOKEN_URI={token}"));
    lineas.push(format!("FCM_PRIVATE_KEY={clave}"));

    let contenido = lineas.join("\n") + "\n";

    match salida {
        Some(destino) => {
            let mut archivo = std::fs::File::create(destino)
                .with_context(|| format!("no se pudo escribir {destino}"))?;
            archivo.write_all(contenido.as_bytes())?;
            restringir(Path::new(destino))?; // Lleva una clave privada
            println!("escrito {destino}");
        }
        None => print!("{contenido}"),
    }

    Ok(())
}

/// Deja el archivo legible solo por su dueño, donde el sistema lo permita.
fn restringir(ruta: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permisos = std::fs::metadata(ruta)?.permissions();
        permisos.set_mode(0o600);
        std::fs::set_permissions(ruta, permisos)?;
    }
    #[cfg(not(unix))]
    let _ = ruta;

    Ok(())
}
