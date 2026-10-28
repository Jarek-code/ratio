//! Pantalla C4: todo lo propio en un solo lugar.
//!
//! Reutiliza la misma funcion de dibujo del vaso de la biblioteca. La lista mezcla
//! borradores, versiones privadas, publicaciones y recetas ocultas; las pestanas solo
//! cambian el filtro, porque el autor debe poder reconocer el estado de cada receta.

use api::recetario::{EstadoReceta, RecetaPropia, mis_recetas};
use dioxus::prelude::*;

use crate::barra::{BarraInferior, Destino};
use crate::biblioteca::familia_legible;
use crate::rutas::Rutas;

/// Hoja de estilos propia del recetario.
const CSS_MIS_RECETAS: Asset = asset!("/assets/mis_recetas.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Pestañas visibles del recetario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filtro {
    /// Todo lo propio, sin importar el estado.
    Todo,
    /// Recetas derivadas de otra, publicadas o no.
    Versiones,
    /// Recetas visibles en el recetario publico.
    Publicadas,
    /// Recetas originales que aun no se publican.
    Borradores,
}

impl Filtro {
    /// Pestanas en el orden en que se muestran.
    const TODOS: [Filtro; 4] = [
        Filtro::Todo,
        Filtro::Versiones,
        Filtro::Publicadas,
        Filtro::Borradores,
    ];

    /// Texto de la pestana.
    fn etiqueta(self) -> &'static str {
        match self {
            Filtro::Todo => "Todo",
            Filtro::Versiones => "Versiones",
            Filtro::Publicadas => "Publicadas",
            Filtro::Borradores => "Borradores",
        }
    }

    /// Si una receta pertenece a esta pestana.
    fn incluye(self, receta: &RecetaPropia) -> bool {
        match self {
            Filtro::Todo => true,
            Filtro::Versiones => receta.origen_id.is_some(),
            Filtro::Publicadas => receta.estado == EstadoReceta::Publicada,
            Filtro::Borradores => receta.estado == EstadoReceta::Borrador,
        }
    }

    /// Mensaje cuando la pestana no tiene recetas.
    fn vacio(self) -> &'static str {
        match self {
            Filtro::Todo => {
                "Todavía no tienes recetas propias. Empieza por ajustar una de la biblioteca."
            }
            Filtro::Versiones => "Todavía no has guardado ninguna versión propia.",
            Filtro::Publicadas => "Todavía no has publicado una receta.",
            Filtro::Borradores => "No tienes borradores pendientes.",
        }
    }
}

/// Texto visible del estado.
fn estado_etiqueta(estado: EstadoReceta) -> &'static str {
    match estado {
        EstadoReceta::Borrador => "Borrador",
        EstadoReceta::Privada => "Privada",
        EstadoReceta::Publicada => "Publicada",
        EstadoReceta::Oculta => "Oculta",
    }
}

/// Clase del chip de estado.
fn estado_clase(estado: EstadoReceta) -> &'static str {
    match estado {
        EstadoReceta::Borrador => "estado-chip estado-chip--borrador",
        EstadoReceta::Privada => "estado-chip estado-chip--privada",
        EstadoReceta::Publicada => "estado-chip estado-chip--publicada",
        EstadoReceta::Oculta => "estado-chip estado-chip--oculta",
    }
}

/// Pantalla C4: carga las recetas de la cuenta y las filtra por pestana.
#[component]
pub fn PantallaMisRecetas() -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let filtro = use_signal(|| Filtro::Todo);

    let mut recurso = use_resource(move || {
        let token = crate::sesion::token(&sesion);
        async move {
            match token {
                Some(token) => mis_recetas(token).await.map(Some),
                None => Ok(None),
            }
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_MIS_RECETAS }

        main { class: "mi-recetario",
            BarraSuperior {}

            header { class: "mi-recetario-cabeza",
                p { class: "eyebrow", "Recetario" }
                h1 { "Mi recetario" }
                p { class: "mi-recetario-sub", "Tus versiones, tus publicaciones y lo que aún no sale de casa." }
            }

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando tu recetario…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos traer tu recetario: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| recurso.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(None)) => rsx! {
                    SinSesion {}
                },
                Some(Ok(Some(lista))) => rsx! {
                    ListaPropia { lista: lista.clone(), filtro }
                },
            }
        }

        BarraInferior { activo: Destino::Perfil }
    }
}

/// Barra superior con la vuelta a Descubrir.
#[component]
fn BarraSuperior() -> Element {
    rsx! {
        header { class: "mi-recetario-barra",
            Link {
                to: Rutas::Descubrir,
                class: "mi-recetario-volver",
                aria_label: "Volver a Descubrir",
                "‹ Descubrir"
            }
            span { class: "mi-recetario-marca", "Ratio" }
        }
    }
}

/// Aviso para quien abre C4 sin sesión.
#[component]
fn SinSesion() -> Element {
    rsx! {
        div { class: "estado",
            p { "Inicia sesión para ver tus versiones, tus publicaciones y tus borradores." }
            Link { to: Rutas::Entrar, class: "mi-recetario-accion", "Entrar" }
        }
    }
}

/// Lista filtrada por las pestanas del recetario.
#[component]
fn ListaPropia(lista: Vec<RecetaPropia>, filtro: Signal<Filtro>) -> Element {
    let mut filtro = filtro;
    let activo = filtro();
    let visibles: Vec<RecetaPropia> = lista
        .iter()
        .filter(|receta| activo.incluye(receta))
        .cloned()
        .collect();

    let total = lista.len();

    rsx! {
        nav { class: "mi-recetario-tabs", role: "tablist", "aria-label": "Filtrar mi recetario",
            for opcion in Filtro::TODOS {
                button {
                    key: "{opcion.etiqueta()}",
                    class: if opcion == activo { "mi-recetario-tab activo" } else { "mi-recetario-tab" },
                    r#type: "button",
                    role: "tab",
                    "aria-selected": if opcion == activo { "true" } else { "false" },
                    onclick: move |_| filtro.set(opcion),
                    span { "{opcion.etiqueta()}" }
                    span { class: "mi-recetario-cuenta", "{lista.iter().filter(|receta| opcion.incluye(receta)).count()}" }
                }
            }
        }

        p { class: "mi-recetario-resumen", role: "status", "{visibles.len()} de {total} recetas" }

        if visibles.is_empty() {
            div { class: "estado",
                p { "{activo.vacio()}" }
                Link { to: Rutas::Biblioteca, class: "mi-recetario-accion", "Ir a la biblioteca" }
            }
        } else {
            ul { class: "mi-recetario-lista",
                for receta in visibles {
                    TarjetaPropia { key: "{receta.receta.id}", receta }
                }
            }
        }
    }
}

/// Tarjeta de una receta propia con su estado y su origen.
#[component]
fn TarjetaPropia(receta: RecetaPropia) -> Element {
    let resumen = &receta.receta;
    let capas = proporciones::capas(&resumen.receta);
    let graduacion = proporciones::graduacion(&resumen.receta);
    let svg = proporciones::svg(
        &resumen.receta,
        &proporciones::Opciones {
            escalas: false,
            mini: true,
        },
    );

    let mut datos = vec![format!("{:.0} ml", capas.total_ml)];

    if graduacion.abv > 0.0 {
        datos.push(format!("{:.1} % ABV", graduacion.abv));
    }

    if let Some(minutos) = resumen.minutos {
        datos.push(format!("{minutos} min"));
    }

    if let Some(dificultad) = resumen.dificultad {
        datos.push(format!("Dificultad {dificultad}"));
    }

    let actualizada =
        (receta.actualizada_en != receta.creada_en).then(|| receta.actualizada_en.clone());

    rsx! {
        li {
            Link { to: Rutas::Ficha { id: resumen.id.clone() }, class: "propia-tarjeta",
                div { class: "propia-mini", dangerous_inner_html: svg }

                div { class: "propia-cuerpo",
                    div { class: "propia-top",
                        h2 { "{resumen.receta.nombre}" }
                        span { class: estado_clase(receta.estado), "{estado_etiqueta(receta.estado)}" }
                    }

                    p { class: "propia-meta", "{familia_legible(&resumen.familia)} · {datos.join(\" · \")}" }

                    if let Some(origen) = receta.origen_nombre.clone() {
                        p { class: "propia-origen", "Versión de {origen}" }
                    }

                    div { class: "propia-fechas",
                        span { "Creada {receta.creada_en}" }
                        if let Some(actualizada) = actualizada {
                            span { "Actualizada {actualizada}" }
                        }
                    }
                }
            }
        }
    }
}
