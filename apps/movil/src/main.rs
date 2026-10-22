//! Ratio · cliente móvil.

// Pantallas, estilos y piezas compartidas de la aplicación.
mod historial;
mod login;
mod marca;
mod styles;

use dioxus::prelude::*;
use dioxus::router::{Routable, Router};

fn main() {
    dioxus::launch(App);
}

/// Rutas disponibles de la aplicación.
///
/// El acceso queda en `/` y el historial en `/historial`. La ruta del historial
/// permite previsualizar la pantalla antes de que exista la ficha de receta.
#[derive(Routable, Clone, PartialEq)]
pub(crate) enum Route {
    #[route("/")]
    Acceso {},
    #[route("/historial")]
    Versiones {},
}

/// Contenedor principal: fuentes, estilos y router de la app.
#[component]
fn App() -> Element {
    rsx! {
        // Fuentes usadas por el diseño de Penpot.
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=Inter:wght@400;500;600;700&display=swap",
        }
        // El CSS se inyecta inline para que viaje con el bundle de la app.
        // Se concatena en una sola cadena: Dioxus intercala nodos de hidratación
        // entre expresiones y rompería la primera regla del historial.
        style { "{styles::LOGIN}{styles::HISTORIAL}" }
        Router::<Route> {}
    }
}

/// Envuelve el login para que el router lo trate como ruta de inicio.
#[component]
fn Acceso() -> Element {
    rsx! {
        login::Login {}
    }
}

/// Envuelve el historial de versiones para exponerlo en `/historial`.
#[component]
fn Versiones() -> Element {
    rsx! {
        historial::Historial {}
    }
}
