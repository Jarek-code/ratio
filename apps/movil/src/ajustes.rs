//! Ajustes (E4) y Onboarding (E5) de la app, parte de la app.
//!
//! Los ajustes viven en un archivo JSON de la carpeta privada y se comparten por contexto:
//! la unidad en que se muestran las cantidades y si ya se vio la bienvenida. La bienvenida
//! explica la app y avisa que el contenido con alcohol es solo para mayores de edad.

// LIBRERÍAS

use dioxus::prelude::*; // Componentes, señales y contexto
use proporciones::Equivalencia; // Cantidad de una capa en todas las unidades
use serde::Deserialize; // Lectura de los ajustes guardados
use serde::Serialize; // Escritura de los ajustes guardados

use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::rutas::Rutas; // Rutas para volver al perfil

// CONSTANTES Y VARIABLES GLOBALES

/// Archivo de la carpeta privada con los ajustes.
const ARCHIVO_AJUSTES: &str = "ajustes.json";

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

// ESTRUCTURAS

/// Unidad en la que se muestran las cantidades de las recetas.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Unidad
{
    /// Mililitros, la unidad del motor.
    #[default]
    Mililitros,
    /// Onzas líquidas de EE. UU.
    Onzas,
    /// Partes respecto al ingrediente principal más pequeño.
    Partes,
}

/// Ajustes de la app guardados en el dispositivo.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ajustes
{
    /// Unidad de las cantidades.
    pub unidad: Unidad,
    /// Si ya se mostró la bienvenida con el aviso de edad.
    pub bienvenida_vista: bool,
}

/// Señal con los ajustes, compartida por contexto.
pub type SenalAjustes = Signal<Ajustes>;

// FUNCIONES

impl Unidad
{
    /// Las unidades en el orden en que se ofrecen.
    pub const TODAS: [Unidad; 3] = [Unidad::Mililitros, Unidad::Onzas, Unidad::Partes];

    /// Nombre visible de la unidad.
    pub fn etiqueta(self) -> &'static str
    {
        match self
        {
            Unidad::Mililitros => "Mililitros (ml)",
            Unidad::Onzas => "Onzas (oz)",
            Unidad::Partes => "Partes",
        }
    }

    /// Cantidad de una capa escrita en esta unidad, por ejemplo «80 ml», «2.7 oz» o «4 partes».
    pub fn cantidad(self, equivalencia: &Equivalencia) -> String
    {
        match self
        {
            Unidad::Mililitros => format!("{:.0} ml", equivalencia.ml),
            Unidad::Onzas => format!("{:.1} oz", equivalencia.oz),
            Unidad::Partes => format!("{:.1} partes", equivalencia.partes),
        }
    }
}

/// Lee los ajustes guardados; sin archivo se usan los de fábrica.
pub fn cargar() -> Ajustes
{
    crate::local::leer_valor(ARCHIVO_AJUSTES).unwrap_or_default()
}

/// Cambia los ajustes en memoria y en disco.
fn cambiar(ajustes: &mut SenalAjustes, nuevos: Ajustes)
{
    crate::local::escribir(ARCHIVO_AJUSTES, &nuevos);
    ajustes.set(nuevos);
}

/// Bienvenida de la primera vez con el aviso de edad (E5).
///
/// Se muestra antes del acceso hasta que la persona toca «Empezar».
#[component]
pub fn Bienvenida() -> Element
{
    let mut ajustes = use_context::<SenalAjustes>(); // Para recordar que ya se vio
    let mut paso_sel = use_signal(|| 0_usize); // Pantalla de la bienvenida

    // Contenido de cada pantalla: título y texto
    let pantallas = [
        ("La proporción se ve en el vaso", "Cada receta se dibuja como capas dentro de su vaso real, con su porcentaje y sus mililitros."),
        ("Ajusta y haz tu versión", "Cambia cantidades, comensales o vaso; guarda tu versión y publícala en La Barra con crédito al original."),
        ("Contenido con alcohol", "Los cócteles solo se muestran si al crear tu cuenta declaras que eres mayor de edad. Bebe con responsabilidad."),
    ];
    let paso = paso_sel().min(pantallas.len() - 1);
    let (titulo, texto) = pantallas[paso];
    let ultimo = paso + 1 == pantallas.len();

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir bienvenida",
            "aria-labelledby": "titulo-bienvenida",

            p { class: "eyebrow", "Bienvenida · {paso + 1} de {pantallas.len()}" }
            h1 { id: "titulo-bienvenida", "{titulo}" }
            p { class: "bienvenida-texto", "{texto}" }

            // Última pantalla: el aviso de edad se presenta como alerta
            if ultimo
            {
                p { class: "aviso-pendientes", role: "note", "Ratio no vende alcohol ni verifica tu edad: confía en tu declaración." }
            }

            div
            {
                class: "acciones-fila",

                if paso > 0
                {
                    button { class: "chip", r#type: "button", onclick: move |_| paso_sel.set(paso - 1), "Atrás" }
                }
                button
                {
                    class: "primario",
                    r#type: "button",
                    onclick: move |_|
                    {
                        // Última pantalla: recordar que se vio; si no, avanzar
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
                    if ultimo { "Empezar" } else { "Siguiente" }
                }
            }
        }
    }
}

/// Pantalla de ajustes: unidad de las cantidades e idioma (E4).
#[component]
pub fn PantallaAjustes() -> Element
{
    let mut ajustes = use_context::<SenalAjustes>(); // Ajustes que se editan
    let actual = ajustes().unidad;

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::Perfil, class: "enlace", aria_label: "Volver al perfil", "‹ Perfil" }
            }

            p { class: "eyebrow", "Perfil" }
            h1 { "Ajustes" }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-unidades",

                p { id: "titulo-unidades", class: "eyebrow", "Unidades" }
                p { class: "nota", "Cómo se muestran las cantidades en la ficha y en las equivalencias." }

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
                            class: if unidad == actual { "chip chip--activo" } else { "chip" },
                            r#type: "button",
                            role: "radio",
                            "aria-checked": if unidad == actual { "true" } else { "false" },
                            onclick: move |_|
                            {
                                let mut nuevos = ajustes();
                                nuevos.unidad = unidad;
                                cambiar(&mut ajustes, nuevos);
                            },
                            "{unidad.etiqueta()}"
                        }
                    }
                }
            }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-idioma",

                p { id: "titulo-idioma", class: "eyebrow", "Idioma" }
                p { "Español" }
                p { class: "nota", "Por ahora Ratio solo está en español." }
            }

            section
            {
                class: "seccion",
                "aria-labelledby": "titulo-bienvenida-ajustes",

                p { id: "titulo-bienvenida-ajustes", class: "eyebrow", "Bienvenida" }
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
                    "Volver a ver la bienvenida al salir"
                }
            }
        }

        BarraInferior { activo: Destino::Perfil }
    }
}
