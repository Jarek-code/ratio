//! Dibujo del vaso: silueta, capas de líquido, escalas y sombra.
//!
//! Devuelve SVG como texto y deja el color en manos del CSS mediante nombres de clase
//! (`cup-cuerpo`, `capa`, `cup-sombra`…). La altura de cada capa no es lineal: sale de
//! `detalle()`, que la resuelve por volumen real del tronco de cono.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::capas::{LLENADO_MAX, Capa, DetalleCapa, capas, detalle};
use crate::geometria::{Geometria, fraccion_por_volumen, radio, volumen};
use crate::tipos::{Asa, Receta};

/// Alto fijo del lienzo, en unidades del dibujo.
const ALTO_LIENZO: f64 = 236.0;
/// Línea de apoyo: todas las tazas se dibujan desde aquí hacia arriba.
const BASE_Y: f64 = 196.0;
/// Colores de reserva del líquido cuando el ingrediente no trae uno válido.
const PALETA: [&str; 6] = [
    "#3E2415", "#B08050", "#8A6A4A", "#C9A227", "#6E8B74", "#A34C3B",
];

static CONTADOR: AtomicUsize = AtomicUsize::new(0);

/// Qué añadir al dibujo además de la silueta y las capas.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Opciones {
    /// Regla de mililitros a la izquierda del vaso.
    pub escalas: bool,
    /// Versión pequeña para listas: sin sombra, sin plato y sin brillo.
    pub mini: bool,
}

/// Lienzo ya resuelto: dónde cae cada borde del vaso y cuánto ocupa la caja.
struct Marco {
    /// Centro horizontal del vaso.
    cx: f64,
    /// Borde izquierdo del lienzo (puede ser negativo).
    x_caja: f64,
    /// Ancho total del lienzo.
    ancho: f64,
    /// Geometría interior del vaso.
    g: Geometria,
    /// Volumen interior completo, en unidades del lienzo.
    v_total: f64,
    /// Media anchura exterior de la boca.
    radio_boca: f64,
    /// Media anchura exterior del fondo.
    radio_fondo: f64,
    /// Borde exterior superior.
    y_exterior_arriba: f64,
    /// Borde exterior inferior.
    y_exterior_abajo: f64,
    /// Borde interior superior.
    y_interior_arriba: f64,
    /// Borde interior inferior.
    y_interior_abajo: f64,
    /// Altura de la elipse de la boca.
    boca_ry: f64,
    /// Línea de suelo real (el pie, si la copa tiene tallo).
    apoyo: f64,
    /// Origen vertical del `viewBox`.
    y_caja: f64,
    /// Alto del `viewBox`.
    alto_caja: f64,
    /// Asa del vaso, si tiene.
    asa: Option<Asa>,
}

impl Marco {
    /// Resuelve el lienzo a partir de la receta y de las opciones del dibujo.
    fn desde(receta: &Receta, opciones: &Opciones) -> Self {
        let vaso = &receta.vaso;
        let g = Geometria::desde(vaso);
        let cx = 62.0 + if opciones.escalas { 46.0 } else { 8.0 };
        let mut izquierda = cx - vaso.radio_boca - 8.0;
        let mut derecha = cx + vaso.radio_boca + if vaso.asa.is_some() { 34.0 } else { 12.0 };

        // El plato, la sombra y la regla de ml se salen del vaso hacia los lados: el lienzo
        // se abre lo necesario para que nada quede cortado.
        if vaso.plato && !opciones.mini {
            izquierda = izquierda.min(cx - (vaso.radio_boca + 26.0) - 4.0);
            derecha = derecha.max(cx + vaso.radio_boca + 26.0 + 4.0);
        }

        if !opciones.mini {
            izquierda = izquierda.min(cx + 4.0 - (vaso.radio_fondo + 30.0) - 4.0);
            derecha = derecha.max(cx + 4.0 + vaso.radio_fondo + 30.0 + 4.0);
        }

        if opciones.escalas {
            izquierda = izquierda.min(cx - vaso.radio_boca - 46.0);
        }

        let x_caja = izquierda.floor();
        let ancho = (derecha - x_caja).ceil();
        let base = BASE_Y - vaso.tallo;
        let y_exterior_abajo = base;
        let y_exterior_arriba = base - vaso.pared - vaso.altura - vaso.borde;
        let y_interior_abajo = base - vaso.pared;
        let y_interior_arriba = y_interior_abajo - vaso.altura;
        let boca_ry = (vaso.radio_boca * 0.13).max(4.0);
        let apoyo = if vaso.tallo > 0.0 {
            BASE_Y
        } else {
            y_exterior_abajo
        };
        let y_caja = (y_exterior_arriba - boca_ry - 3.0).max(0.0).round();
        // Margen inferior suficiente para que la sombra no quede cortada.
        let y_caja_fin = (apoyo + if opciones.mini { 6.0 } else { 24.0 })
            .min(ALTO_LIENZO)
            .round();
        let v_total = volumen(1.0, g.radio_fondo, g.radio_boca, g.altura);

        Self {
            cx,
            x_caja,
            ancho,
            g,
            v_total,
            radio_boca: vaso.radio_boca,
            radio_fondo: vaso.radio_fondo,
            y_exterior_arriba,
            y_exterior_abajo,
            y_interior_arriba,
            y_interior_abajo,
            boca_ry,
            apoyo,
            y_caja,
            alto_caja: y_caja_fin - y_caja,
            asa: vaso.asa,
        }
    }
}

/// Silueta interior y exterior del vaso, con la esquina del fondo redondeada.
fn paths(m: &Marco) -> (String, String) {
    let c = m.cx;
    let ro = (m.radio_fondo * 0.55).clamp(1.5, 6.0);
    let ri = (m.g.radio_fondo * 0.55).clamp(1.0, 6.0);
    let (ib, it) = (m.g.radio_fondo, m.g.radio_boca);
    let (ob, ot) = (m.radio_fondo, m.radio_boca);

    let interna = format!(
        "M {} {} Q {} {} {} {} L {} {} Q {} {} {} {} L {} {} A {} {} 0 0 1 {} {} Z",
        n(c - ib),
        n(m.y_interior_abajo - ri),
        n(c - ib),
        n(m.y_interior_abajo),
        n(c - ib + ri),
        n(m.y_interior_abajo),
        n(c + ib - ri),
        n(m.y_interior_abajo),
        n(c + ib),
        n(m.y_interior_abajo),
        n(c + ib),
        n(m.y_interior_abajo - ri),
        n(c + it),
        n(m.y_interior_arriba),
        n(it),
        n(m.boca_ry),
        n(c - it),
        n(m.y_interior_arriba),
    );
    let externa = format!(
        "M {} {} Q {} {} {} {} L {} {} Q {} {} {} {} L {} {} A {} {} 0 0 1 {} {} Z",
        n(c - ob),
        n(m.y_exterior_abajo - ro),
        n(c - ob),
        n(m.y_exterior_abajo),
        n(c - ob + ro),
        n(m.y_exterior_abajo),
        n(c + ob - ro),
        n(m.y_exterior_abajo),
        n(c + ob),
        n(m.y_exterior_abajo),
        n(c + ob),
        n(m.y_exterior_abajo - ro),
        n(c + ot),
        n(m.y_exterior_arriba),
        n(ot),
        n(m.boca_ry),
        n(c - ot),
        n(m.y_exterior_arriba),
    );

    (interna, externa)
}

/// Asa del vaso: dos trazos del mismo camino, uno oscuro y otro claro por dentro.
fn asa(m: &Marco) -> String {
    let Some(asa) = m.asa else {
        return String::new();
    };

    let (alto, suelo) = match asa {
        Asa::Grande => (20.0, 26.0),
        Asa::Chica => (14.0, 18.0),
    };
    let x = m.cx + m.radio_boca - 1.0;
    let y1 = m.y_exterior_arriba + alto;
    let y2 = m.y_exterior_abajo - suelo;
    let xm = x + asa.vuelo();
    let d = format!(
        "M {} {} C {} {} {} {} {} {}",
        n(x),
        n(y1),
        n(xm + 6.0),
        n(y1 + 2.0),
        n(xm + 6.0),
        n(y2 - 2.0),
        n(x - 2.0),
        n(y2),
    );

    format!("<path class=\"cup-asa-borde\" d=\"{d}\" /><path class=\"cup-asa\" d=\"{d}\" />")
}

/// Color del líquido: el del componente o uno de la paleta de reserva.
fn color_de(color: &Option<String>, indice: usize) -> String {
    color
        .as_deref()
        .filter(|valor| hex_valido(valor))
        .map(|valor| valor.to_string())
        .unwrap_or_else(|| PALETA[indice % PALETA.len()].to_string())
}

/// Color del menisco: el del catálogo o, si no hay, el de la capa aclarado.
fn superficie_de(color: &str, superficie: &Option<String>) -> String {
    superficie
        .as_deref()
        .filter(|valor| hex_valido(valor))
        .map(|valor| valor.to_string())
        .unwrap_or_else(|| aclarar(color, 0.12))
}

/// Aclara un color `#RRGGBB` acercando cada canal al blanco.
fn aclarar(hex: &str, factor: f64) -> String {
    let valor = u32::from_str_radix(&hex[1..7], 16).unwrap_or(0x8A6A4A);
    let canal = |desplazamiento: u32| {
        let c = ((valor >> desplazamiento) & 0xFF) as f64;
        (c + (255.0 - c) * factor).round() as u8
    };

    format!("#{:02X}{:02X}{:02X}", canal(16), canal(8), canal(0))
}

/// Comprueba que el color venga en el formato `#RRGGBB`.
fn hex_valido(s: &str) -> bool {
    s.len() == 7
        && s.starts_with('#')
        && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Capas de líquido apiladas desde el fondo, cada una con su menisco.
fn capas_svg(m: &Marco, detalle: &[DetalleCapa], id: usize, mini: bool) -> String {
    let mut salida = String::new();

    for (i, capa) in detalle.iter().enumerate() {
        if capa.capa.ml <= 0.0 || capa.alto < 0.4 {
            continue;
        }

        let color = color_de(&capa.capa.color, i);
        let superficie = superficie_de(&color, &capa.capa.superficie);
        let (r0, r1) = (capa.radio_abajo, capa.radio_arriba);
        let y0 = m.y_interior_abajo - capa.fraccion_dibujo_desde * m.g.altura;
        let y1 = m.y_interior_abajo - capa.fraccion_dibujo_hasta * m.g.altura;
        // El menisco se queda dentro de su capa: si la capa es fina, la elipse adelgaza.
        let ry = (r1 * 0.13).clamp(1.0, 5.5).min(capa.alto * 0.45);
        let clase = clase_de(&capa.capa);
        let trazo = format!(
            "M {} {} L {} {} L {} {} L {} {} Z",
            n(m.cx - r0),
            n(y0),
            n(m.cx + r0),
            n(y0),
            n(m.cx + r1),
            n(y1),
            n(m.cx - r1),
            n(y1),
        );
        let menisco = format!(
            "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{superficie}\" />",
            n(m.cx),
            n(y1),
            n(r1),
            n(ry),
        );

        salida.push_str(&format!(
            "<g class=\"{clase}\"><path d=\"{trazo}\" fill=\"{color}\" />{menisco}"
        ));

        if capa.capa.dilucion && !mini {
            let agua = format!("url(#agua-{id})");
            salida.push_str(&format!(
                "<path d=\"{trazo}\" fill=\"{agua}\" /><ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{agua}\" />",
                n(m.cx),
                n(y1),
                n(r1),
                n(ry),
            ));
        }

        salida.push_str("</g>");
    }

    salida
}

/// Clase CSS de una capa, con la marca de dilución cuando toca.
fn clase_de(capa: &Capa) -> &'static str {
    if capa.dilucion {
        "capa capa--dilucion"
    } else {
        "capa"
    }
}

/// Regla de mililitros a la izquierda del vaso.
fn escalas_svg(m: &Marco, capacidad_ml: f64) -> String {
    let mut salida = String::new();

    for fraccion in [0.25, 0.5, 0.75, 1.0] {
        let objetivo = m.v_total * fraccion * LLENADO_MAX;
        let f = fraccion_por_volumen(objetivo, m.g.radio_fondo, m.g.radio_boca, m.g.altura);
        let y = m.y_interior_abajo - f * m.g.altura;
        let r = radio(f, m.g.radio_fondo, m.g.radio_boca);
        let ml = (capacidad_ml * fraccion).round() as i64;

        salida.push_str(&format!(
            "<line class=\"cup-tick\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />\
<text class=\"cup-tick-txt\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{ml}</text>",
            n(m.cx - r - 12.0),
            n(y),
            n(m.cx + r + 6.0),
            n(y),
            n(m.cx - r - 16.0),
            n(y + 3.5),
        ));
    }

    salida
}

/// Dibuja el vaso de la receta y devuelve el SVG como texto.
pub fn svg(receta: &Receta, opciones: &Opciones) -> String {
    let c = capas(receta);
    let d = detalle(receta);
    let m = Marco::desde(receta, opciones);
    let (interna, externa) = paths(&m);
    let id = CONTADOR.fetch_add(1, Ordering::Relaxed) + 1;

    let cuerpo = capas_svg(&m, &d.capas, id, opciones.mini);
    let brillo = if opciones.mini {
        String::new()
    } else {
        format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#brillo-{id})\" />",
            n(m.cx - m.g.radio_boca),
            n(m.y_interior_arriba),
            n(m.g.radio_boca * 2.0),
            n(m.g.altura),
        )
    };
    let escalas = if opciones.escalas {
        escalas_svg(&m, c.capacidad_ml)
    } else {
        String::new()
    };
    let plato = if receta.vaso.plato && !opciones.mini {
        format!(
            "<ellipse class=\"cup-plato\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"8\" />",
            n(m.cx),
            n(m.y_exterior_abajo + 3.0),
            n(m.radio_boca + 26.0),
        )
    } else {
        String::new()
    };
    let tallo = if receta.vaso.tallo > 0.0 {
        format!(
            "<path class=\"cup-tallo\" d=\"M {} {} L {} {} L {} {} L {} {} Z\" />\
<ellipse class=\"cup-pie\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"4\" />",
            n(m.cx - 3.4),
            n(m.y_exterior_abajo),
            n(m.cx - 3.6),
            n(BASE_Y - 5.0),
            n(m.cx + 3.6),
            n(BASE_Y - 5.0),
            n(m.cx + 3.4),
            n(m.y_exterior_abajo),
            n(m.cx),
            n(BASE_Y - 3.0),
            n(m.radio_boca * 0.52),
        )
    } else {
        String::new()
    };
    let sombra = if opciones.mini {
        String::new()
    } else {
        format!(
            "<ellipse class=\"cup-sombra\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"9\" />",
            n(m.cx + 4.0),
            n(m.apoyo + 9.0),
            n(m.radio_fondo + 30.0),
        )
    };
    let clase = if opciones.mini {
        "cup-svg cup-svg--mini"
    } else {
        "cup-svg"
    };

    let mut salida = String::with_capacity(2048);
    salida.push_str(&format!(
        "<svg class=\"{clase}\" viewBox=\"{} {} {} {}\" width=\"{}\" height=\"{}\" role=\"img\" aria-label=\"Proporción de ingredientes de {}\">",
        n(m.x_caja),
        n(m.y_caja),
        n(m.ancho),
        n(m.alto_caja),
        n(m.ancho),
        n(m.alto_caja),
        escapar(&receta.nombre),
    ));
    salida.push_str(&format!(
        "<defs><clipPath id=\"clip-{id}\"><path d=\"{interna}\" /></clipPath>\
<pattern id=\"agua-{id}\" width=\"7\" height=\"7\" patternUnits=\"userSpaceOnUse\" patternTransform=\"rotate(45)\">\
<line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"7\" stroke=\"rgba(255,255,255,.55)\" stroke-width=\"1.3\" /></pattern>\
<linearGradient id=\"brillo-{id}\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"0\">\
<stop offset=\"0\" stop-color=\"#ffffff\" stop-opacity=\"0.14\" />\
<stop offset=\"0.32\" stop-color=\"#ffffff\" stop-opacity=\"0.02\" />\
<stop offset=\"0.78\" stop-color=\"#000000\" stop-opacity=\"0.10\" />\
<stop offset=\"1\" stop-color=\"#000000\" stop-opacity=\"0.22\" /></linearGradient></defs>"
    ));
    salida.push_str(&sombra);
    salida.push_str(&plato);
    salida.push_str(&tallo);
    salida.push_str(&asa(&m));
    salida.push_str(&format!("<path class=\"cup-cuerpo\" d=\"{externa}\" />"));
    salida.push_str(&format!(
        "<ellipse class=\"cup-boca\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" />",
        n(m.cx),
        n(m.y_exterior_arriba),
        n(m.radio_boca),
        n(m.boca_ry),
    ));
    salida.push_str(&format!("<g clip-path=\"url(#clip-{id})\">{cuerpo}{brillo}</g>"));
    salida.push_str(&format!("<path class=\"cup-contorno\" d=\"{externa}\" />"));
    salida.push_str(&format!("<path class=\"cup-pared\" d=\"{interna}\" />"));
    salida.push_str(&format!(
        "<ellipse class=\"cup-borde\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" />",
        n(m.cx),
        n(m.y_exterior_arriba),
        n(m.radio_boca),
        n(m.boca_ry),
    ));
    salida.push_str(&escalas);
    salida.push_str("</svg>");

    salida
}

/// Número con los mismos dígitos que imprime JavaScript para el mismo `f64`.
fn n(x: f64) -> String {
    if !x.is_finite() {
        return "0".to_string();
    }

    let texto = format!("{x}");

    if texto == "-0" { "0".to_string() } else { texto }
}

/// Escapa el texto que va dentro del SVG.
fn escapar(s: &str) -> String {
    let mut salida = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '&' => salida.push_str("&amp;"),
            '<' => salida.push_str("&lt;"),
            '>' => salida.push_str("&gt;"),
            '"' => salida.push_str("&quot;"),
            _ => salida.push(c),
        }
    }

    salida
}
