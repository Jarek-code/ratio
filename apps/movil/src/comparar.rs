//! Comparar dos recetas lado a lado con sus datos y sus diferencias.

use dioxus::prelude::*;

/// Hoja de estilos propia de la pantalla comparar.
const CSS_COMPARAR: Asset = asset!("/assets/comparar.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Los nutrientes que se comparan, en el orden en que se muestran.
const NUTRIENTES: [Nutriente; 6] = [
    Nutriente {
        nombre: "Kcal",
        campo: Campo::Kcal,
    },
    Nutriente {
        nombre: "Proteína",
        campo: Campo::Proteina,
    },
    Nutriente {
        nombre: "Carbohidratos",
        campo: Campo::Carbohidratos,
    },
    Nutriente {
        nombre: "Grasa",
        campo: Campo::Grasa,
    },
    Nutriente {
        nombre: "Cafeína",
        campo: Campo::Cafeina,
    },
    Nutriente {
        nombre: "Azúcar",
        campo: Campo::Azucar,
    },
];

/// Un nutriente del cuadro de diferencias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Nutriente {
    nombre: &'static str,
    campo: Campo,
}

/// Campo numérico del perfil de una receta mock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Campo {
    Kcal,
    Proteina,
    Carbohidratos,
    Grasa,
    Cafeina,
    Azucar,
}

impl Campo {
    /// Unidad del dato, para la tabla.
    fn unidad(self) -> &'static str {
        match self {
            Campo::Kcal => "kcal",
            Campo::Cafeina => "mg",
            _ => "g",
        }
    }
}

/// Receta de ejemplo, hardcodeada mientras no hay datos reales.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RecetaMock {
    nombre: &'static str,
    kcal: f32,
    proteina: f32,
    carbohidratos: f32,
    grasa: f32,
    cafeina: f32,
    azucar: f32,
    /// Nivel de llenado de la taza, en porcentaje.
    llenado: u32,
}

impl RecetaMock {
    /// Valor del campo pedido.
    fn valor(&self, campo: Campo) -> f32 {
        match campo {
            Campo::Kcal => self.kcal,
            Campo::Proteina => self.proteina,
            Campo::Carbohidratos => self.carbohidratos,
            Campo::Grasa => self.grasa,
            Campo::Cafeina => self.cafeina,
            Campo::Azucar => self.azucar,
        }
    }
}

/// Catálogo mock para poblar los dos selectores.
const RECETAS_MOCK: [RecetaMock; 4] = [
    RecetaMock {
        nombre: "Espresso doble",
        kcal: 18.0,
        proteina: 1.2,
        carbohidratos: 2.6,
        grasa: 0.2,
        cafeina: 128.0,
        azucar: 0.0,
        llenado: 35,
    },
    RecetaMock {
        nombre: "Latte avellana",
        kcal: 220.0,
        proteina: 9.4,
        carbohidratos: 26.0,
        grasa: 8.1,
        cafeina: 75.0,
        azucar: 24.0,
        llenado: 90,
    },
    RecetaMock {
        nombre: "Frío de vainilla",
        kcal: 165.0,
        proteina: 6.8,
        carbohidratos: 22.5,
        grasa: 4.9,
        cafeina: 95.0,
        azucar: 19.0,
        llenado: 80,
    },
    RecetaMock {
        nombre: "Cortado de cardamomo",
        kcal: 64.0,
        proteina: 3.5,
        carbohidratos: 6.2,
        grasa: 3.1,
        cafeina: 64.0,
        azucar: 5.4,
        llenado: 55,
    },
];

/// Comparador de recetas: dos tazas lado a lado y su tabla de diferencias.
#[component]
pub fn PantallaComparar() -> Element {
    let mut receta_a = use_signal(|| 0_usize);
    let mut receta_b = use_signal(|| 1_usize);

    let indice_a = receta_a().min(RECETAS_MOCK.len() - 1);
    let indice_b = receta_b().min(RECETAS_MOCK.len() - 1);
    let a = &RECETAS_MOCK[indice_a];
    let b = &RECETAS_MOCK[indice_b];

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_COMPARAR }

        main { class: "cmp",
            section { class: "cmp-tarjeta",
                h1 { "Comparar" }
                p { class: "cmp-subtitulo", "Pon dos recetas frente a frente." }

                div { class: "cmp-selectores",
                    div { class: "cmp-selector",
                        label { class: "cmp-selector-etiqueta", r#for: "cmp-a", "Receta A" }
                        select {
                            id: "cmp-a",
                            value: receta_a,                            onchange: move |e| receta_a.set(seleccion(&e.value())),
                            for (indice, receta) in RECETAS_MOCK.iter().enumerate() {
                                option { value: "{indice}", "{receta.nombre}" }
                            }
                        }
                    }
                    div { class: "cmp-selector",
                        label { class: "cmp-selector-etiqueta", r#for: "cmp-b", "Receta B" }
                        select {
                            id: "cmp-b",
                            value: receta_b,
                            onchange: move |e| receta_b.set(seleccion(&e.value())),
                            for (indice, receta) in RECETAS_MOCK.iter().enumerate() {
                                option { value: "{indice}", "{receta.nombre}" }
                            }
                        }
                    }
                }
            }

            div { class: "cmp-tazas",
                Taza { nombre: a.nombre, llenado: a.llenado, kcal: a.kcal, cafeina: a.cafeina, activa: true }
                Taza { nombre: b.nombre, llenado: b.llenado, kcal: b.kcal, cafeina: b.cafeina, activa: false }
            }

            section { class: "cmp-tarjeta cmp-diferencias",
                h2 { "Diferencias · A frente a B" }

                div { class: "cmp-tabla",
                    div { class: "cmp-fila cmp-fila-cabecera",
                        span { class: "cmp-nutriente", "Nutriente" }
                        span { class: "cmp-valor", "A" }
                        span { class: "cmp-valor", "B" }
                        span { class: "cmp-delta", "Δ" }
                    }

                    for nutriente in NUTRIENTES {
                        Diferencia { a, b, nutriente: nutriente }
                    }
                }
            }
        }
    }
}

/// Una taza con su nivel de llenado y sus datos principales.
#[component]
fn Taza(nombre: &'static str, llenado: u32, kcal: f32, cafeina: f32, activa: bool) -> Element {
    rsx! {
        article {
            class: if activa { "cmp-taza cmp-taza--activa" } else { "cmp-taza" },
            h3 { class: "cmp-taza-nombre", "{nombre}" }

            div {
                class: "cmp-vaso",
                role: "img",
                "aria-label": "{nombre} al {llenado} %",
                div { class: "cmp-vaso-cuerpo" }
                div { class: "cmp-vaso-inner" }
                div { class: "cmp-vaso-cafe", style: "--llenado:{llenado}%" }
                div { class: "cmp-vaso-asa" }
            }

            p { class: "cmp-taza-dato",
                strong { "{kcal:.0} kcal" }
                " · {cafeina:.0} mg"
            }
        }
    }
}

/// Una fila del cuadro de diferencias, con la delta y su flecha.
#[component]
fn Diferencia(a: &'static RecetaMock, b: &'static RecetaMock, nutriente: Nutriente) -> Element {
    let campo = nutriente.campo;
    let va = a.valor(campo);
    let vb = b.valor(campo);
    let delta = va - vb;

    let (clase, texto, flecha) = if delta.abs() < 0.05 {
        ("cmp-delta--igual", "igual".to_string(), "—")
    } else if delta > 0.0 {
        ("cmp-delta--mayor", format!("+{delta:.1}"), "↑")
    } else {
        ("cmp-delta--menor", format!("{delta:.1}"), "↓")
    };

    rsx! {
        div { class: "cmp-fila",
            span { class: "cmp-nutriente", "{nutriente.nombre}" }
            span { class: "cmp-valor", "{va:.1} {campo.unidad()}" }
            span { class: "cmp-valor", "{vb:.1} {campo.unidad()}" }
            span { class: "cmp-delta {clase}", "{flecha} {texto}" }
        }
    }
}

/// Índice de la opción elegida, con los huecos al final.
fn seleccion(texto: &str) -> usize {
    texto
        .parse::<usize>()
        .unwrap_or(RECETAS_MOCK.len() - 1)
        .min(RECETAS_MOCK.len() - 1)
}
