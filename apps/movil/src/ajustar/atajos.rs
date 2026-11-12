//! Atajos de C1: presets de fuerza y escala global en mililitros por comensal.

use dioxus::prelude::*;

use super::{CapaAgregada, FACTOR_MAXIMO, VOLUMEN_MAXIMO_ML};

/// Atajo de proporción que se aplica a todas las capas a la vez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preset {
    /// Vuelve a las cantidades publicadas.
    Original,
    /// Sube los ingredientes que aportan fuerza.
    Fuerte,
    /// Baja los ingredientes que aportan fuerza.
    Debil,
}

impl Preset {
    /// Presets visibles, en el orden en que se ofrecen.
    const TODOS: [Preset; 3] = [Preset::Original, Preset::Fuerte, Preset::Debil];

    /// Texto corto del botón.
    fn etiqueta(self) -> &'static str {
        match self {
            Preset::Original => "Original",
            Preset::Fuerte => "Más fuerte",
            Preset::Debil => "Más débil",
        }
    }
}

/// Presets de fuerza y deslizador de escala, que mueven todas las capas a la vez.
///
/// `total_base` es el volumen de la receta sin factores y `volumen_escala` el volumen por
/// comensal que muestra el deslizador.
#[component]
pub(super) fn Atajos(
    ingredientes: Vec<proporciones::Componente>,
    factores: Signal<Vec<f64>>,
    agregados: Signal<Vec<CapaAgregada>>,
    total_base: f64,
    volumen_escala: f64,
) -> Element {
    let mut factores = factores;
    let mut agregados = agregados;

    rsx! {
        section { class: "seccion", "aria-labelledby": "titulo-atajos",
            div { class: "seccion-cabeza",
                p { id: "titulo-atajos", class: "eyebrow", "Atajos de proporción" }
                span { class: "mini", "Se aplican a todas las capas" }
            }

            div { class: "ajustar-presets",
                for preset in Preset::TODOS {
                    {
                        let ingredientes = ingredientes.clone();
                        rsx! {
                            button {
                                key: "{preset.etiqueta()}",
                                class: "chip",
                                r#type: "button",
                                onclick: move |_| {
                                    let actuales = factores.read().clone();
                                    factores.set(factores_preset(preset, &ingredientes, &actuales));

                                    // Los ingredientes agregados siguen la misma regla que
                                    // las capas del origen para que el atajo sea coherente.
                                    let mut extras = agregados.read().clone();
                                    for extra in &mut extras {
                                        extra.factor = factor_preset(
                                            preset,
                                            &extra.nombre,
                                            extra.graduacion,
                                            extra.factor,
                                        );
                                    }
                                    agregados.set(extras);
                                },
                                "{preset.etiqueta()}"
                            }
                        }
                    }
                }
            }

            div { class: "ajustar-escala",
                label {
                    class: "ajustar-etiqueta",
                    r#for: "escala-ml",
                    "Escala"
                }
                input {
                    id: "escala-ml",
                    class: "ajustar-rango ajustar-rango--escala",
                    r#type: "range",
                    min: "0",
                    max: "{VOLUMEN_MAXIMO_ML:.0}",
                    step: "5",
                    value: "{volumen_escala:.0}",
                    disabled: total_base <= 0.0,
                    aria_label: "Volumen por comensal en mililitros",
                    "aria-valuetext": "{volumen_escala:.0} mililitros por comensal",
                    oninput: move |evento| {
                        let objetivo = evento
                            .value()
                            .parse::<f64>()
                            .unwrap_or(0.0)
                            .clamp(0.0, VOLUMEN_MAXIMO_ML);

                        // Un solo factor para todas las capas conserva la proporción de
                        // la receta mientras el volumen por comensal se mueve en mililitros.
                        if total_base > 0.0 {
                            let factor = (objetivo / total_base).clamp(0.0, FACTOR_MAXIMO);
                            let mut nuevos = factores.read().clone();
                            nuevos.fill(factor);
                            factores.set(nuevos);

                            let mut extras = agregados.read().clone();
                            for extra in &mut extras {
                                extra.factor = factor;
                            }
                            agregados.set(extras);
                        }
                    },
                }
                span { class: "ajustar-escala-valor", "{volumen_escala:.0} ml" }
            }

            p { class: "mini", "La escala termina en 500 ml por comensal y mueve todas las capas." }
        }
    }
}

/// Aplica un preset sobre los factores actuales y los deja dentro del rango del control.
fn factores_preset(
    preset: Preset,
    ingredientes: &[proporciones::Componente],
    actuales: &[f64],
) -> Vec<f64> {
    let mut factores = if actuales.len() == ingredientes.len() {
        actuales.to_vec()
    } else {
        vec![1.0; ingredientes.len()]
    };

    match preset {
        Preset::Original => factores.fill(1.0),
        Preset::Fuerte => {
            for (factor, ingrediente) in factores.iter_mut().zip(ingredientes) {
                if aporta_fuerza(&ingrediente.nombre, ingrediente.graduacion) {
                    *factor = (*factor * 1.25).clamp(0.0, FACTOR_MAXIMO);
                }
            }
        }
        Preset::Debil => {
            for (factor, ingrediente) in factores.iter_mut().zip(ingredientes) {
                if aporta_fuerza(&ingrediente.nombre, ingrediente.graduacion) {
                    *factor = (*factor * 0.75).clamp(0.0, FACTOR_MAXIMO);
                }
            }
        }
    }

    factores
}

/// Calcula el factor de una capa agregada para el preset elegido.
fn factor_preset(preset: Preset, nombre: &str, graduacion: f64, actual: f64) -> f64 {
    match preset {
        Preset::Original => 1.0,
        Preset::Fuerte if aporta_fuerza(nombre, graduacion) => {
            (actual * 1.25).clamp(0.0, FACTOR_MAXIMO)
        }
        Preset::Debil if aporta_fuerza(nombre, graduacion) => {
            (actual * 0.75).clamp(0.0, FACTOR_MAXIMO)
        }
        Preset::Fuerte | Preset::Debil => actual,
    }
}

/// Indica si la capa aporta fuerza: alcohol o la parte concentrada del café.
fn aporta_fuerza(nombre: &str, graduacion: f64) -> bool {
    graduacion > 0.0 || contiene(nombre, &["espresso", "café", "cafe", "cold brew"])
}

/// Compara sin distinguir mayúsculas y acepta cualquiera de las palabras.
fn contiene(nombre: &str, palabras: &[&str]) -> bool {
    let nombre = nombre.to_lowercase();
    palabras.iter().any(|palabra| nombre.contains(palabra))
}
