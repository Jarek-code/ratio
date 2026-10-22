use dioxus::prelude::*;

/// Logo del vaso con capas, compartido por las pantallas de Ratio.
///
/// Se dibuja inline para que el diseño no dependa de archivos externos y para
/// conservar el mismo trazo que usa el frame de Penpot.
#[component]
pub fn Marca() -> Element {
    rsx! {
        svg {
            class: "brand-mark",
            view_box: "0 0 64 64",
            role: "img",
            "aria-label": "Vaso con capas de bebida",
            path {
                d: "M18 9 H46 L43.4 50.5 Q43 56 37.5 56 H26.5 Q21 56 20.6 50.5 Z",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.6",
                stroke_linejoin: "round",
            }
            path {
                d: "M24.3 43.5 H39.7 L38.9 50.2 Q38.7 53.5 35.7 53.5 H28.3 Q25.3 53.5 25.1 50.2 Z",
                fill: "#B9633B",
            }
            path {
                d: "M22.8 31 H41.2 L40.2 43.5 H23.8 Z",
                fill: "#D8A44C",
            }
            path {
                d: "M21.3 20 H42.7 L41.7 31 H22.3 Z",
                fill: "#6E4A32",
            }
            path {
                d: "M27 62 H37",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.6",
                stroke_linecap: "round",
            }
        }
    }
}
