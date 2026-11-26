use std::sync::OnceLock; // El dibujo se calcula una sola vez

use dioxus::prelude::*;

use crate::i18n::Texto;

/// Dibujo del logo: la taza de espresso con la proporción del propio motor.
///
/// Es la misma taza que genera `proporciones::svg()` con 30 ml de espresso, 25 ml de leche y
/// 25 ml de espuma, así que el logo y las fichas comparten trazo y colores.
fn logo() -> &'static str {
    static DIBUJO: OnceLock<String> = OnceLock::new();

    DIBUJO.get_or_init(|| {
        // Geometría de `taza_espresso` en migrations/20261022120000_vasos_sandbox.sql
        let vaso = proporciones::Vaso::nuevo(90.0, 46.0, 34.0, 76.0, 4.0)
            .con_borde(4.0)
            .con_asa(proporciones::Asa::Chica)
            .con_plato();

        let receta = proporciones::Receta::nueva(
            "Ratio",
            vaso,
            vec![
                proporciones::Componente::nuevo("Espresso", 30.0, 0.0).con_color("#3E2415"),
                proporciones::Componente::nuevo("Leche", 25.0, 0.0).con_color("#F3E7D6"),
                proporciones::Componente::nuevo("Espuma", 25.0, 0.0).con_color("#FBF3E6"),
            ],
        );

        proporciones::svg(
            &receta,
            &proporciones::Opciones {
                escalas: false,
                mini: false,
            },
        )
    })
}

/// Logo de la aplicación, compartido por las pantallas.
#[component]
pub fn Marca() -> Element {
    rsx! {
        span {
            class: "brand-mark",
            role: "img",
            "aria-label": "{crate::i18n::ahora(Texto::VasoConCapas)}",
            dangerous_inner_html: logo(),
        }
    }
}
