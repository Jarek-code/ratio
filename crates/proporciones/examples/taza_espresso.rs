//! Dibujo de la taza de espresso del logo, con la geometría del catálogo.
//!
//! Escribe el SVG en la salida estándar:
//!
//! ```text
//! cargo run -p proporciones --example taza_espresso > logo.svg
//! ```

use proporciones::{Asa, Componente, Opciones, Receta, Vaso, svg};

fn main() {
    // Geometría de `taza_espresso` en migrations/20261022120000_vasos_sandbox.sql
    let vaso = Vaso::nuevo(90.0, 46.0, 34.0, 76.0, 4.0)
        .con_borde(4.0)
        .con_asa(Asa::Chica)
        .con_plato();

    let receta = Receta::nueva(
        "Ratio",
        vaso,
        vec![
            Componente::nuevo("Espresso", 30.0, 0.0).con_color("#3E2415"),
            Componente::nuevo("Leche", 25.0, 0.0).con_color("#F3E7D6"),
            Componente::nuevo("Espuma", 25.0, 0.0).con_color("#FBF3E6"),
        ],
    );

    print!(
        "{}",
        svg(
            &receta,
            &Opciones {
                escalas: false,
                mini: false,
            }
        )
    );
}
