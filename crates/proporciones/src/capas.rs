//! Reparto de la receta en capas y altura que ocupa cada una.

use crate::geometria::{Geometria, fraccion_por_volumen, radio, volumen};
use crate::tipos::Receta;

/// Un vaso nunca se llena hasta el borde: la superficie tiene que verse.
pub const LLENADO_MAX: f64 = 0.94;

/// Una capa de la receta, con su parte del volumen total.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Capa {
    pub nombre: String,
    pub ml: f64,
    /// Porcentaje del volumen total, incluida la dilución.
    pub porcentaje: f64,
    /// Mililitros acumulados debajo de esta capa.
    pub desde_ml: f64,
    pub graduacion: f64,
    pub dilucion: bool,
    /// Color del líquido, tal como venía en el componente.
    pub color: Option<String>,
    /// Color de la superficie, tal como venía en el componente.
    pub superficie: Option<String>,
}

/// Resumen de la receta ya repartida.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Capas {
    pub capas: Vec<Capa>,
    pub total_ml: f64,
    pub capacidad_ml: f64,
    /// Fracción de la capacidad ocupada, con el tope de llenado aplicado.
    pub llenado: f64,
    pub alcohol_ml: f64,
    pub dilucion_ml: f64,
    /// Alcohol por volumen, en %.
    pub abv: f64,
}

/// Una capa con su geometría resuelta.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DetalleCapa {
    pub capa: Capa,
    /// Fracción de altura por volumen real.
    pub fraccion_desde: f64,
    pub fraccion_hasta: f64,
    /// Las mismas fracciones recortadas al tope de llenado: son las que se dibujan.
    pub fraccion_dibujo_desde: f64,
    pub fraccion_dibujo_hasta: f64,
    /// Radios con el tope de llenado aplicado: sirven para dibujar.
    pub radio_abajo: f64,
    pub radio_arriba: f64,
    pub alto: f64,
    /// Volumen que devuelve la altura calculada. Debe coincidir con `ml`.
    pub recalculado_ml: f64,
    /// Error relativo de esa comprobación.
    pub error: f64,
    pub recortada: bool,
}

/// Todo lo medible de una receta ya dibujada.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Detalle {
    pub capas: Vec<DetalleCapa>,
    pub fraccion_llenado: f64,
    pub altura_ocupada: f64,
    pub error_max: f64,
}

/// Reparte la receta en capas y calcula el total, el alcohol y la dilución.
pub fn capas(receta: &Receta) -> Capas {
    let total_ml: f64 = receta.ingredientes.iter().map(|c| c.ml).sum();
    let alcohol_ml: f64 = receta
        .ingredientes
        .iter()
        .map(|c| c.ml * c.graduacion / 100.0)
        .sum();
    let dilucion_ml: f64 = receta
        .ingredientes
        .iter()
        .filter(|c| c.dilucion)
        .map(|c| c.ml)
        .sum();

    let capacidad_ml = receta.capacidad_ml.max(1e-9);
    let mut desde_ml = 0.0;
    let capas = receta
        .ingredientes
        .iter()
        .map(|c| {
            let capa = Capa {
                nombre: c.nombre.clone(),
                ml: c.ml,
                porcentaje: if total_ml > 0.0 {
                    c.ml / total_ml * 100.0
                } else {
                    0.0
                },
                desde_ml,
                graduacion: c.graduacion,
                dilucion: c.dilucion,
                color: c.color.clone(),
                superficie: c.superficie.clone(),
            };
            desde_ml += c.ml;
            capa
        })
        .collect();

    Capas {
        capas,
        total_ml,
        capacidad_ml,
        llenado: (total_ml / capacidad_ml).min(LLENADO_MAX),
        alcohol_ml,
        dilucion_ml,
        abv: if total_ml > 0.0 {
            alcohol_ml / total_ml * 100.0
        } else {
            0.0
        },
    }
}

/// Resuelve la geometría de cada capa y comprueba que la altura devuelve su volumen.
pub fn detalle(receta: &Receta) -> Detalle {
    let c = capas(receta);
    let g = Geometria::desde(&receta.vaso);
    let v_total = volumen(1.0, g.radio_fondo, g.radio_boca, g.altura);
    let v_llenado = v_total * c.llenado;

    let capas = c
        .capas
        .iter()
        .map(|capa| {
            let v_ini = v_total * capa.desde_ml / c.capacidad_ml;
            let v_fin = v_total * (capa.desde_ml + capa.ml) / c.capacidad_ml;
            let fraccion_desde = fraccion_por_volumen(v_ini, g.radio_fondo, g.radio_boca, g.altura);
            let fraccion_hasta = fraccion_por_volumen(v_fin, g.radio_fondo, g.radio_boca, g.altura);

            // Para dibujar se recorta al tope de llenado; para medir, no.
            let dibujo_desde =
                fraccion_por_volumen(v_ini.min(v_llenado), g.radio_fondo, g.radio_boca, g.altura);
            let dibujo_hasta =
                fraccion_por_volumen(v_fin.min(v_llenado), g.radio_fondo, g.radio_boca, g.altura);

            let recalculado_ml = (volumen(fraccion_hasta, g.radio_fondo, g.radio_boca, g.altura)
                - volumen(fraccion_desde, g.radio_fondo, g.radio_boca, g.altura))
                / v_total
                * c.capacidad_ml;

            DetalleCapa {
                capa: capa.clone(),
                fraccion_desde,
                fraccion_hasta,
                fraccion_dibujo_desde: dibujo_desde,
                fraccion_dibujo_hasta: dibujo_hasta,
                radio_abajo: radio(dibujo_desde, g.radio_fondo, g.radio_boca),
                radio_arriba: radio(dibujo_hasta, g.radio_fondo, g.radio_boca),
                alto: (dibujo_hasta - dibujo_desde).abs() * g.altura,
                recalculado_ml,
                error: if capa.ml > 0.0 {
                    (recalculado_ml - capa.ml).abs() / capa.ml
                } else {
                    0.0
                },
                recortada: v_fin > v_llenado + 1e-9,
            }
        })
        .collect::<Vec<_>>();

    let fraccion_llenado = capas.iter().map(|c| c.fraccion_hasta).fold(0.0, f64::max);

    Detalle {
        error_max: capas.iter().map(|c| c.error).fold(0.0, f64::max),
        altura_ocupada: fraccion_llenado * g.altura,
        fraccion_llenado,
        capas,
    }
}
