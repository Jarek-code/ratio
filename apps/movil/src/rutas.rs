//! Rutas de la app: una variante por pantalla.

use dioxus::prelude::*;

use crate::descubrir;
use crate::ficha;
use crate::login;
use crate::salud::PruebaSalud;

/// Pantallas que ya existen.
#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Rutas {
    /// Acceso con correo y contraseña (E1).
    #[route("/")]
    Entrar,
    /// Descubrir: sugerencia del día y accesos del recetario (B1).
    #[route("/descubrir")]
    Descubrir,
    /// Ficha de una receta (B3).
    #[route("/receta/:id")]
    Ficha { id: String },
}

/// Acceso; al validar el formulario continúa a Descubrir.
#[component]
fn Entrar() -> Element {
    let navegador = navigator();

    rsx! {
        login::Login {
            on_entrar: move |_| {
                navegador.push(Rutas::Descubrir);
            }
        }
        PruebaSalud {}
    }
}

/// Descubrir (B1).
#[component]
fn Descubrir() -> Element {
    rsx! {
        descubrir::PantallaDescubrir {}
    }
}

/// Ficha de receta (B3).
#[component]
fn Ficha(id: String) -> Element {
    rsx! {
        ficha::PantallaFicha { id }
    }
}
