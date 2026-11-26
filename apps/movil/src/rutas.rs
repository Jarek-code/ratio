//! Rutas de la app: una variante por pantalla.

use dioxus::prelude::*;

use crate::ajustar;
use crate::ajustes;
use crate::biblioteca;
use crate::comparar;
use crate::crear;
use crate::descubrir;
use crate::ficha;
use crate::guardados;
use crate::historial;
use crate::ingredientes;
use crate::la_barra;
use crate::lista_compra;
use crate::login;
use crate::mis_recetas;
use crate::moderacion;
use crate::perfil;
use crate::preparar;
use crate::salud::PruebaSalud;

/// Pantallas que ya existen.
#[derive(Debug, Clone, PartialEq, Routable)]
pub enum Rutas {
    /// Atajos comunes a todas las pantallas (por ejemplo, abrir Actividad desde un push).
    #[layout(Atajos)]
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
    /// Crear una receta desde cero (D1).
    #[route("/crear")]
    CrearReceta,
    /// Recetas guardadas por la cuenta (C5).
    #[route("/guardados")]
    Guardados,
    /// Recetario propio: versiones, publicaciones y borradores (C4).
    #[route("/mis-recetas")]
    MisRecetas,
    /// Lista de compra a partir de las recetas elegidas (C7).
    #[route("/lista-compra")]
    ListaCompra,
    /// Muro de La Barra (D2); `pestana` elige la sección y vacía abre Recientes.
    #[route("/barra?:pestana")]
    LaBarra { pestana: String },
    /// Hilo de comentarios de una receta (D2 y D3).
    #[route("/receta/:id/hilo")]
    Hilo { id: String },
    /// Novedades de La Barra sobre las recetas propias (D6).
    #[route("/actividad")]
    Actividad,
    /// Cola de reportes e historial de moderación (D8).
    #[route("/moderacion")]
    Moderacion,
    /// Comparación de dos recetas (B4); sin `con`, se elige la segunda.
    #[route("/receta/:id/comparar?:con")]
    Comparar { id: String, con: String },
    /// Catálogo de ingredientes, con búsqueda y alta.
    #[route("/ingredientes")]
    Ingredientes,
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

    /// La Barra en su primera sección.
    pub fn la_barra() -> Self {
        Rutas::LaBarra {
            pestana: String::new(),
        }
    }
}

/// Atajos que valen en cualquier pantalla.
///
/// Vigila el archivo que deja una notificación tocada. Con sesión abierta navega a Actividad
/// una sola vez por aviso; sin sesión no lo consume, para que espere a que la persona entre.
#[component]
fn Atajos() -> Element {
    let navegador = navigator();
    let sesion = use_context::<crate::sesion::SenalSesion>();

    // Sondeo corto: la notificación puede llegar con la app ya abierta o recién despertada
    use_future(move || async move {
        loop {
            if crate::push::tomar_atajo(&sesion) {
                navegador.push(Rutas::Actividad);
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    rsx! {
        Outlet::<Rutas> {}
    }
}

/// Acceso; al validar el formulario continúa a Descubrir.
#[component]
fn Entrar() -> Element {
    let navegador = navigator();
    let sesion = use_context::<crate::sesion::SenalSesion>();

    // Con una sesión guardada no hace falta volver a entrar; si venía de una notificación,
    // se abre Actividad en vez de Descubrir
    use_effect(move || {
        if sesion.read().is_some() {
            if crate::push::tomar_atajo(&sesion) {
                navegador.replace(Rutas::Actividad);
            } else {
                navegador.replace(Rutas::Descubrir);
            }
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

/// Crear una receta desde cero (D1).
#[component]
fn CrearReceta() -> Element {
    rsx! {
        crear::PantallaCrear {}
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

/// Muro de La Barra (D2).
#[component]
fn LaBarra(pestana: String) -> Element {
    rsx! {
        la_barra::PantallaLaBarra { pestana }
    }
}

/// Hilo de una receta (D2 y D3).
#[component]
fn Hilo(id: String) -> Element {
    rsx! {
        la_barra::PantallaHilo { id }
    }
}

/// Moderación (D8).
#[component]
fn Moderacion() -> Element {
    rsx! {
        moderacion::PantallaModeracion {}
    }
}

/// Actividad de La Barra (D6).
///
/// Se pinta con la pestaña de La Barra: la lista de novedades se escribe una sola vez y esta
/// dirección se queda para los enlaces que ya existen.
#[component]
fn Actividad() -> Element {
    rsx! {
        la_barra::PantallaLaBarra { pestana: "actividad".to_string() }
    }
}

/// Comparar dos recetas (B4).
#[component]
fn Comparar(id: String, con: String) -> Element {
    rsx! {
        comparar::PantallaComparar { id, con }
    }
}

/// Catálogo de ingredientes.
#[component]
fn Ingredientes() -> Element {
    rsx! {
        ingredientes::PantallaIngredientes {}
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

/// Botón para volver a la pantalla anterior, no a una ruta fija.
///
/// Se usa en toda pantalla que no sea una pestaña de la barra inferior. Si no hay historial
/// (por ejemplo al abrir un enlace directo) va a `respaldo`.
#[component]
pub fn Volver(etiqueta: String, #[props(into)] respaldo: Rutas) -> Element {
    let navegador = navigator();

    rsx! {
        button {
            class: "volver",
            r#type: "button",
            aria_label: "{etiqueta}",
            onclick: move |_| {
                if navegador.can_go_back() {
                    navegador.go_back();
                } else {
                    navegador.push(respaldo.clone());
                }
            },
            "‹ {etiqueta}"
        }
    }
}
