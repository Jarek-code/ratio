//! Barra inferior con los cinco destinos de la app.

use dioxus::prelude::*;

use crate::actividad::NoLeidas;
use crate::i18n::Texto;
use crate::i18n::con;
use crate::i18n::t;
use crate::rutas::Rutas;

/// Destino de la barra inferior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destino {
    /// Descubrir (B1).
    Descubrir,
    /// Biblioteca (B2).
    Biblioteca,
    /// Recetario propio, donde se crea y se publica (C4 y D1).
    Recetario,
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
        Destino::Recetario,
        Destino::LaBarra,
        Destino::Perfil,
    ];

    /// Texto del destino, para pedirlo en el idioma elegido.
    fn texto(self) -> Texto {
        match self {
            Destino::Descubrir => Texto::DestinoDescubrir,
            Destino::Biblioteca => Texto::DestinoBiblioteca,
            Destino::Recetario => Texto::DestinoRecetario,
            Destino::LaBarra => Texto::DestinoLaBarra,
            Destino::Perfil => Texto::DestinoPerfil,
        }
    }
}

/// Barra fija con el destino activo resaltado y un punto en La Barra si hay novedades sin leer.
#[component]
pub fn BarraInferior(activo: Destino) -> Element {
    let no_leidas = use_context::<NoLeidas>().0;
    let idioma = use_context::<crate::ajustes::SenalAjustes>().read().idioma;

    rsx! {
        nav { class: "barra-inferior", "aria-label": "{t(idioma, Texto::NavAria)}",
            for destino in Destino::TODOS {
                if destino == activo {
                    button { class: "destino activo", r#type: "button", "aria-current": "page", "{t(idioma, destino.texto())}" }
                } else if destino == Destino::Descubrir {
                    Link { to: Rutas::Descubrir, class: "destino", "{t(idioma, destino.texto())}" }
                } else if destino == Destino::Biblioteca {
                    Link { to: Rutas::biblioteca(), class: "destino", "{t(idioma, destino.texto())}" }
                } else if destino == Destino::LaBarra {
                    Link { to: Rutas::la_barra(), class: "destino",
                        "{t(idioma, destino.texto())}"
                        if no_leidas() > 0 {
                            span { class: "destino-punto", role: "img", "aria-label": "{con(idioma, Texto::NovedadesSinLeer, &[&no_leidas().to_string()])}" }
                        }
                    }
                } else if destino == Destino::Perfil {
                    Link { to: Rutas::Perfil, class: "destino", "{t(idioma, destino.texto())}" }
                } else {
                    // Recetario: se crea derivando una receta y se publica desde Mi recetario
                    Link { to: Rutas::MisRecetas, class: "destino", "{t(idioma, destino.texto())}" }
                }
            }
        }
    }
}
