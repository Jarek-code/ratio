//! Herramientas del proyecto, en Rust.
//!
//! ```text
//! cargo run -p xtask -- icono                escribe el ícono cuadrado de la app
//! cargo run -p xtask -- icono --android DIR  además lo reparte en las densidades de Android
//! cargo run -p xtask -- env-render <json>    imprime las variables del servidor en KEY=VALUE
//! cargo run -p xtask -- apk                  compila y firma el APK para teléfono
//! ```

mod apk;
mod env;
mod icono;

fn main() -> anyhow::Result<()> {
    let argumentos: Vec<String> = std::env::args().skip(1).collect();

    match argumentos.first().map(String::as_str) {
        Some("icono") => {
            let android = argumentos
                .iter()
                .position(|arg| arg == "--android")
                .and_then(|posicion| argumentos.get(posicion + 1));
            icono::escribir(android.map(String::as_str))?;
        }
        Some("env-render") => {
            let Some(ruta) = argumentos.get(1) else {
                anyhow::bail!("falta la ruta del JSON de la cuenta de servicio");
            };
            env::escribir(ruta, argumentos.get(2).map(String::as_str))?;
        }
        Some("apk") => apk::compilar()?,
        Some(otra) => anyhow::bail!("orden desconocida: {otra}"),
        None => println!("{USO}"),
    }

    Ok(())
}

/// Ayuda breve.
const USO: &str = "\
Herramientas de Ratio

  cargo run -p xtask -- icono [--android <res>]    ícono de la app
  cargo run -p xtask -- env-render <json> [salida] variables del servidor
  cargo run -p xtask -- apk                        APK firmado para teléfono
";
