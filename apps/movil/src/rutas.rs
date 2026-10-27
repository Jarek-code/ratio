//! Rutas de la app: una variante por pantalla.

use dioxus::prelude::*;

use crate::ajustar;
use crate::biblioteca;
use crate::descubrir;
use crate::ficha;
use crate::historial;
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
    /// Biblioteca con el catálogo completo (B2).
    #[route("/biblioteca")]
    Biblioteca,
    /// Ficha de una receta (B3).
    #[route("/receta/:id")]
    Ficha { id: String },
    /// Historial de versiones de una bebida (B6).
    #[route("/receta/:id/historial")]
    Historial { id: String },
    /// Ajuste de proporciones de una receta (C1).
    #[route("/receta/:id/ajustar")]
    Ajustar { id: String },
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

/// Historial de versiones (B6).
#[component]
fn Historial(id: String) -> Element {
    rsx! {
        historial::Historial { id }
    }
}

/// Biblioteca (B2).
#[component]
fn Biblioteca() -> Element {
    rsx! {
        biblioteca::PantallaBiblioteca {}
    }
}

/// Ficha de receta (B3).
#[component]
fn Ficha(id: String) -> Element {
    rsx! {
        ficha::PantallaFicha { id }
    }
}

/// Ajuste de proporciones (C1).
#[component]
fn Ajustar(id: String) -> Element {
    rsx! {
        ajustar::PantallaAjustar { id }
    }
}
