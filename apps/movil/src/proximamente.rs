//! Aviso de las funciones que todavía no tienen pantalla.

use dioxus::prelude::*;

/// Texto que explica que la función llegará después.
const AVISO_FUTURO: &str = "Esta función se programará en el futuro";

/// Envuelve un control cuya pantalla todavía no existe y muestra el aviso al tocarlo
/// o al pasar el cursor por encima. `children` es el control que queda visible.
#[component]
pub fn Proximamente(children: Element) -> Element {
    let mut abierto = use_signal(|| false);

    rsx! {
        span {
            class: if *abierto.read() { "proximamente abierto" } else { "proximamente" },
            onclick: move |_| abierto.toggle(),
            onmouseleave: move |_| abierto.set(false),

            {children}
            span { class: "aviso", role: "tooltip", "{AVISO_FUTURO}" }
        }
    }
}
