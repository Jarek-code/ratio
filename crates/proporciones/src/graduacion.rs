//! Graduación de la bebida ya mezclada.

use crate::capas::capas;
use crate::tipos::Receta;

/// Etiqueta que muestra la ficha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nivel {
    Cero,
    Bajo,
    Medio,
    Alto,
}

/// Graduación estimada, con la dilución contada como volumen.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Graduacion {
    pub abv: f64,
    pub alcohol_ml: f64,
    pub volumen_ml: f64,
    pub dilucion_ml: f64,
    pub nivel: Nivel,
}

pub fn graduacion(receta: &Receta) -> Graduacion {
    let c = capas(receta);
    let nivel = match c.abv {
        abv if abv >= 20.0 => Nivel::Alto,
        abv if abv >= 10.0 => Nivel::Medio,
        abv if abv > 0.0 => Nivel::Bajo,
        _ => Nivel::Cero,
    };
    Graduacion {
        abv: c.abv,
        alcohol_ml: c.alcohol_ml,
        volumen_ml: c.total_ml,
        dilucion_ml: c.dilucion_ml,
        nivel,
    }
}
