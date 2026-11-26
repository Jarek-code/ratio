//! Ajustes (E4) y Onboarding (E5) de la app, parte de la app.
//!
//! Los ajustes viven en un archivo JSON de la carpeta privada y se comparten por contexto:
//! la unidad en que se muestran las cantidades y si ya se vio la bienvenida. La bienvenida
//! explica la app y avisa que el contenido con alcohol es solo para mayores de edad.

// LIBRERÍAS

use crate::i18n::Idioma; // Idioma de la interfaz
use crate::i18n::Texto;
use crate::i18n::con; // Texto con huecos rellenos
use crate::i18n::t; // Texto en el idioma elegido
use crate::rutas::Rutas; // Texto traducible
use dioxus::prelude::*; // Componentes, señales y contexto
use proporciones::Equivalencia; // Cantidad de una capa en todas las unidades
use serde::Deserialize; // Lectura de los ajustes guardados
use serde::Serialize; // Escritura de los ajustes guardados

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior

// CONSTANTES Y VARIABLES GLOBALES

/// Archivo de la carpeta privada con los ajustes.
const ARCHIVO_AJUSTES: &str = "ajustes.json";

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// ESTRUCTURAS

/// Unidad en la que se muestran las cantidades de las recetas.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Unidad {
    /// Mililitros, la unidad del motor.
    #[default]
    Mililitros,
    /// Onzas líquidas de EE. UU.
    Onzas,
    /// Partes respecto al ingrediente principal más pequeño.
    Partes,
}

/// Tema de color de la app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Tema {
    /// Fondo crema y tinta oscura, el de siempre.
    #[default]
    Claro,
    /// Fondo oscuro y texto claro, para poca luz.
    Oscuro,
}

impl Tema {
    /// Los temas en el orden en que se ofrecen.
    pub const TODOS: [Tema; 2] = [Tema::Claro, Tema::Oscuro];

    /// Nombre visible del tema.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Tema::Claro => "Claro",
            Tema::Oscuro => "Oscuro",
        }
    }

    /// Valor del atributo `data-theme` que leen las hojas de estilo.
    pub fn atributo(self) -> &'static str {
        match self {
            Tema::Claro => "claro",
            Tema::Oscuro => "oscuro",
        }
    }
}

/// Ajustes de la app guardados en el dispositivo.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ajustes {
    /// Unidad de las cantidades.
    pub unidad: Unidad,
    /// Tema de color.
    pub tema: Tema,
    /// Idioma de la interfaz.
    pub idioma: Idioma,
    /// Si ya se mostró la bienvenida con el aviso de edad.
    pub bienvenida_vista: bool,
}

/// Señal con los ajustes, compartida por contexto.
pub type SenalAjustes = Signal<Ajustes>;

// FUNCIONES

impl Unidad {
    /// Las unidades en el orden en que se ofrecen.
    pub const TODAS: [Unidad; 3] = [Unidad::Mililitros, Unidad::Onzas, Unidad::Partes];

    /// Nombre visible de la unidad.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Unidad::Mililitros => "Mililitros (ml)",
            Unidad::Onzas => "Onzas (oz)",
            Unidad::Partes => "Partes",
        }
    }

    /// Cantidad de una capa escrita en esta unidad, en el idioma elegido.
    pub fn cantidad(self, equivalencia: &Equivalencia) -> String {
        let idioma = crate::i18n::actual();

        match self {
            Unidad::Mililitros => crate::i18n::con(
                idioma,
                crate::i18n::Texto::MililitrosCorto,
                &[&format!("{:.0}", equivalencia.ml)],
            ),
            Unidad::Onzas => crate::i18n::con(
                idioma,
                crate::i18n::Texto::OnzasCorto,
                &[&format!("{:.1}", equivalencia.oz)],
            ),
            Unidad::Partes => crate::i18n::con(
                idioma,
                crate::i18n::Texto::PartesCorto,
                &[&format!("{:.1}", equivalencia.partes)],
            ),
        }
    }
}

/// Lee los ajustes guardados; sin archivo se usan los de fábrica.
pub fn cargar() -> Ajustes {
    crate::local::leer_valor(ARCHIVO_AJUSTES).unwrap_or_default()
}

/// Cambia los ajustes en memoria y en disco.
fn cambiar(ajustes: &mut SenalAjustes, nuevos: Ajustes) {
    crate::local::escribir(ARCHIVO_AJUSTES, &nuevos);
    // El idioma se aplica en el acto: los textos que se arman fuera de un componente lo leen
    crate::i18n::fijar_actual(nuevos.idioma);
    ajustes.set(nuevos);
}

/// Bienvenida de la primera vez con el aviso de edad (E5).
///
/// Se muestra antes del acceso hasta que la persona toca «Empezar».
#[component]
pub fn Bienvenida() -> Element {
    let mut ajustes = use_context::<SenalAjustes>(); // Para recordar que ya se vio
    let mut paso_sel = use_signal(|| 0_usize); // Pantalla de la bienvenida
    let idioma = ajustes.read().idioma; // Idioma de los textos

    // Contenido de cada pantalla: título y texto, en el idioma elegido
    let pantallas = [
        (Texto::BienvenidaTitulo1, Texto::BienvenidaTexto1),
        (Texto::BienvenidaTitulo2, Texto::BienvenidaTexto2),
        (Texto::BienvenidaTitulo3, Texto::BienvenidaTexto3),
    ];
    let paso = paso_sel().min(pantallas.len() - 1);
    let (titulo, texto) = pantallas[paso];
    let ultimo = paso + 1 == pantallas.len();

    rsx! {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir bienvenida",
            "aria-labelledby": "titulo-bienvenida",

            p { class: "eyebrow", "{con(idioma, Texto::BienvenidaPaso, &[&(paso + 1).to_string(), &pantallas.len().to_string()])}" }
            h1 { id: "titulo-bienvenida", "{t(idioma, titulo)}" }
            p { class: "bienvenida-texto", "{t(idioma, texto)}" }

            // Última pantalla: el aviso de edad se presenta como alerta
            if ultimo
            {
                p { class: "aviso-pendientes", role: "note", "{t(idioma, Texto::BienvenidaAvisoEdad)}" }
            }

            div
            {
                class: "acciones-fila",

                if paso > 0
                {
                    button { class: "chip", r#type: "button", onclick: move |_| paso_sel.set(paso - 1), "{t(idioma, Texto::Atras)}" }
                }
                button
                {
                    class: "primario",
                    r#type: "button",
                    onclick: move |_|
                    {
                        // En la última pantalla se recuerda que ya se vio y se sigue al acceso
                        if ultimo
                        {
                            let mut nuevos = ajustes();
                            nuevos.bienvenida_vista = true;
                            cambiar(&mut ajustes, nuevos);
                        }
                        else
                        {
                            paso_sel.set(paso + 1);
                        }
                    },
                    if ultimo { "{t(idioma, Texto::Empezar)}" } else { "{t(idioma, Texto::Siguiente)}" }
                }
            }
        }
    }
}

/// Pantalla de ajustes: unidades, tema e idioma (E4).
#[component]
pub fn PantallaAjustes() -> Element {
    let mut ajustes = use_context::<SenalAjustes>(); // Ajustes que se editan
    let navegador = navigator(); // Para volver a la pantalla de donde se vino
    let unidad_actual = ajustes().unidad;
    let tema_actual = ajustes().tema;
    let idioma = ajustes().idioma;

    rsx! {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                button {
                    class: "volver",
                    r#type: "button",
                    aria_label: "{t(idioma, Texto::VolverAtras)}",
                    onclick: move |_| navegador.go_back(),
                    "‹ {t(idioma, Texto::Atras)}"
                }
            }

            p { class: "eyebrow", "{t(idioma, Texto::DestinoPerfil)}" }
            h1 { "{t(idioma, Texto::Ajustes)}" }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-unidades",

                p { id: "titulo-unidades", class: "eyebrow", "{t(idioma, Texto::Unidades)}" }
                p { class: "nota", "{t(idioma, Texto::UnidadesNota)}" }

                div
                {
                    class: "chips",
                    role: "radiogroup",
                    "aria-labelledby": "titulo-unidades",

                    for unidad in Unidad::TODAS
                    {
                        button
                        {
                            key: "{unidad.etiqueta()}",
                            class: if unidad == unidad_actual { "chip chip--activo" } else { "chip" },
                            r#type: "button",
                            role: "radio",
                            "aria-checked": if unidad == unidad_actual { "true" } else { "false" },
                            onclick: move |_|
                            {
                                let mut nuevos = ajustes();
                                nuevos.unidad = unidad;
                                cambiar(&mut ajustes, nuevos);
                            },
                            "{t(idioma, texto_unidad(unidad))}"
                        }
                    }
                }
            }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-tema",

                p { id: "titulo-tema", class: "eyebrow", "{t(idioma, Texto::Tema)}" }
                p { class: "nota", "{t(idioma, Texto::TemaNota)}" }

                div
                {
                    class: "chips",
                    role: "radiogroup",
                    "aria-labelledby": "titulo-tema",

                    for tema in Tema::TODOS
                    {
                        button
                        {
                            key: "{tema.etiqueta()}",
                            class: if tema == tema_actual { "chip chip--activo" } else { "chip" },
                            r#type: "button",
                            role: "radio",
                            "aria-checked": if tema == tema_actual { "true" } else { "false" },
                            onclick: move |_|
                            {
                                let mut nuevos = ajustes();
                                nuevos.tema = tema;
                                cambiar(&mut ajustes, nuevos);
                            },
                            "{t(idioma, texto_tema(tema))}"
                        }
                    }
                }
            }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-idioma",

                p { id: "titulo-idioma", class: "eyebrow", "{t(idioma, Texto::IdiomaSeccion)}" }
                p { class: "nota", "{t(idioma, Texto::IdiomaNota)}" }

                div
                {
                    class: "chips",
                    role: "radiogroup",
                    "aria-labelledby": "titulo-idioma",

                    for opcion in Idioma::TODOS
                    {
                        button
                        {
                            key: "{opcion.clave()}",
                            class: if opcion == idioma { "chip chip--activo" } else { "chip" },
                            r#type: "button",
                            role: "radio",
                            "aria-checked": if opcion == idioma { "true" } else { "false" },
                            onclick: move |_|
                            {
                                let mut nuevos = ajustes();
                                nuevos.idioma = opcion;
                                cambiar(&mut ajustes, nuevos);
                            },
                            // Cada idioma se escribe en el suyo, para que se reconozca
                            "{opcion.etiqueta()}"
                        }
                    }
                }
            }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-bienvenida-ajustes",

                p { id: "titulo-bienvenida-ajustes", class: "eyebrow", "{t(idioma, Texto::BienvenidaSeccion)}" }
                button
                {
                    class: "chip",
                    r#type: "button",
                    onclick: move |_|
                    {
                        let mut nuevos = ajustes();
                        nuevos.bienvenida_vista = false;
                        cambiar(&mut ajustes, nuevos);
                    },
                    "{t(idioma, Texto::VolverBienvenida)}"
                }
            }
        }

        SeccionCuenta {}

        BarraInferior { activo: Destino::Perfil }
    }
}

/// Cuenta: alias, declaración de edad y cierre de sesión.
#[component]
fn SeccionCuenta() -> Element {
    let sesion = use_context::<crate::sesion::SenalSesion>();
    let mut ajustes = use_context::<SenalAjustes>();
    let navegador = navigator();
    let idioma = ajustes().idioma;

    // El alias se propone con lo que ya tiene la sesión guardada
    let alias_guardado = crate::sesion::alias(&sesion).unwrap_or_default();
    let mut alias = use_signal(|| alias_guardado.clone());
    let mut adulto = use_signal(|| crate::sesion::es_adulto(&sesion));
    let mut guardando = use_signal(|| false);
    let mut aviso = use_signal(|| None::<String>);

    // Sin sesión no hay cuenta que editar
    if sesion.read().is_none() {
        return rsx! {
            section { class: "seccion",
                p { class: "eyebrow", "{t(idioma, Texto::Cuenta)}" }
                p { class: "nota", "{t(idioma, Texto::CuentaSinSesion)}" }
                Link { to: Rutas::Entrar, class: "primario", "{t(idioma, Texto::Entrar)}" }
            }
        };
    }

    rsx! {
        section { class: "seccion", "aria-labelledby": "titulo-cuenta",
            p { id: "titulo-cuenta", class: "eyebrow", "{t(idioma, Texto::Cuenta)}" }
            p { class: "nota", "{t(idioma, Texto::CuentaNota)}" }

            label { class: "ajustar-nombre", r#for: "cuenta-alias",
                span { "{t(idioma, Texto::Alias)}" }
                input {
                    id: "cuenta-alias",
                    class: "ajustar-select",
                    r#type: "text",
                    maxlength: "32",
                    value: "{alias}",
                    oninput: move |evento| alias.set(evento.value()),
                }
            }

            label { class: "chip",
                input {
                    r#type: "checkbox",
                    checked: adulto(),
                    onchange: move |evento| adulto.set(evento.checked()),
                }
                span { "{t(idioma, Texto::MayorDeEdadCuenta)}" }
            }

            div { class: "ajustar-acciones",
                button {
                    class: "primario",
                    r#type: "button",
                    disabled: guardando(),
                    onclick: move |_| {
                        let Some(token) = crate::sesion::token(&sesion) else {
                            aviso.set(Some(crate::i18n::ahora(Texto::IniciaSesionCuenta).to_string()));
                            return;
                        };
                        if guardando() {
                            return;
                        }

                        let datos = api::perfil::CuentaPropia {
                            alias: alias.read().clone(),
                            mayor_de_edad: adulto(),
                        };

                        guardando.set(true);
                        aviso.set(None);

                        spawn(async move {
                            match crate::red::con_plazo(api::perfil::actualizar_cuenta(token, datos)).await {
                                Ok(cuenta) => {
                                    crate::sesion::renombrar(&mut sesion.clone(), &cuenta.alias, cuenta.mayor_de_edad);
                                    aviso.set(Some(crate::i18n::ahora(Texto::CuentaGuardada).to_string()));
                                }
                                Err(causa) => aviso.set(Some(crate::i18n::con(
                                    crate::i18n::actual(),
                                    Texto::NoSeGuardoCuenta,
                                    &[&crate::red::mensaje(&causa)],
                                ))),
                            }
                            guardando.set(false);
                        });
                    },
                    if guardando() { "{t(idioma, Texto::Guardando)}" } else { "{t(idioma, Texto::GuardarCambios)}" }
                }

                button {
                    class: "chip",
                    r#type: "button",
                    onclick: move |_| {
                        crate::sesion::salir(&mut sesion.clone());
                        navegador.push(Rutas::Entrar);
                    },
                    "{t(idioma, Texto::Salir)}"
                }
            }

            if let Some(texto) = aviso() {
                p { class: "ajustar-aviso", role: "status", "{texto}" }
            }
        }
    }
}

/// Texto de una unidad de medida.
fn texto_unidad(unidad: Unidad) -> Texto {
    match unidad {
        Unidad::Mililitros => Texto::Mililitros,
        Unidad::Onzas => Texto::Onzas,
        Unidad::Partes => Texto::Partes,
    }
}

/// Texto de un tema de color.
fn texto_tema(tema: Tema) -> Texto {
    match tema {
        Tema::Claro => Texto::Claro,
        Tema::Oscuro => Texto::Oscuro,
    }
}
