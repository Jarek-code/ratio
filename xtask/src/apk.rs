//! Compilación del APK para un teléfono: las dos arquitecturas, el ícono, la firma y el
//! archivo final en `target/ratio-android.apk`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::icono::DENSIDADES;

/// Arquitectura de los teléfonos actuales.
const ABI_64: &str = "aarch64-linux-android";
/// Arquitectura de los teléfonos y televisores de 32 bits.
const ABI_32: &str = "armv7-linux-androideabi";
/// Nivel mínimo de API, igual que el `minSdk` del proyecto.
const API: &str = "24";
/// Nombre de la clave local, que se crea la primera vez.
const CLAVE: &str = "ratio-release.jks";
/// Contraseña por omisión de la clave del proyecto.
const CLAVE_CLAVE: &str = "ratio-release";
/// Alias por omisión dentro de la clave.
const CLAVE_ALIAS: &str = "ratio";

/// Compila, firma y deja el APK listo para instalar.
pub fn compilar() -> Result<()> {
    let raiz = raiz()?;
    let movil = raiz.join("apps/movil");
    let ndk = variable("ANDROID_NDK_HOME")?;
    let sdk = variable("ANDROID_HOME").or_else(|_| variable("ANDROID_SDK_ROOT"))?;

    // Restos de la compilación anterior: dos íconos con el mismo nombre rompen Gradle
    limpiar(&raiz);

    for abi in [ABI_64, ABI_32] {
        println!("Compilando para {abi}…");
        let mut orden = Command::new("dx");
        orden
            .args([
                "build",
                "--platform",
                "android",
                "--release",
                "--target",
                abi,
            ])
            .current_dir(&movil);

        // `ring` compila C: necesita el compilador del NDK para 32 bits
        if abi == ABI_32 {
            let bin = Path::new(&ndk).join("toolchains/llvm/prebuilt/linux-x86_64/bin");
            orden.env(
                "CC_armv7_linux_androideabi",
                bin.join(format!("armv7a-linux-androideabi{API}-clang")),
            );
            orden.env("AR_armv7_linux_androideabi", bin.join("llvm-ar"));
        }

        let salida = orden.output().context("no se pudo ejecutar dx")?;
        anyhow::ensure!(
            salida.status.success(),
            "dx falló para {abi}: {}",
            String::from_utf8_lossy(&salida.stderr)
        );
    }

    let proyecto = proyecto_de_gradle(&raiz)?;
    let res = proyecto.join("app/src/main/res");
    crate::icono::escribir(Some(&res.to_string_lossy()))?;

    desactivar_r8(&proyecto)?;
    numero_de_version(&proyecto, &movil)?;

    println!("Armando el APK…");
    if let Err(causa) = armar(&proyecto) {
        // `dx` vuelve a escribir su ícono de plantilla en cada compilación: si apareció
        // junto al nuestro, Gradle ve dos recursos con el mismo nombre. Se retira y se repite.
        println!("Gradle se quejó ({causa}); se retira el ícono de la plantilla y se repite");
        quitar_plantilla(&res);
        armar(&proyecto)?;
    }

    let libre = proyecto.join("app/build/outputs/apk/release/app-release-unsigned.apk");
    anyhow::ensure!(libre.is_file(), "Gradle no dejó el APK sin firmar");

    let salida = raiz.join("target/ratio-android.apk");
    firmar(&sdk, &libre, &salida)?;

    let tamaño = std::fs::metadata(&salida)
        .map(|meta| meta.len() / (1024 * 1024))
        .unwrap_or(0);
    println!();
    println!("APK: {} ({tamaño} MB)", salida.display());
    Ok(())
}

/// Ejecuta Gradle para armar el APK de release.
fn armar(proyecto: &Path) -> Result<()> {
    let salida = Command::new("./gradlew")
        .args(["--no-daemon", ":app:assembleRelease"])
        .current_dir(proyecto)
        .output()
        .context("no se pudo ejecutar Gradle")?;

    anyhow::ensure!(
        salida.status.success(),
        "{}",
        String::from_utf8_lossy(&salida.stderr)
            .lines()
            .find(|linea| linea.contains("What went wrong") || linea.contains("Error:"))
            .unwrap_or("sin detalle")
    );

    Ok(())
}

/// Borra el ícono de la plantilla de `dx` de todas las densidades.
fn quitar_plantilla(res: &Path) {
    for (nombre, _) in DENSIDADES {
        let _ = std::fs::remove_file(res.join(format!("mipmap-{nombre}/ic_launcher.webp")));
        let _ = std::fs::remove_file(res.join(format!("mipmap-{nombre}/ic_launcher.png")));
    }
}

/// Firma el APK con la clave del proyecto, creándola si es la primera vez.
fn firmar(sdk: &str, libre: &Path, destino: &Path) -> Result<()> {
    let raiz = raiz()?;
    let herramientas = Path::new(sdk).join("build-tools/34.0.0");

    // En integración continua la clave llega por entorno; si no, se usa la local
    let clave = std::env::var("RATIO_KEYSTORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| raiz.join("target").join(CLAVE));
    let contrasena =
        std::env::var("RATIO_KEYSTORE_PASSWORD").unwrap_or_else(|_| CLAVE_CLAVE.to_string());
    let alias = std::env::var("RATIO_KEY_ALIAS").unwrap_or_else(|_| CLAVE_ALIAS.to_string());

    if !clave.is_file() {
        println!("Generando la clave de firma en {}", clave.display());
        let estado = Command::new("keytool")
            .args([
                "-genkeypair",
                "-keystore",
                &clave.to_string_lossy(),
                "-alias",
                &alias,
                "-keyalg",
                "RSA",
                "-keysize",
                "4096",
                "-validity",
                "10000",
                "-storepass",
                &contrasena,
                "-keypass",
                &contrasena,
                "-dname",
                "CN=Ratio, OU=ITT, O=Ratio, L=Tijuana, ST=Baja California, C=MX",
            ])
            .status()
            .context("no se pudo ejecutar keytool")?;
        anyhow::ensure!(estado.success(), "keytool no pudo crear la clave");
    }

    let alineado = raiz.join("target/ratio-android-alineado.apk");
    let estado = Command::new(herramientas.join("zipalign"))
        .args(["-f", "-p", "4"])
        .arg(libre)
        .arg(&alineado)
        .status()
        .context("no se pudo ejecutar zipalign")?;
    anyhow::ensure!(estado.success(), "zipalign falló");

    let estado = Command::new(herramientas.join("apksigner"))
        .args(["sign", "--ks"])
        .arg(&clave)
        .args(["--ks-pass", &format!("pass:{contrasena}")])
        .args(["--key-pass", &format!("pass:{contrasena}")])
        .arg("--ks-key-alias")
        .arg(&alias)
        .arg("--out")
        .arg(destino)
        .arg(&alineado)
        .status()
        .context("no se pudo ejecutar apksigner")?;
    anyhow::ensure!(estado.success(), "apksigner falló");

    let _ = std::fs::remove_file(&alineado);
    Ok(())
}

/// `dx` deja R8 activo en release; renombraría clases que el lado nativo usa por nombre.
fn desactivar_r8(proyecto: &Path) -> Result<()> {
    let ruta = proyecto.join("app/build.gradle.kts");
    let texto = std::fs::read_to_string(&ruta).context("no se pudo leer build.gradle.kts")?;

    let cambiado = texto.replace(
        "getByName(\"release\") {\n            isMinifyEnabled = true",
        "getByName(\"release\") {\n            isMinifyEnabled = false",
    );
    std::fs::write(&ruta, cambiado).context("no se pudo escribir build.gradle.kts")?;
    Ok(())
}

/// Pone el `versionCode` a partir de la versión (0.2.0 → 200) para poder instalar encima.
fn numero_de_version(proyecto: &Path, movil: &Path) -> Result<()> {
    let manifiesto = std::fs::read_to_string(movil.join("Cargo.toml"))?;
    let version = manifiesto
        .lines()
        .find_map(|linea| {
            linea
                .strip_prefix("version = \"")
                .and_then(|r| r.strip_suffix('"'))
        })
        .context("el cliente no declara versión")?;

    let partes: Vec<u64> = version.split('.').filter_map(|p| p.parse().ok()).collect();
    anyhow::ensure!(
        partes.len() >= 2,
        "la versión no tiene el formato mayor.menor.parche"
    );
    let codigo = partes[0] * 10_000 + partes[1] * 100 + partes.get(2).copied().unwrap_or(0);

    let ruta = proyecto.join("app/build.gradle.kts");
    let texto = std::fs::read_to_string(&ruta)?;
    let actualizado = texto
        .lines()
        .map(|linea| {
            if linea.trim_start().starts_with("versionCode = ") {
                "        versionCode = ".to_string() + &codigo.to_string()
            } else {
                linea.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&ruta, actualizado + "\n")?;

    println!("versión {version}, versionCode {codigo}");
    Ok(())
}

/// Quita íconos de compilaciones anteriores para que Gradle no vea recursos duplicados.
fn limpiar(raiz: &Path) {
    let Ok(entradas) = std::fs::read_dir(raiz.join("target/dx")) else {
        return;
    };

    for entrada in entradas.flatten() {
        let profundidad = walkdir(&entrada.path(), 0);
        for ruta in profundidad {
            if ruta
                .file_name()
                .is_some_and(|nombre| nombre == "ic_launcher.png")
            {
                let _ = std::fs::remove_file(&ruta);
            }
        }
    }
}

/// Archivos de un directorio hasta cinco niveles, sin seguir enlaces.
fn walkdir(ruta: &Path, nivel: usize) -> Vec<PathBuf> {
    let mut encontrados = Vec::new();
    if nivel > 12 {
        return encontrados;
    }
    let Ok(entradas) = std::fs::read_dir(ruta) else {
        return encontrados;
    };

    for entrada in entradas.flatten() {
        let camino = entrada.path();
        if camino.is_dir() {
            encontrados.extend(walkdir(&camino, nivel + 1));
        } else {
            encontrados.push(camino);
        }
    }

    encontrados
}

/// Raíz del proyecto de Gradle que generó `dx` para la variante de release.
fn proyecto_de_gradle(raiz: &Path) -> Result<PathBuf> {
    walkdir(&raiz.join("target/dx"), 0)
        .into_iter()
        .find(|ruta| ruta.file_name().is_some_and(|nombre| nombre == "gradlew"))
        .and_then(|ruta| {
            let texto = ruta.to_string_lossy();
            texto
                .contains("/release/")
                .then(|| ruta.parent().map(Path::to_path_buf))?
        })
        .context("dx no generó el proyecto de Gradle")
}

/// Valor de una variable de entorno obligatoria.
fn variable(nombre: &str) -> Result<String> {
    std::env::var(nombre).with_context(|| format!("falta la variable {nombre}"))
}

/// Raíz del espacio de trabajo.
fn raiz() -> Result<PathBuf> {
    let mut actual = std::env::current_dir().context("no se pudo leer el directorio actual")?;
    loop {
        if actual.join("Cargo.toml").is_file() && actual.join("apps/movil").is_dir() {
            return Ok(actual);
        }
        if !actual.pop() {
            anyhow::bail!("no se encontró la raíz de Ratio");
        }
    }
}
