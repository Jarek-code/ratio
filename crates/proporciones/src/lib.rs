//! Motor de proporciones de Ratio.
//!
//! Calcula la geometría del vaso y la altura real de cada capa, la graduación con la
//! dilución contada como volumen y las equivalencias de unidades. No sabe de interfaz,
//! de base de datos ni de color: entra una receta y salen números.

mod capas;
mod geometria;
mod graduacion;
mod tipos;
mod unidades;

pub use capas::{Capa, Capas, Detalle, DetalleCapa, LLENADO_MAX, capas, detalle};
pub use geometria::{Geometria, fraccion_por_volumen, radio, volumen};
pub use graduacion::{Graduacion, Nivel, graduacion};
pub use tipos::{Componente, Receta, Vaso};
pub use unidades::{CUCHARA_ML, Equivalencia, OZ_ML, equivalencias};
