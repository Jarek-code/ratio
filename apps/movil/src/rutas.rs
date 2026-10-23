//! Rutas de la app: una variante por pantalla.

use dioxus::prelude::*;

use crate::biblioteca;
use crate::comparar;
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

    /// Comparar dos recetas (B4).
    #[route("/comparar")]
    Comparar,

    /// Historial de versiones de una bebida (B6).
    #[route("/receta/:id/historial")]
    Historial { id: String },
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

/// Comparar (B4).
#[component]
fn Comparar() -> Element {
    rsx! {
        comparar::PantallaComparar {}
    }
}
