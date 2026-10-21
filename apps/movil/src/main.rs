//! Ratio · cliente móvil.

// Pantalla de acceso.
mod login;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

/// Muestra la pantalla de acceso.
#[component]
fn App() -> Element {
    rsx! {
        // Fuentes usadas por el diseño de Penpot.
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=Inter:wght@400;500;600;700&display=swap",
        }
        // El CSS se inyecta inline para que viaje con el bundle de la app.
        style { {login::CSS} }
        login::Login {}
    }
}
