//! Catálogo de ingredientes (B5): búsqueda, categorías y alta de ingredientes nuevos.

use api::ingredientes::{IngredienteNuevo, IngredienteOpcion, catalogo, crear_ingrediente};
use dioxus::prelude::*;

use crate::i18n::Texto;
use crate::red::AvisoRed;
use crate::rutas::Rutas;
use crate::sesion::SenalSesion;

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Pantalla del catálogo: buscar, ver por categoría y agregar.
#[component]
pub fn PantallaIngredientes() -> Element {
    let sesion = use_context::<SenalSesion>(); // Para saber si se puede agregar
    let mut busqueda = use_signal(String::new); // Texto del buscador
    let mut recurso = use_resource(move || crate::red::con_reintentos(catalogo));
    let mut agregando = use_signal(|| false); // Si el formulario está abierto

    let lista = recurso.read().clone();

    rsx! {
        document::Stylesheet { href: CSS_BASE }

        main { class: "descubrir",
            header { class: "seccion-cabeza",
                crate::rutas::Volver {
                    etiqueta: crate::i18n::ahora(Texto::VolverAtras).to_string(),
                    respaldo: Rutas::biblioteca(),
                }
            }

            p { class: "eyebrow", "{crate::i18n::ahora(Texto::RecetarioSeccion)}" }
            h1 { "{crate::i18n::ahora(Texto::CatalogoDeIngredientes)}" }

            label { class: "ajustar-nombre", r#for: "buscar-ingrediente",
                span { "{crate::i18n::ahora(Texto::BuscarIngrediente)}" }
                input {
                    id: "buscar-ingrediente",
                    class: "ajustar-select",
                    r#type: "search",
                    value: "{busqueda}",
                    oninput: move |evento| busqueda.set(evento.value()),
                }
            }

            match lista {
                None => rsx! { p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoIngredientes)}" } },
                Some(Err(causa)) => rsx! {
                    AvisoRed { causa, reintentar: move |_| recurso.restart() }
                },
                Some(Ok(catalogo_vec)) => rsx! {
                    ListaCatalogo { catalogo: catalogo_vec, busqueda: busqueda() }
                },
            }

            // Alta de un ingrediente que no está en el catálogo
            if sesion.read().is_some() {
                button {
                    class: "chip",
                    r#type: "button",
                    onclick: move |_| agregando.set(!agregando()),
                    if agregando() {
                        "{crate::i18n::ahora(Texto::Cancelar)}"
                    } else {
                        "{crate::i18n::ahora(Texto::NuevoIngrediente)}"
                    }
                }

                if agregando() {
                    FormularioIngrediente {
                        al_agregar: move |_| {
                            agregando.set(false);
                            recurso.restart();
                        },
                    }
                }
            }
        }
    }
}

/// Lista agrupada por categoría y filtrada por la búsqueda.
#[component]
fn ListaCatalogo(catalogo: Vec<IngredienteOpcion>, busqueda: String) -> Element {
    let buscado = busqueda.trim().to_lowercase();
    let filtrados: Vec<IngredienteOpcion> = catalogo
        .iter()
        .filter(|ingrediente| {
            buscado.is_empty() || ingrediente.nombre.to_lowercase().contains(&buscado)
        })
        .cloned()
        .collect();

    if filtrados.is_empty() {
        return rsx! {
            p { class: "estado", "{crate::i18n::ahora(Texto::SinIngredientesQueCoincidan)}" }
        };
    }

    // Agrupados por la primera letra del nombre: es estable y no depende de datos sueltos
    let mut grupos: Vec<(String, Vec<IngredienteOpcion>)> = Vec::new();
    for ingrediente in filtrados {
        let inicial = ingrediente
            .nombre
            .chars()
            .next()
            .map(|letra| letra.to_uppercase().to_string())
            .unwrap_or_else(|| "#".to_string());

        match grupos.last_mut() {
            Some((letra, lista)) if *letra == inicial => lista.push(ingrediente),
            _ => grupos.push((inicial, vec![ingrediente])),
        }
    }

    rsx! {
        for (letra, ingredientes_vec) in grupos {
            section { key: "{letra}", class: "seccion",
                p { class: "eyebrow", "{letra}" }

                ul { class: "ingredientes-lista",
                    for ingrediente in ingredientes_vec {
                        li { key: "{ingrediente.id}", class: "ingrediente-fila",
                            span {
                                class: "ingrediente-punto",
                                style: "background:{ingrediente.color.clone().unwrap_or_else(|| \"#B9A48C\".to_string())}",
                                "aria-hidden": "true",
                            }
                            span { class: "ingrediente-nombre", "{ingrediente.nombre}" }
                            span { class: "mini",
                                "{ingrediente.unidad}"
                                if ingrediente.graduacion_pct > 0.0 {
                                    " · {ingrediente.graduacion_pct:.0} % ABV"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Formulario mínimo: nombre, categoría, unidad, alcohol y color.
#[component]
fn FormularioIngrediente(al_agregar: EventHandler<()>) -> Element {
    let sesion = use_context::<SenalSesion>();
    let mut nombre = use_signal(String::new);
    let mut tipo = use_signal(String::new);
    let mut unidad = use_signal(|| "ml".to_string());
    let mut graduacion = use_signal(String::new);
    let mut color = use_signal(String::new);
    let mut dilucion = use_signal(|| false);
    let mut guardando = use_signal(|| false);
    let mut aviso = use_signal(|| None::<String>);

    rsx! {
        section { class: "seccion",
            label { class: "ajustar-nombre", r#for: "ing-nombre",
                span { "{crate::i18n::ahora(Texto::NombreIngrediente)}" }
                input {
                    id: "ing-nombre",
                    class: "ajustar-select",
                    r#type: "text",
                    maxlength: "60",
                    value: "{nombre}",
                    oninput: move |evento| nombre.set(evento.value()),
                }
            }

            label { class: "ajustar-nombre", r#for: "ing-tipo",
                span { "{crate::i18n::ahora(Texto::CategoriaIngrediente)}" }
                input {
                    id: "ing-tipo",
                    class: "ajustar-select",
                    r#type: "text",
                    maxlength: "30",
                    placeholder: "{crate::i18n::ahora(Texto::PlaceholderCategoria)}",
                    value: "{tipo}",
                    oninput: move |evento| tipo.set(evento.value()),
                }
            }

            div { class: "ajustar-acciones",
                label { class: "ajustar-nombre", r#for: "ing-unidad",
                    span { "{crate::i18n::ahora(Texto::Unidad)}" }
                    select {
                        id: "ing-unidad",
                        class: "ajustar-select",
                        value: "{unidad}",
                        onchange: move |evento| unidad.set(evento.value()),
                        option { value: "ml", "ml" }
                        option { value: "mg", "mg" }
                    }
                }

                label { class: "ajustar-nombre", r#for: "ing-graduacion",
                    span { "{crate::i18n::ahora(Texto::Graduacion)}" }
                    input {
                        id: "ing-graduacion",
                        class: "ajustar-select",
                        r#type: "text",
                        inputmode: "decimal",
                        value: "{graduacion}",
                        oninput: move |evento| graduacion.set(evento.value()),
                    }
                }

                label { class: "ajustar-nombre", r#for: "ing-color",
                    span { "{crate::i18n::ahora(Texto::Color)}" }
                    input {
                        id: "ing-color",
                        class: "ajustar-select",
                        r#type: "text",
                        maxlength: "7",
                        placeholder: "#7A4A2B",
                        value: "{color}",
                        oninput: move |evento| color.set(evento.value()),
                    }
                }
            }

            label { class: "chip",
                input {
                    r#type: "checkbox",
                    checked: dilucion(),
                    onchange: move |evento| dilucion.set(evento.checked()),
                }
                span { "{crate::i18n::ahora(Texto::DiluyeLaBebida)}" }
            }

            button {
                class: "primario",
                r#type: "button",
                disabled: guardando(),
                onclick: move |_| {
                    let Some(token) = crate::sesion::token(&sesion) else {
                        aviso.set(Some(crate::i18n::ahora(Texto::IniciaSesionIngrediente).to_string()));
                        return;
                    };
                    if guardando() {
                        return;
                    }

                    let datos = IngredienteNuevo {
                        nombre: nombre.read().trim().to_string(),
                        tipo: Some(tipo.read().trim().to_string()),
                        unidad: unidad.read().clone(),
                        graduacion_pct: graduacion.read().replace(',', ".").parse().unwrap_or(0.0),
                        color: Some(color.read().trim().to_string()),
                        dilucion: dilucion(),
                    };

                    guardando.set(true);
                    aviso.set(None);

                    spawn(async move {
                        match crate::red::con_plazo(crear_ingrediente(token, datos)).await {
                            Ok(_) => {
                                aviso.set(Some(crate::i18n::ahora(Texto::IngredienteAgregado).to_string()));
                                nombre.set(String::new());
                                color.set(String::new());
                                graduacion.set(String::new());
                                al_agregar.call(());
                            }
                            Err(causa) => aviso.set(Some(crate::i18n::con(
                                crate::i18n::actual(),
                                Texto::NoSePudoAgregarIngrediente,
                                &[&crate::red::mensaje(&causa)],
                            ))),
                        }
                        guardando.set(false);
                    });
                },
                if guardando() {
                    "{crate::i18n::ahora(Texto::Guardando)}"
                } else {
                    "{crate::i18n::ahora(Texto::AgregarIngrediente)}"
                }
            }

            if let Some(texto) = aviso() {
                p { class: "ajustar-aviso", role: "status", "{texto}" }
            }
        }
    }
}
