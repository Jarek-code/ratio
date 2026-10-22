//! Dibujo del vaso: silueta, capas de líquido, escalas y etiquetas.
//!
//! Porta `CupChart` del prototipo en JS. Devuelve SVG como texto y conserva los
//! nombres de clase (`cup-cuerpo`, `capa`, `leg-row`…) para que el estilo siga
//! viviendo en CSS.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::capas::{LLENADO_MAX, capas, detalle};
use crate::geometria::{Geometria, fraccion_por_volumen, radio, volumen};
use crate::tipos::{Asa, Receta};

/// Alto fijo del lienzo, en unidades del dibujo.
const ALTO_LIENZO: f64 = 236.0;
/// Línea de apoyo: todas las tazas se dibujan desde aquí hacia arriba.
const BASE_Y: f64 = 196.0;

/// Colores de reserva cuando el ingrediente no trae uno válido.
const PALETA: [&str; 6] = [
    "#3E2415", "#B08050", "#8A6A4A", "#C9A227", "#6E8B74", "#A34C3B",
];

static CONTADOR: AtomicUsize = AtomicUsize::new(0);

/// Qué añadir al dibujo además de la silueta y las capas.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Opciones {
    pub etiquetas: bool,
    pub escalas: bool,
    pub mini: bool,
}

/// Lienzo ya resuelto: dónde cae cada borde del vaso y cuánto ocupa la caja.
struct Marco {
    cx: f64,
    ancho: f64,
    g: Geometria,
    v_total: f64,
    radio_boca: f64,
    radio_fondo: f64,
    y_exterior_arriba: f64,
    y_exterior_abajo: f64,
    y_interior_arriba: f64,
    y_interior_abajo: f64,
    boca_ry: f64,
    apoyo: f64,
    y_caja: f64,
    alto_caja: f64,
    asa: Option<Asa>,
}

impl Marco {
    fn desde(receta: &Receta, opciones: &Opciones) -> Self {
        let vaso = &receta.vaso;
        let g = Geometria::desde(vaso);
        let cx = 62.0 + if opciones.escalas { 46.0 } else { 8.0 };
        let ancho = (cx
            + vaso.radio_boca
            + if vaso.asa.is_some() { 34.0 } else { 12.0 }
            + if opciones.etiquetas { 150.0 } else { 0.0 })
        .ceil();
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
        let y_caja_fin = (apoyo + if opciones.mini { 6.0 } else { 18.0 })
            .min(ALTO_LIENZO)
            .round();
        let v_total = volumen(1.0, g.radio_fondo, g.radio_boca, g.altura);
        Self {
            cx,
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

/// Silueta interior y exterior. El radio de la esquina del fondo se adapta al
/// ancho: en un chupito el fondo es casi un punto.
fn paths(m: &Marco) -> (String, String) {
    let c = m.cx;
    let ro = (m.radio_fondo * 0.55).clamp(1.5, 6.0);
    let ri = (m.g.radio_fondo * 0.55).clamp(1.0, 6.0);
    let (ib, it) = (m.g.radio_fondo, m.g.radio_boca);
    let (ob, ot) = (m.radio_fondo, m.radio_boca);

    let interna = format!(
        "M {} {} Q {} {} {} {} L {} {} Q {} {} {} {} L {} {} L {} {} Z",
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
        n(c - it),
        n(m.y_interior_arriba),
    );
    let externa = format!(
        "M {} {} Q {} {} {} {} L {} {} Q {} {} {} {} L {} {} L {} {} Z",
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
        n(c - ot),
        n(m.y_exterior_arriba),
    );
    (interna, externa)
}

fn asa(m: &Marco) -> String {
    let Some(asa) = m.asa else {
        return String::new();
    };
    let alto = match asa {
        Asa::Grande => 20.0,
        Asa::Chica => 14.0,
    };
    let suelo = match asa {
        Asa::Grande => 26.0,
        Asa::Chica => 18.0,
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

/// Dibuja la receta: silueta, capas y lo que pidan las opciones.
pub fn svg(receta: &Receta, opciones: &Opciones) -> String {
    let c = capas(receta);
    let d = detalle(receta);
    let m = Marco::desde(receta, opciones);
    let (interna, externa) = paths(&m);
    let id = CONTADOR.fetch_add(1, Ordering::Relaxed) + 1;
    let altura = m.g.altura;

    let mut cuerpo = String::new();
    for (i, capa) in d.capas.iter().enumerate() {
        if capa.capa.ml <= 0.0 || capa.alto < 0.4 {
            continue;
        }
        let color = color_de(&capa.capa.color, i);
        let superficie = aclarar(&color, 0.18);
        let (r0, r1) = (capa.radio_abajo, capa.radio_arriba);
        let y0 = m.y_interior_abajo - capa.fraccion_dibujo_desde * altura;
        let y1 = m.y_interior_abajo - capa.fraccion_dibujo_hasta * altura;
        let ry = (r1 * 0.13).clamp(2.2, 5.5);
        let clase = if capa.capa.dilucion {
            "capa capa--dilucion"
        } else {
            "capa"
        };
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
        cuerpo.push_str(&format!(
            "<g class=\"{clase}\"><path d=\"{trazo}\" fill=\"{color}\" /><ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{superficie}\" />",
            n(m.cx),
            n(y1),
            n(r1),
            n(ry),
        ));
        if capa.capa.dilucion {
            cuerpo.push_str(&format!(
                "<path d=\"{trazo}\" fill=\"url(#dil-{id})\" /><ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"url(#dil-{id})\" />",
                n(m.cx),
                n(y1),
                n(r1),
                n(ry),
            ));
        }
        cuerpo.push_str("</g>");
    }

    let brillo = if opciones.mini {
        String::new()
    } else {
        format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#sheen-{id})\" />",
            n(m.cx - m.g.radio_boca),
            n(m.y_interior_arriba),
            n(m.g.radio_boca * 2.0),
            n(altura),
        )
    };

    let mut escalas = String::new();
    if opciones.escalas {
        for f in [0.25, 0.5, 0.75, 1.0] {
            let v = m.v_total * f * LLENADO_MAX;
            let fr = fraccion_por_volumen(v, m.g.radio_fondo, m.g.radio_boca, altura);
            let y = m.y_interior_abajo - fr * altura;
            let r = radio(fr, m.g.radio_fondo, m.g.radio_boca);
            let ml = (c.capacidad_ml * f).round() as i64;
            escalas.push_str(&format!(
                "<line class=\"cup-tick\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" /><text class=\"cup-tick-txt\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{ml}</text>",
                n(m.cx - r - 12.0),
                n(y),
                n(m.cx + r + 6.0),
                n(y),
                n(m.cx - r - 16.0),
                n(y + 3.5),
            ));
        }
    }

    let mut etiquetas = String::new();
    if opciones.etiquetas {
        // Umbral proporcional: 15 unidades en un vaso de 150 y 6 en una taza baja de 40,
        // para que las tazas pequeñas no se queden sin etiquetas.
        let alto_minimo = (m.g.altura * 0.10).max(6.0);

        for (i, capa) in d.capas.iter().enumerate() {
            if capa.capa.ml <= 0.0 || capa.alto < alto_minimo {
                continue;
            }
            let color = color_de(&capa.capa.color, i);
            let rm = (capa.radio_abajo + capa.radio_arriba) / 2.0;
            let y = m.y_interior_abajo - capa.fraccion_dibujo_hasta * altura;
            let x1 = m.cx + rm;
            let x2 = x1.max(m.cx + m.g.radio_boca * 0.55) + 14.0;
            etiquetas.push_str(&format!(
                "<g class=\"cup-label\"><path d=\"M {} {} L {} {}\" /><circle cx=\"{}\" cy=\"{}\" r=\"2.6\" fill=\"{color}\" stroke=\"rgba(0,0,0,.35)\" /><text x=\"{}\" y=\"{}\"><tspan class=\"cup-label-n\">{}</tspan><tspan class=\"cup-label-v\" dx=\"6\">{} ml · {}%</tspan></text></g>",
                n(x1),
                n(y),
                n(x2),
                n(y),
                n(x1),
                n(y),
                n(x2 + 6.0),
                n(y + 3.6),
                escapar(&capa.capa.nombre),
                n(capa.capa.ml),
                capa.capa.porcentaje.round() as i64,
            ));
        }
    }

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
            "<path class=\"cup-tallo\" d=\"M {} {} L {} {} L {} {} L {} {} Z\" /><ellipse class=\"cup-pie\" cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"4\" />",
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
        "<svg class=\"{clase}\" viewBox=\"0 {} {} {}\" width=\"{}\" height=\"{}\" role=\"img\" aria-label=\"Proporción de ingredientes de {}\">",
        n(m.y_caja),
        n(m.ancho),
        n(m.alto_caja),
        n(m.ancho),
        n(m.alto_caja),
        escapar(&receta.nombre),
    ));
    salida.push_str(&format!(
        "<defs><clipPath id=\"clip-{id}\"><path d=\"{interna}\" /></clipPath>\
<pattern id=\"dil-{id}\" width=\"7\" height=\"7\" patternUnits=\"userSpaceOnUse\" patternTransform=\"rotate(45)\">\
<line x1=\"0\" y1=\"0\" x2=\"0\" y2=\"7\" stroke=\"rgba(255,255,255,.55)\" stroke-width=\"1.3\" /></pattern>\
<linearGradient id=\"sheen-{id}\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"0\">\
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
    salida.push_str(&format!(
        "<g clip-path=\"url(#clip-{id})\">{cuerpo}{brillo}</g>"
    ));
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
    salida.push_str(&etiquetas);
    salida.push_str("</svg>");
    salida
}

/// Leyenda de ingredientes: color, cantidad, porcentaje y graduación.
pub fn leyenda(receta: &Receta, compacta: bool) -> String {
    let c = capas(receta);
    let mut filas = String::new();
    for (i, capa) in c.capas.iter().enumerate() {
        let color = color_de(&capa.color, i);
        let superficie = aclarar(&color, 0.18);
        let clase = if capa.dilucion {
            "leg-row leg-row--dilucion"
        } else {
            "leg-row"
        };
        let etiqueta = if capa.dilucion {
            " <span class=\"etq-dilucion\">dilución</span>"
        } else {
            ""
        };
        let grado = if capa.graduacion > 0.0 {
            format!(" · {}°", n(capa.graduacion))
        } else {
            String::new()
        };
        let barra = if compacta {
            String::new()
        } else {
            format!(
                "<span class=\"leg-bar\"><i style=\"width:{}%;background:{superficie}\"></i></span>",
                n(capa.porcentaje),
            )
        };
        filas.push_str(&format!(
            "<li class=\"{clase}\"><span class=\"leg-chip\" style=\"background:{color};box-shadow:inset 0 0 0 1px rgba(255,255,255,.12)\"></span><span class=\"leg-nombre\">{}{etiqueta}</span>{barra}<span class=\"leg-val\">{} ml<small>{}%{grado}</small></span></li>",
            escapar(&capa.nombre),
            n(capa.ml),
            capa.porcentaje.round() as i64,
        ));
    }
    let total = if compacta {
        String::new()
    } else {
        format!(
            "<li class=\"leg-row leg-total\"><span class=\"leg-chip leg-chip--total\"></span><span class=\"leg-nombre\">Total en taza</span><span class=\"leg-bar\"></span><span class=\"leg-val\">{} ml<small>{}% del vaso</small></span></li>",
            n(c.total_ml),
            (c.llenado * 100.0).round() as i64,
        )
    };
    let clase = if compacta {
        "leyenda leyenda--compacta"
    } else {
        "leyenda"
    };
    format!("<ul class=\"{clase}\">{filas}{total}</ul>")
}

/// Barra apilada de una sola línea, para listas y tarjetas.
pub fn barra(receta: &Receta, alto: u32) -> String {
    let c = capas(receta);
    let tramos: String = c
        .capas
        .iter()
        .enumerate()
        .filter(|(_, capa)| capa.ml > 0.0)
        .map(|(i, capa)| {
            format!(
                "<i style=\"flex:{};background:{}\" title=\"{}\"></i>",
                n(capa.ml),
                color_de(&capa.color, i),
                escapar(&capa.nombre),
            )
        })
        .collect();
    format!("<span class=\"barra-stack\" style=\"height:{alto}px\">{tramos}</span>")
}

fn color_de(color: &Option<String>, indice: usize) -> String {
    color
        .as_deref()
        .filter(|c| hex_valido(c))
        .map(str::to_string)
        .unwrap_or_else(|| PALETA[indice % PALETA.len()].to_string())
}

fn hex_valido(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

/// Aclara un color mezclándolo con blanco: la superficie de la capa.
fn aclarar(hex: &str, factor: f64) -> String {
    let v = u32::from_str_radix(&hex[1..7], 16).unwrap_or(0x8A6A4A);
    let canal = |desplazamiento: u32| {
        let c = ((v >> desplazamiento) & 0xFF) as f64;
        (c + (255.0 - c) * factor).round() as u8
    };
    format!("#{:02X}{:02X}{:02X}", canal(16), canal(8), canal(0))
}

fn escapar(s: &str) -> String {
    let mut salida = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => salida.push_str("&amp;"),
            '<' => salida.push_str("&lt;"),
            '>' => salida.push_str("&gt;"),
            '"' => salida.push_str("&quot;"),
            '\'' => salida.push_str("&#39;"),
            _ => salida.push(ch),
        }
    }
    salida
}

/// Número para el SVG: dos decimales y sin ceros de relleno.
fn n(x: f64) -> String {
    if !x.is_finite() {
        return "0".to_string();
    }
    let texto = format!("{x:.2}");
    let texto = texto.trim_end_matches('0').trim_end_matches('.');
    if texto.is_empty() || texto == "-0" {
        "0".to_string()
    } else {
        texto.to_string()
    }
}
