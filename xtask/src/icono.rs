//! Ícono de la aplicación: la taza de espresso que dibuja el motor, sobre un cuadrado
//! redondeado, repartida en las densidades del lanzador de Android.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// Lado del cuadrado redondeado, en píxeles.
const LADO: u32 = 1024;
/// Proporción del lienzo que ocupa la taza.
const PROPORCION_TAZA: f64 = 0.74;
/// Color de fondo del ícono (el cobre de la app).
const FONDO: &str = "#B9633B";
/// Exponente de la superelipse: 4 da el cuadrado redondeado continuo.
const EXPONENTE: f64 = 4.0;
/// Densidades del lanzador y su lado en píxeles.
const DENSIDADES: [(&str, u32); 5] = [
    ("mdpi", 48),
    ("hdpi", 72),
    ("xhdpi", 96),
    ("xxhdpi", 144),
    ("xxxhdpi", 192),
];

/// Escribe el ícono cuadrado y, si se pide, los mipmap del proyecto de Android.
pub fn escribir(res_android: Option<&str>) -> Result<()> {
    let raiz = raiz()?;
    let svg = svg_del_icono(&raiz)?;

    let cuadrado = raiz.join("target/icono-cuadrado.png");
    dibujar(&svg, LADO, &cuadrado)?;
    println!("ícono: {}", cuadrado.display());

    if let Some(res) = res_android {
        escribir_densidades(&svg, Path::new(res))?;
    }

    Ok(())
}

/// Reparte el ícono en cada densidad del lanzador, en el proyecto que generó `dx`.
fn escribir_densidades(svg: &str, res: &Path) -> Result<()> {
    for (nombre, lado) in DENSIDADES {
        let carpeta = res.join(format!("mipmap-{nombre}"));
        std::fs::create_dir_all(&carpeta)
            .with_context(|| format!("no se pudo crear {}", carpeta.display()))?;

        // El webp de la plantilla tiene prioridad con el mismo nombre
        let _ = std::fs::remove_file(carpeta.join("ic_launcher.webp"));
        dibujar(svg, lado, &carpeta.join("ic_launcher.png"))?;
    }

    // El ícono adaptativo de la plantilla también tiene prioridad en Android 8 o posterior
    for patron in [
        "mipmap-anydpi-v26/ic_launcher.xml",
        "drawable/ic_launcher_background.xml",
        "drawable-v24/ic_launcher_foreground.xml",
    ] {
        let _ = std::fs::remove_file(res.join(patron));
    }

    println!("mipmap escritos en {}", res.display());
    Ok(())
}

/// Dibuja el SVG en un PNG cuadrado del lado pedido, sin fondo alrededor del cuadrado.
fn dibujar(svg: &str, lado: u32, destino: &Path) -> Result<()> {
    let opciones = resvg::usvg::Options::default();
    let arbol =
        resvg::usvg::Tree::from_str(svg, &opciones).context("el SVG del ícono no es válido")?;

    let mut pixeles =
        resvg::tiny_skia::Pixmap::new(lado, lado).context("no se pudo reservar el lienzo")?;
    let escala = lado as f32 / arbol.size().width().max(arbol.size().height());
    resvg::render(
        &arbol,
        resvg::tiny_skia::Transform::from_scale(escala, escala),
        &mut pixeles.as_mut(),
    );

    pixeles
        .save_png(destino)
        .with_context(|| format!("no se pudo escribir {}", destino.display()))?;
    Ok(())
}

/// SVG del ícono: la taza del motor centrada sobre el cuadrado redondeado.
fn svg_del_icono(raiz: &Path) -> Result<String> {
    let salida = Command::new(env!("CARGO"))
        .args([
            "run",
            "-q",
            "-p",
            "proporciones",
            "--example",
            "taza_espresso",
        ])
        .current_dir(raiz)
        .output()
        .context("no se pudo ejecutar el ejemplo de la taza")?;

    if !salida.status.success() {
        anyhow::bail!(
            "el ejemplo de la taza falló: {}",
            String::from_utf8_lossy(&salida.stderr)
        );
    }

    let taza = String::from_utf8(salida.stdout).context("la taza no salió como texto")?;
    componer(&taza)
}

/// Envuelve el dibujo de la taza en el cuadrado redondeado con los colores de la app.
fn componer(taza: &str) -> Result<String> {
    let estilos = "\
  .cup-sombra { display: none; }
  .cup-plato { fill: #E8D6BF; stroke: #C9A98A; stroke-width: 1.2; }
  .cup-cuerpo { fill: #FFFFFF; }
  .cup-boca { fill: #EFE4D5; }
  .cup-contorno { fill: none; stroke: #7B5A3C; stroke-width: 2; }
  .cup-pared { fill: none; stroke: rgba(43, 22, 12, 0.35); stroke-width: 1.4; }
  .cup-borde { fill: none; stroke: #7B5A3C; stroke-width: 1.6; }
  .cup-asa-borde { fill: none; stroke: #7B5A3C; stroke-width: 11; stroke-linecap: round; }
  .cup-asa { fill: none; stroke: #F1E6D6; stroke-width: 7.5; stroke-linecap: round; }";

    let con_estilos = taza.replacen("><defs>", &format!("><style>{estilos}</style><defs>"), 1);

    // Del dibujo se toma su `viewBox` y su contenido, sin el contenedor
    let inicio = con_estilos
        .find('>')
        .context("el SVG de la taza no tiene etiqueta de apertura")?;
    let fin = con_estilos
        .rfind("</svg>")
        .context("el SVG de la taza no cierra")?;
    let contenido = &con_estilos[inicio + 1..fin];

    let view_box = con_estilos[..inicio]
        .split("viewBox=\"")
        .nth(1)
        .and_then(|resto| resto.split('"').next())
        .context("el SVG de la taza no declara viewBox")?;
    let numeros: Vec<f64> = view_box
        .split_whitespace()
        .filter_map(|valor| valor.parse().ok())
        .collect();
    anyhow::ensure!(
        numeros.len() == 4,
        "el viewBox de la taza no tiene cuatro números"
    );
    let (x, y, ancho, alto) = (numeros[0], numeros[1], numeros[2], numeros[3]);

    let lado = LADO as f64;
    let escala = lado * PROPORCION_TAZA / ancho;
    let traslado_x = (lado - ancho * escala) / 2.0 - x * escala;
    let traslado_y = (lado - alto * escala) / 2.0 - y * escala;

    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{LADO}" height="{LADO}" viewBox="0 0 {LADO} {LADO}">
  <path d="{fondo}" fill="{FONDO}"/>
  <g transform="translate({traslado_x:.3} {traslado_y:.3}) scale({escala:.5})">{contenido}</g>
</svg>
"#,
        fondo = superelipse()
    ))
}

/// Camino de un cuadrado redondeado continuo (superelipse) del tamaño del lienzo.
fn superelipse() -> String {
    let centro = LADO as f64 / 2.0;
    let radio = centro;
    let mut camino = String::new();

    for indice in 0..240 {
        let angulo = 2.0 * std::f64::consts::PI * indice as f64 / 240.0;
        let (seno, coseno) = angulo.sin_cos();
        let x = centro + radio * coseno.signum() * coseno.abs().powf(2.0 / EXPONENTE);
        let y = centro + radio * seno.signum() * seno.abs().powf(2.0 / EXPONENTE);

        let orden = if indice == 0 { "M" } else { "L" };
        camino.push_str(&format!("{orden} {x:.2} {y:.2} "));
    }

    camino.push('Z');
    camino
}

/// Raíz del espacio de trabajo, subiendo desde el ejecutable hasta el `Cargo.toml` mayor.
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
