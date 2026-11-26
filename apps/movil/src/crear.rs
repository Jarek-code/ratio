//! Crear una receta desde cero (D1), parte de la app.
//!
//! El formulario pide lo mínimo para que la bebida exista: nombre, pestaña, familia, vaso e
//! ingredientes del catálogo con su cantidad. Los pasos son opcionales. La receta nace como
//! borrador de la cuenta y se publica después desde Mi recetario o desde La Barra.

// LIBRERÍAS

use api::guardar::ComponenteAgregado; // Capa nueva con su cantidad
use api::guardar::PasoNuevo; // Paso de preparación
use api::guardar::RecetaDesdeCero; // Datos que viajan al servidor
use api::guardar::crear_receta; // Server function que guarda la receta
use api::ingredientes::IngredienteOpcion; // Catálogo de ingredientes
use api::vasos::VasoOpcion; // Catálogo de vasos
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::i18n::Texto; // Textos traducibles
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Rutas para volver y abrir la receta nueva
use crate::sesion::SenalSesion; // Cuenta y token

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Hoja propia del editor de la receta.
const CSS_AJUSTAR: Asset = asset!("/assets/ajustar.css");
/// Campos, selectores y tarjetas de formulario.
const CSS_FORMULARIO: Asset = asset!("/assets/formulario.css");

/// Cantidad con la que entra un ingrediente nuevo, en mililitros o gramos.
const CANTIDAD_INICIAL: f64 = 30.0;

/// Pasos que se ofrecen de entrada.
const PASOS_INICIALES: usize = 1;

// ESTRUCTURAS

/// Una capa que se está armando en el formulario.
#[derive(Debug, Clone, PartialEq)]
struct Capa {
    /// Ingrediente del catálogo.
    ingrediente: IngredienteOpcion,
    /// Cantidad en la unidad del catálogo (ml o mg).
    cantidad: f64,
}

/// Un paso que se está escribiendo.
#[derive(Debug, Clone, PartialEq)]
struct Paso {
    /// Encabezado corto.
    titulo: String,
    /// Texto del paso.
    instruccion: String,
    /// Temporizador en segundos, si lo lleva.
    temporizador_s: Option<i32>,
}

// FUNCIONES

/// Pantalla D1: formulario para crear una receta nueva.
#[component]
pub fn PantallaCrear() -> Element {
    let sesion = use_context::<SenalSesion>();
    let navegador = navigator();

    // Catálogos que llenan los desplegables
    let mut vasos_res = use_resource(move || crate::red::con_reintentos(api::vasos::vasos));
    let mut ingredientes_res =
        use_resource(move || crate::red::con_reintentos(api::ingredientes::catalogo));

    // Datos del formulario
    let mut nombre = use_signal(String::new);
    let mut tipo = use_signal(|| "cafe".to_string());
    let mut familia = use_signal(String::new);
    let mut indice_vaso = use_signal(|| 0_usize);
    let mut capas = use_signal(Vec::<Capa>::new);
    let mut pasos = use_signal(|| {
        vec![
            Paso {
                titulo: String::new(),
                instruccion: String::new(),
                temporizador_s: None,
            };
            PASOS_INICIALES
        ]
    });
    let mut seleccion = use_signal(String::new);
    let mut filtro_ingrediente = use_signal(String::new); // Buscador del desplegable
    let mut cantidad_nueva = use_signal(|| CANTIDAD_INICIAL.to_string());
    let mut guardando = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    // Estados ya resueltos, sin prestados que duren hasta el rsx
    let vasos_cargando = vasos_res.read().is_none();
    let vasos_error = match &*vasos_res.read() {
        Some(Err(causa)) => Some(causa.clone()),
        _ => None,
    };
    let vasos_vec: Vec<VasoOpcion> = vasos_res
        .read()
        .as_ref()
        .and_then(|res| res.as_ref().ok())
        .cloned()
        .unwrap_or_default();
    let ingredientes_cargando = ingredientes_res.read().is_none();
    let ingredientes_error = match &*ingredientes_res.read() {
        Some(Err(causa)) => Some(causa.clone()),
        _ => None,
    };
    let catalogo_vec: Vec<IngredienteOpcion> = ingredientes_res
        .read()
        .as_ref()
        .and_then(|res| res.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    let capas_actuales = capas.read().clone();
    let pasos_actuales = pasos.read().clone();
    let vaso_actual = vasos_vec.get(*indice_vaso.read()).cloned();
    let total_ml: f64 = capas_actuales
        .iter()
        .map(|capa| api::ml_equivalentes(capa.cantidad, &capa.ingrediente.unidad))
        .sum();
    let capacidad = vaso_actual
        .as_ref()
        .map(|vaso| vaso.vaso.capacidad_ml)
        .unwrap_or(0.0);
    let sin_ingredientes = capas_actuales.is_empty();
    let vasos_para_guardar = vasos_vec.clone(); // Copia para el manejador de guardado

    // Vista previa: el vaso elegido con las capas que ya se agregaron
    let vista_previa = vaso_actual.as_ref().map(|vaso| {
        let receta = proporciones::Receta::nueva(
            "",
            vaso.vaso.clone(),
            capas_actuales
                .iter()
                .map(|capa| {
                    let mut componente = proporciones::Componente::nuevo(
                        capa.ingrediente.nombre.clone(),
                        capa.cantidad,
                        capa.ingrediente.graduacion_pct,
                    );
                    componente.dilucion = capa.ingrediente.dilucion;
                    if let Some(color) = capa.ingrediente.color.clone() {
                        componente = componente.con_color(color);
                    }
                    componente
                })
                .collect(),
        );

        proporciones::svg(
            &receta,
            &proporciones::Opciones {
                escalas: false,
                mini: false,
            },
        )
    });

    // Los ingredientes que ya están no se ofrecen otra vez; el buscador acota la lista
    let buscado = filtro_ingrediente.read().trim().to_lowercase();

    // Unidad del ingrediente elegido: la cantidad se escribe en esa medida
    let unidad_nueva = catalogo_vec
        .iter()
        .find(|opcion| opcion.id == seleccion())
        .map(|opcion| opcion.unidad.clone())
        .unwrap_or_else(|| "ml".to_string());
    let disponibles: Vec<IngredienteOpcion> = catalogo_vec
        .iter()
        .filter(|opcion| {
            !capas_actuales
                .iter()
                .any(|capa| capa.ingrediente.id == opcion.id)
                && (buscado.is_empty() || opcion.nombre.to_lowercase().contains(&buscado))
        })
        .cloned()
        .collect();

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_AJUSTAR }
        document::Stylesheet { href: CSS_FORMULARIO }

        main { class: "ajustar",
            header { class: "ajustar-barra",
                crate::rutas::Volver {
                    etiqueta: crate::i18n::ahora(Texto::VolverAtras).to_string(),
                    respaldo: Rutas::MisRecetas,
                }
            }

            section { class: "ajustar-cabeza",
                p { class: "eyebrow", "{crate::i18n::ahora(Texto::CrearRecetaSeccion)}" }
                h1 { "{crate::i18n::ahora(Texto::RecetaNuevaTitulo)}" }
                p { class: "nota", "{crate::i18n::ahora(Texto::CrearRecetaNota)}" }
            }

            // Datos de la bebida
            section { class: "seccion", "aria-labelledby": "titulo-datos",
                p { id: "titulo-datos", class: "eyebrow", "{crate::i18n::ahora(Texto::DatosDeLaBebida)}" }

                label { class: "ajustar-nombre", r#for: "receta-nombre",
                    span { "{crate::i18n::ahora(Texto::NombreDeLaReceta)}" }
                    input {
                        id: "receta-nombre",
                        class: "ajustar-select",
                        r#type: "text",
                        maxlength: "120",
                        placeholder: "{crate::i18n::ahora(Texto::PlaceholderNombreReceta)}",
                        value: "{nombre}",
                        oninput: move |evento| nombre.set(evento.value()),
                    }
                }

                div { class: "chips", role: "radiogroup", "aria-label": "{crate::i18n::ahora(Texto::TipoDeBebida)}",
                    for opcion in ["cafe", "cocteleria"] {
                        button {
                            key: "{opcion}",
                            class: if tipo() == opcion { "chip chip--activo" } else { "chip" },
                            r#type: "button",
                            role: "radio",
                            "aria-checked": if tipo() == opcion { "true" } else { "false" },
                            onclick: move |_| {
                                tipo.set(opcion.to_string());
                                // La familia propuesta sigue a la pestaña
                                if familia().is_empty() {
                                    familia.set(opcion.to_string());
                                }
                            },
                            if opcion == "cafe" {
                                "{crate::i18n::ahora(Texto::CafeSeccion)}"
                            } else {
                                "{crate::i18n::ahora(Texto::CocteleriaSeccion)}"
                            }
                        }
                    }
                }

                label { class: "ajustar-nombre", r#for: "receta-familia",
                    span { "{crate::i18n::ahora(Texto::FamiliaDeLaBebida)}" }
                    input {
                        id: "receta-familia",
                        class: "ajustar-select",
                        r#type: "text",
                        maxlength: "40",
                        placeholder: "{crate::i18n::ahora(Texto::PlaceholderFamilia)}",
                        value: "{familia}",
                        oninput: move |evento| familia.set(evento.value()),
                    }
                }
            }

            // Vaso
            section { class: "seccion", "aria-labelledby": "titulo-vaso-nuevo",
                p { id: "titulo-vaso-nuevo", class: "eyebrow", "{crate::i18n::ahora(Texto::Vaso)}" }

                if vasos_cargando {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoVasos)}" }
                } else if let Some(causa) = vasos_error {
                    AvisoRed { causa, reintentar: move |_| vasos_res.restart() }
                } else if vasos_vec.is_empty() {
                    p { class: "estado", "{crate::i18n::ahora(Texto::NoHayVasos)}" }
                } else {
                        label { class: "solo-lector", r#for: "vaso-nuevo", "{crate::i18n::ahora(Texto::ElegirElVaso)}" }
                        select {
                            id: "vaso-nuevo",
                            class: "ajustar-select",
                            value: "{indice_vaso}",
                            onchange: move |evento| {
                                if let Ok(indice) = evento.value().parse::<usize>()
                                    && indice < vasos_vec.len()
                                {
                                    indice_vaso.set(indice);
                                }
                            },
                            for (indice, vaso) in vasos_vec.iter().enumerate() {
                                option {
                                    key: "{vaso.id}",
                                    value: "{indice}",
                                    "{vaso.nombre} · {vaso.vaso.capacidad_ml:.0} ml"
                                }
                            }
                        }
                }

                span { class: "mini", "{total_ml:.0} ml / {capacidad:.0} ml" }

                if let Some(svg) = vista_previa.clone() {
                    div { class: "crear-vista", dangerous_inner_html: svg }
                }
            }

            // Ingredientes
            section { class: "seccion", "aria-labelledby": "titulo-ingredientes-nuevos",
                p { id: "titulo-ingredientes-nuevos", class: "eyebrow", "{crate::i18n::ahora(Texto::Ingredientes)}" }

                if ingredientes_cargando {
                    p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoIngredientes)}" }
                } else if let Some(causa) = ingredientes_error {
                    AvisoRed { causa, reintentar: move |_| ingredientes_res.restart() }
                } else {
                        div { class: "anadir-tarjeta",
                            label { class: "campo", r#for: "crear-buscar-ingrediente",
                                span { class: "campo-etiqueta", "{crate::i18n::ahora(Texto::BuscarIngrediente)}" }
                                input {
                                    id: "crear-buscar-ingrediente",
                                    class: "campo-control",
                                    r#type: "search",
                                    autocomplete: "off",
                                    placeholder: "{crate::i18n::ahora(Texto::BuscarIngrediente)}",
                                    value: "{filtro_ingrediente}",
                                    oninput: move |evento| filtro_ingrediente.set(evento.value()),
                                }
                            }

                            label { class: "campo", r#for: "crear-ingrediente",
                                span { class: "campo-etiqueta", "{crate::i18n::ahora(Texto::Ingredientes)}" }
                                select {
                                    id: "crear-ingrediente",
                                    class: "campo-control",
                                    value: "{seleccion}",
                                    onchange: move |evento| seleccion.set(evento.value()),

                                    option { value: "", "{crate::i18n::ahora(Texto::EligeIngredienteCorto)}" }
                                    for opcion in disponibles.iter() {
                                        option { key: "{opcion.id}", value: "{opcion.id}", "{opcion.nombre}" }
                                    }
                                }
                            }

                            if let Some(elegido) = catalogo_vec.iter().find(|opcion| opcion.id == seleccion()) {
                                div { class: "anadir-ficha",
                                    span {
                                        class: "anadir-ficha-punto",
                                        style: "background:{elegido.color.clone().unwrap_or_else(|| \"#B9A48C\".to_string())}",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "anadir-ficha-nombre", "{elegido.nombre}" }
                                    span { class: "anadir-ficha-cifra",
                                        if elegido.graduacion_pct > 0.0 {
                                            "{elegido.graduacion_pct:.0} % ABV"
                                        } else {
                                            "{elegido.unidad}"
                                        }
                                    }
                                }
                            }

                            div { class: "anadir-fila",
                                label { class: "campo", r#for: "crear-cantidad",
                                    span { class: "campo-etiqueta", "{crate::i18n::ahora(Texto::Cantidad)}" }
                                    span { class: "campo-unidad",
                                        input {
                                            id: "crear-cantidad",
                                            class: "campo-control",
                                            r#type: "text",
                                            inputmode: "decimal",
                                            autocomplete: "off",
                                            value: "{cantidad_nueva}",
                                            oninput: move |evento| cantidad_nueva.set(evento.value()),
                                        }
                                        span { "{unidad_nueva}" }
                                    }
                                }

                            button {
                                class: "primario",
                                r#type: "button",
                                disabled: seleccion().is_empty(),
                                onclick: move |_| {
                                    let id = seleccion();
                                    let Some(ingrediente) = catalogo_vec.iter().find(|opcion| opcion.id == id).cloned() else {
                                        return;
                                    };
                                    let cantidad = cantidad_nueva()
                                        .replace(',', ".")
                                        .parse::<f64>()
                                        .unwrap_or(CANTIDAD_INICIAL);
                                    let cantidad = if cantidad.is_finite() && cantidad > 0.0 {
                                        cantidad
                                    } else {
                                        CANTIDAD_INICIAL
                                    };

                                    capas.write().push(Capa { ingrediente, cantidad });
                                    seleccion.set(String::new());
                                    cantidad_nueva.set(CANTIDAD_INICIAL.to_string());
                                    error.set(None);
                                },
                                "{crate::i18n::ahora(Texto::AnadirBoton)}"
                            }
                            }
                        }
                }

                if sin_ingredientes {
                    p { class: "estado", "{crate::i18n::ahora(Texto::RecetaSinIngredientes)}" }
                } else {
                    ul { class: "ajustar-capas",
                        for (indice, capa) in capas_actuales.iter().enumerate() {
                            li { key: "{capa.ingrediente.id}", class: "ajustar-capa",
                                div { class: "ajustar-capa-cabeza",
                                    span {
                                        class: "ajustar-capa-punto",
                                        style: "background:{capa.ingrediente.color.clone().unwrap_or_else(|| \"#B9A48C\".to_string())}",
                                        "aria-hidden": "true",
                                    }
                                    span { class: "ajustar-capa-nombre", "{capa.ingrediente.nombre}" }
                                    span { class: "ajustar-capa-cifra",
                                        "{capa.cantidad:.0} {capa.ingrediente.unidad}"
                                    }
                                    button {
                                        class: "ajustar-capa-quitar",
                                        r#type: "button",
                                        aria_label: "{crate::i18n::ahora(Texto::Quitar)} {capa.ingrediente.nombre}",
                                        onclick: move |_| { capas.write().remove(indice); },
                                        "{crate::i18n::ahora(Texto::Quitar)}"
                                    }
                                }
                            }
                        }
                    }
                }

                if total_ml > capacidad && capacidad > 0.0 {
                    p { class: "ajustar-aviso ajustar-aviso--error", role: "alert",
                        "{crate::i18n::ahora(Texto::NoCabeEnElVaso)}"
                    }
                }
            }

            // Pasos
            section { class: "seccion", "aria-labelledby": "titulo-pasos-nuevos",
                p { id: "titulo-pasos-nuevos", class: "eyebrow", "{crate::i18n::ahora(Texto::PasosOpcionales)}" }

                for (indice, paso) in pasos_actuales.iter().enumerate() {
                    div { key: "{indice}", class: "ajustar-paso",
                        label { class: "ajustar-nombre", r#for: "paso-titulo-{indice}",
                            span { "Paso {indice + 1}" }
                            input {
                                id: "paso-titulo-{indice}",
                                class: "ajustar-select",
                                r#type: "text",
                                placeholder: "{crate::i18n::ahora(Texto::TituloDelPaso)}",
                                value: "{paso.titulo}",
                                oninput: move |evento| {
                                    if let Some(slot) = pasos.write().get_mut(indice) {
                                        slot.titulo = evento.value();
                                    }
                                },
                            }
                        }

                        textarea {
                            class: "barra-texto",
                            placeholder: "{crate::i18n::ahora(Texto::InstruccionDelPaso)}",
                            value: "{paso.instruccion}",
                            oninput: move |evento| {
                                if let Some(slot) = pasos.write().get_mut(indice) {
                                    slot.instruccion = evento.value();
                                }
                            },
                        }

                        button {
                            class: "ajustar-capa-quitar",
                            r#type: "button",
                            onclick: move |_| { pasos.write().remove(indice); },
                            "{crate::i18n::ahora(Texto::QuitarPaso)}"
                        }
                    }
                }

                button {
                    class: "chip",
                    r#type: "button",
                    onclick: move |_| {
                        pasos.write().push(Paso {
                            titulo: String::new(),
                            instruccion: String::new(),
                            temporizador_s: None,
                        });
                    },
                    "{crate::i18n::ahora(Texto::AgregarPaso)}"
                }
            }

            if let Some(mensaje) = error() {
                p { class: "ajustar-aviso ajustar-aviso--error", role: "alert", "{mensaje}" }
            }

            // Guardar
            div { class: "ajustar-acciones",
                button {
                    class: "primario",
                    r#type: "button",
                    disabled: *guardando.read(),
                    onclick: move |_| {
                        if *guardando.read() {
                            return;
                        }

                        let Some(token) = crate::sesion::token(&sesion) else {
                            error.set(Some(crate::i18n::ahora(Texto::IniciaSesionCrear).to_string()));
                            return;
                        };

                        // Datos del formulario: se revisan aquí para no gastar un viaje
                        let nombre_limpio = nombre().trim().to_string();
                        if nombre_limpio.is_empty() {
                            error.set(Some(crate::i18n::ahora(Texto::FaltaNombreReceta).to_string()));
                            return;
                        }
                        if familia().trim().is_empty() {
                            error.set(Some(crate::i18n::ahora(Texto::FaltaFamilia).to_string()));
                            return;
                        }
                        let Some(vaso) = vasos_para_guardar.get(*indice_vaso.read()).cloned() else {
                            error.set(Some(crate::i18n::ahora(Texto::FaltaVaso).to_string()));
                            return;
                        };
                        if capas.read().is_empty() {
                            error.set(Some(crate::i18n::ahora(Texto::RecetaSinIngredientes).to_string()));
                            return;
                        }

                        let datos = RecetaDesdeCero {
                            nombre: nombre_limpio,
                            tipo: tipo(),
                            familia: familia().trim().to_string(),
                            vaso_id: vaso.id.clone(),
                            capacidad_ml: vaso.vaso.capacidad_ml,
                            componentes: capas
                                .read()
                                .iter()
                                .map(|capa| ComponenteAgregado {
                                    ingrediente_id: capa.ingrediente.id.clone(),
                                    cantidad: capa.cantidad,
                                })
                                .collect(),
                            pasos: pasos
                                .read()
                                .iter()
                                .filter(|paso| !paso.instruccion.trim().is_empty())
                                .map(|paso| PasoNuevo {
                                    titulo: if paso.titulo.trim().is_empty() {
                                        crate::i18n::ahora(Texto::PasoSinTitulo).to_string()
                                    } else {
                                        paso.titulo.trim().to_string()
                                    },
                                    instruccion: paso.instruccion.trim().to_string(),
                                    temporizador_s: paso.temporizador_s,
                                })
                                .collect(),
                            id: crate::local::nuevo_id(),
                        };

                        guardando.set(true);
                        error.set(None);

                        spawn(async move {
                            let resultado =
                                crate::red::con_plazo(crear_receta(token, datos)).await;
                            guardando.set(false);

                            match resultado {
                                Ok(receta_id) => {
                                    // La receta nueva se abre en su ficha
                                    navegador.push(Rutas::Ficha { id: receta_id });
                                }
                                Err(causa) => error.set(Some(crate::red::mensaje(&causa))),
                            }
                        });
                    },
                    if *guardando.read() {
                        "{crate::i18n::ahora(Texto::Guardando)}"
                    } else {
                        "{crate::i18n::ahora(Texto::CrearRecetaBoton)}"
                    }
                }

                Link { to: Rutas::MisRecetas, class: "chip", "{crate::i18n::ahora(Texto::Cancelar)}" }
            }
        }
    }
}
