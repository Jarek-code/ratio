//! Rutas de la app: una variante por pantalla.

use dioxus::prelude::*;

use crate::actividad;
use crate::ajustar;
use crate::ajustes;
use crate::comparar;
use crate::biblioteca;
use crate::descubrir;
use crate::ficha;
use crate::guardados;
use crate::historial;
use crate::lista_compra;
use crate::login;
use crate::mis_recetas;
use crate::perfil;
use crate::preparar;
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
    /// Biblioteca con el catálogo completo (B2), con búsqueda y filtros opcionales.
    ///
    /// `familia` filtra por familia (`espresso`, `con_leche`…) y `tipo` por pestaña
    /// (`cafe` o `cocteleria`); vacíos, no filtran.
    #[route("/biblioteca?:busqueda&:familia&:tipo")]
    Biblioteca {
        busqueda: String,
        familia: String,
        tipo: String,
    },
    /// Ficha de una receta (B3).
    #[route("/receta/:id")]
    Ficha { id: String },
    /// Historial de versiones de una bebida (B6).
    #[route("/receta/:id/historial")]
    Historial { id: String },
    /// Ajuste de proporciones de una receta (C1).
    #[route("/receta/:id/ajustar")]
    Ajustar { id: String },
    /// Recetas guardadas por la cuenta (C5).
    #[route("/guardados")]
    Guardados,
    /// Recetario propio: versiones, publicaciones y borradores (C4).
    #[route("/mis-recetas")]
    MisRecetas,
    /// Lista de compra a partir de las recetas elegidas (C7).
    #[route("/lista-compra")]
    ListaCompra,
    /// Novedades de La Barra sobre las recetas propias (D6).
    #[route("/actividad")]
    Actividad,
    /// Comparación de dos recetas (B4); sin `con`, se elige la segunda.
    #[route("/receta/:id/comparar?:con")]
    Comparar { id: String, con: String },
    /// Ingredientes de una receta en varias unidades (B5).
    #[route("/receta/:id/ingredientes")]
    Equivalencias { id: String },
    /// Preparación guiada con temporizador (C3).
    #[route("/receta/:id/preparar")]
    Preparar { id: String },
    /// Perfil propio (E2).
    #[route("/perfil")]
    Perfil,
    /// Perfil público de otra cuenta (E3).
    #[route("/perfil/:alias")]
    PerfilPublico { alias: String },
    /// Ajustes de unidades e idioma (E4).
    #[route("/ajustes")]
    Ajustes,
}

impl Rutas {
    /// Biblioteca sin búsqueda ni filtros.
    pub fn biblioteca() -> Self {
        Rutas::Biblioteca {
            busqueda: String::new(),
            familia: String::new(),
            tipo: String::new(),
        }
    }
}

/// Acceso; al validar el formulario continúa a Descubrir.
#[component]
fn Entrar() -> Element {
    let navegador = navigator();
    let sesion = use_context::<crate::sesion::SenalSesion>();

    // Con una sesión guardada no hace falta volver a entrar
    use_effect(move || {
        if sesion.read().is_some() {
            navegador.replace(Rutas::Descubrir);
        }
    });

    // Con sesión no se pinta nada: el efecto ya lleva a Descubrir
    if sesion.read().is_some() {
        return rsx! {};
    }

    // La primera vez se muestra la bienvenida con el aviso de edad (E5)
    let ajustes = use_context::<ajustes::SenalAjustes>();
    if !ajustes().bienvenida_vista {
        return rsx! {
            ajustes::Bienvenida {}
        };
    }

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
fn Biblioteca(busqueda: String, familia: String, tipo: String) -> Element {
    rsx! {
        biblioteca::PantallaBiblioteca { busqueda, familia, tipo }
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

/// Guardados (C5).
#[component]
fn Guardados() -> Element {
    rsx! {
        guardados::PantallaGuardados {}
    }
}

/// Mi recetario (C4).
#[component]
fn MisRecetas() -> Element {
    rsx! {
        mis_recetas::PantallaMisRecetas {}
    }
}

/// Lista de compra (C7).
#[component]
fn ListaCompra() -> Element {
    rsx! {
        lista_compra::PantallaListaCompra {}
    }
}

/// Actividad de La Barra (D6).
#[component]
fn Actividad() -> Element {
    rsx! {
        actividad::PantallaActividad {}
    }
}

/// Comparar dos recetas (B4).
#[component]
fn Comparar(id: String, con: String) -> Element {
    rsx! {
        comparar::PantallaComparar { id, con }
    }
}

/// Ingredientes y equivalencias (B5).
#[component]
fn Equivalencias(id: String) -> Element {
    rsx! {
        comparar::PantallaEquivalencias { id }
    }
}

/// Preparación guiada (C3).
#[component]
fn Preparar(id: String) -> Element {
    rsx! {
        preparar::PantallaPreparar { id }
    }
}

/// Perfil propio (E2).
#[component]
fn Perfil() -> Element {
    rsx! {
        perfil::PantallaPerfil { alias: String::new() }
    }
}

/// Perfil público (E3).
#[component]
fn PerfilPublico(alias: String) -> Element {
    rsx! {
        perfil::PantallaPerfil { alias }
    }
}

/// Ajustes (E4).
#[component]
fn Ajustes() -> Element {
    rsx! {
        ajustes::PantallaAjustes {}
    }
}
