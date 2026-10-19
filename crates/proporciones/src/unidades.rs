//! La misma cantidad en todas las unidades que muestra la interfaz.

use crate::capas::capas;
use crate::tipos::Receta;

pub const OZ_ML: f64 = 29.5735;
pub const CUCHARA_ML: f64 = 15.0;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Equivalencia {
    pub nombre: String,
    pub ml: f64,
    pub oz: f64,
    /// Partes normalizadas contra el ingrediente con alcohol más pequeño.
    pub partes: f64,
    pub cucharadas: f64,
    pub dilucion: bool,
}

pub fn equivalencias(receta: &Receta) -> Vec<Equivalencia> {
    let c = capas(receta);
    let menor = c
        .capas
        .iter()
        .filter(|x| !x.dilucion && x.ml > 0.0)
        .map(|x| x.ml)
        .fold(f64::INFINITY, f64::min);
    let menor = if menor.is_finite() { menor } else { 1.0 };

    c.capas
        .iter()
        .map(|x| Equivalencia {
            nombre: x.nombre.clone(),
            ml: x.ml,
            oz: x.ml / OZ_ML,
            partes: x.ml / menor,
            cucharadas: x.ml / CUCHARA_ML,
            dilucion: x.dilucion,
        })
        .collect()
}
