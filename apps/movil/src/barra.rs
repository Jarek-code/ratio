//! Barra inferior con los cinco destinos de la app.

use dioxus::prelude::*;

use crate::proximamente::Proximamente;
use crate::rutas::Rutas;

/// Destino de la barra inferior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destino {
    /// Descubrir (B1).
    Descubrir,
    /// Biblioteca (B2).
    Biblioteca,
    /// Crear (D1).
    Crear,
    /// La Barra (D2).
    LaBarra,
    /// Perfil (E2).
    Perfil,
}

impl Destino {
    /// Los cinco destinos, en el orden en que se pintan.
    const TODOS: [Destino; 5] = [
        Destino::Descubrir,
        Destino::Biblioteca,
        Destino::Crear,
        Destino::LaBarra,
        Destino::Perfil,
    ];

    /// Nombre visible del destino.
    fn nombre(self) -> &'static str {
        match self {
            Destino::Descubrir => "Descubrir",
            Destino::Biblioteca => "Biblioteca",
            Destino::Crear => "Crear",
            Destino::LaBarra => "La Barra",
            Destino::Perfil => "Perfil",
        }
    }
}

/// Barra fija con el destino activo resaltado; los que aún no tienen pantalla avisan.
#[component]
pub fn BarraInferior(activo: Destino) -> Element {
    rsx! {
        nav { class: "barra-inferior", "aria-label": "Navegación principal",
            for destino in Destino::TODOS {
                if destino == activo {
                    button { class: "destino activo", r#type: "button", "aria-current": "page", "{destino.nombre()}" }
                } else if destino == Destino::Descubrir {
                    Link { to: Rutas::Descubrir, class: "destino", "{destino.nombre()}" }
                } else if destino == Destino::Biblioteca {
                    Link { to: Rutas::Biblioteca, class: "destino", "{destino.nombre()}" }
                } else {
                    Proximamente {
                        button { class: "destino", r#type: "button", "aria-disabled": "true", "{destino.nombre()}" }
                    }
                }
            }
        }
    }
}
