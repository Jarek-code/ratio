//! Dibujo del vaso con la geometría del catálogo, en la salida estándar.
//!
//! Sin argumentos dibuja la taza de espresso del logo:
//!
//! ```text
//! cargo run -p proporciones --example taza_espresso > logo.svg
//! ```
//!
//! Con `--capas N` (1, 2 o 3) dibuja un moka en la taza de 160 ml con las primeras N capas
//! servidas, para ilustraciones por pasos:
//!
//! ```text
//! cargo run -p proporciones --example taza_espresso -- --capas 2 > moka.svg
//! ```

use proporciones::{Asa, Componente, Opciones, Receta, Vaso, svg};

/// Moka de tres capas iguales en la taza de 160 ml, con las primeras `capas` servidas.
fn moka(capas: usize) -> Receta {
    // Geometría de `taza_160` en migrations/20261022120000_vasos_sandbox.sql
    let vaso = Vaso::nuevo(160.0, 62.0, 52.0, 128.0, 5.0)
        .con_borde(5.0)
        .con_asa(Asa::Grande)
        .con_plato();

    // Tres capas de 50 ml: espresso, chocolate y espuma
    let todas = [
        Componente::nuevo("Espresso", 50.0, 0.0).con_color("#2E1B12"),
        Componente::nuevo("Chocolate", 50.0, 0.0).con_color("#8A5230"),
        Componente::nuevo("Espuma", 50.0, 0.0).con_color("#F3E3C8"),
    ];

    Receta::nueva(
        "Moka",
        vaso,
        todas.iter().take(capas.clamp(1, 3)).cloned().collect(),
    )
}

/// Espresso tonic de toronja en vaso highball (350 ml).
fn espresso_tonic() -> Receta {
    let vaso = Vaso::nuevo(350.0, 33.0, 30.0, 186.0, 4.0).con_borde(6.0);
    Receta::nueva(
        "Espresso tonic de toronja",
        vaso,
        vec![
            Componente::nuevo("Agua tónica", 150.0, 0.0).con_color("#E4EEF2"),
            Componente::nuevo("Espresso", 40.0, 0.0).con_color("#2E1B12"),
            Componente::nuevo("Refresco de toronja", 60.0, 0.0).con_color("#E8804F"),
        ],
    )
}

/// Negroni en vaso gibraltar (135 ml), a partes iguales.
fn negroni() -> Receta {
    let vaso = Vaso::nuevo(135.0, 42.0, 39.0, 98.0, 5.0).con_borde(5.0);
    Receta::nueva(
        "Negroni",
        vaso,
        vec![
            Componente::nuevo("Ginebra", 30.0, 40.0).con_color("#E9EEE8"),
            Componente::nuevo("Campari", 30.0, 25.0).con_color("#B3122B"),
            Componente::nuevo("Vermut rojo", 30.0, 15.0).con_color("#6E2230"),
        ],
    )
}

/// Margarita en copa de cóctel (180 ml).
fn margarita() -> Receta {
    let vaso = Vaso::nuevo(180.0, 47.0, 4.0, 76.0, 2.5)
        .con_borde(4.0)
        .con_tallo(66.0);
    Receta::nueva(
        "Margarita",
        vaso,
        vec![
            Componente::nuevo("Tequila", 50.0, 40.0).con_color("#E7DCA8"),
            Componente::nuevo("Triple seco", 25.0, 30.0).con_color("#F2EAD0"),
            Componente::nuevo("Zumo de lima", 25.0, 0.0).con_color("#C6D86B"),
        ],
    )
}

fn main() {
    // `--capas N`: moka por pasos; `--bebida X`: otra bebida; sin argumentos, la taza del logo
    let argumentos: Vec<String> = std::env::args().collect();
    let bebida = argumentos
        .iter()
        .position(|arg| arg == "--bebida")
        .and_then(|posicion| argumentos.get(posicion + 1));
    if let Some(nombre) = bebida {
        let receta = match nombre.as_str() {
            "margarita" => margarita(),
            "negroni" => negroni(),
            _ => espresso_tonic(),
        };
        print!(
            "{}",
            svg(
                &receta,
                &Opciones {
                    escalas: false,
                    mini: false
                }
            )
        );
        return;
    }
    let capas = argumentos
        .iter()
        .position(|arg| arg == "--capas")
        .and_then(|posicion| argumentos.get(posicion + 1))
        .and_then(|valor| valor.parse::<usize>().ok());

    let Some(capas) = capas else {
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
        return;
    };

    let receta = moka(capas);

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
