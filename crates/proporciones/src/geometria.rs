//! Geometría del vaso y conversión entre volumen y altura.
//!
//! El vaso es un tronco de cono. La altura de una capa no se reparte en línea recta: se
//! invierte el volumen, de modo que 30 ml ocupan la misma altura relativa en una taza
//! pequeña que en un vaso alto.

use crate::tipos::Vaso;

/// Geometría útil del vaso: radios interiores y altura.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometria {
    pub radio_fondo: f64,
    pub radio_boca: f64,
    pub altura: f64,
}

impl Geometria {
    pub fn desde(vaso: &Vaso) -> Self {
        Self {
            radio_fondo: (vaso.radio_fondo - vaso.pared).max(0.0),
            radio_boca: (vaso.radio_boca - vaso.pared).max(0.0),
            altura: vaso.altura,
        }
    }
}

/// Radio del tronco de cono a una fracción de altura.
pub fn radio(fraccion: f64, radio_fondo: f64, radio_boca: f64) -> f64 {
    radio_fondo + (radio_boca - radio_fondo) * fraccion
}

/// Volumen del tronco de cono hasta la fracción `h`, con `r` el radio a esa altura.
///
/// `V(h) = π·h/3 · (r₀² + r₀·r + r²)`
pub fn volumen(h: f64, radio_fondo: f64, radio_boca: f64, altura: f64) -> f64 {
    let r = radio(h, radio_fondo, radio_boca);
    std::f64::consts::PI * h * altura / 3.0 * (radio_fondo * radio_fondo + radio_fondo * r + r * r)
}

/// Fracción de altura que ocupa un volumen, resuelta por bisección.
pub fn fraccion_por_volumen(objetivo: f64, radio_fondo: f64, radio_boca: f64, altura: f64) -> f64 {
    let (mut bajo, mut alto) = (0.0, 1.0);
    for _ in 0..34 {
        let medio = (bajo + alto) / 2.0;
        if volumen(medio, radio_fondo, radio_boca, altura) < objetivo {
            bajo = medio;
        } else {
            alto = medio;
        }
    }
    (bajo + alto) / 2.0
}
