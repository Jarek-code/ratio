//! Motor de proporciones de Ratio.
//!
//! Calcula la geometría del vaso y la altura real de cada capa, la graduación con la
//! dilución contada como volumen, las equivalencias de unidades y el dibujo SVG del
//! vaso. Entra una receta y salen números, más el gráfico de esos números.

mod capas;
mod geometria;
mod graduacion;
mod svg;
mod tipos;
mod unidades;

pub use capas::{Capa, Capas, Detalle, DetalleCapa, LLENADO_MAX, capas, detalle};
pub use geometria::{Geometria, fraccion_por_volumen, radio, volumen};
pub use graduacion::{Graduacion, Nivel, graduacion};
pub use svg::{Opciones, barra, leyenda, svg};
pub use tipos::{Asa, Componente, Receta, Vaso};
pub use unidades::{CUCHARA_ML, Equivalencia, OZ_ML, equivalencias};
