//! Ingredientes (B5): glosario del catalogo y equivalencias de unidades.
//!
//! La pantalla reune dos ayudas que no necesitan estar conectadas a una receta:
//! un glosario de los ingredientes del catalogo, agrupados por familia, y un
//! conversor de mililitros a onzas y cucharadas para quien prepara sin bascula.
//! Las sustituciones entre ingredientes quedan para una fase posterior (B5 completo).

use api::ingredientes::{IngredienteOpcion, catalogo};
use dioxus::prelude::*;

use crate::barra::{BarraInferior, Destino};
use crate::rutas::Rutas;

/// Hoja de estilos propia de la pantalla de ingredientes.
const CSS_INGREDIENTES: Asset = asset!("/assets/ingredientes.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Color de reserva para los ingredientes sin color propio en el catalogo.
const COLOR_RESERVA: &str = "#B9A48C";

/// Pantalla B5: carga el catalogo y muestra el glosario y las equivalencias.
#[component]
pub fn PantallaIngredientes() -> Element {
    let mut recurso = use_resource(catalogo);
    let busqueda = use_signal(String::new);

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_INGREDIENTES }

        main { class: "ingredientes",
            BarraSuperior {}

            header { class: "ingredientes-cabeza",
                p { class: "eyebrow", "Recetario" }
                h1 { "Ingredientes" }
                p { class: "ingredientes-sub", "El glosario de Ratio y las equivalencias para medir sin bascula." }
            }

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando el glosario..." }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos traer el catalogo: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| recurso.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(lista)) => rsx! {
                    Buscador { busqueda }
                    Glosario { lista: filtrar(&lista, &busqueda.read()) }
                },
            }

            Equivalencias {}
        }

        BarraInferior { activo: Destino::Biblioteca }
    }
}

/// Barra superior con la vuelta a la biblioteca.
#[component]
fn BarraSuperior() -> Element {
    rsx! {
        header { class: "ingredientes-barra",
            Link {
                to: Rutas::Biblioteca,
                class: "ingredientes-volver",
                aria_label: "Volver a la biblioteca",
                "< Biblioteca"
            }
            span { class: "ingredientes-marca", "Ratio" }
        }
    }
}

/// Buscador por nombre de ingrediente.
#[component]
fn Buscador(busqueda: Signal<String>) -> Element {
    let mut busqueda = busqueda;

    rsx! {
        form {
            class: "buscador",
            role: "search",
            onsubmit: move |evento| evento.prevent_default(),

            label { class: "solo-lector", r#for: "buscar-ingrediente", "Buscar ingredientes" }
            input {
                id: "buscar-ingrediente",
                r#type: "search",
                placeholder: "Buscar por nombre",
                autocomplete: "off",
                value: "{busqueda}",
                oninput: move |evento| busqueda.set(evento.value()),
            }
        }
    }
}

/// Glosario agrupado por familia: cada ingrediente con su color, unidad y alcohol.
#[component]
fn Glosario(lista: Vec<IngredienteOpcion>) -> Element {
    let grupos = agrupar(&lista);

    rsx! {
        section { class: "ingredientes-seccion", "aria-labelledby": "glosario-titulo",
            div { class: "ingredientes-seccion-cabeza",
                h2 { id: "glosario-titulo", "Glosario" }
                span { class: "ingredientes-cuenta", role: "status", "{lista.len()} ingredientes" }
            }

            if lista.is_empty() {
                p { class: "estado", "No hay ingredientes que coincidan con tu busqueda." }
            } else {
                for grupo in grupos {
                    section { class: "ingredientes-grupo",
                        h3 { "{tipo_legible(&grupo.0)}" }
                        ul { class: "ingredientes-lista",
                            for opcion in grupo.1 {
                                IngredienteFila { opcion }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Fila de un ingrediente: circulo de color, nombre y datos (unidad, alcohol, dilucion).
#[component]
fn IngredienteFila(opcion: IngredienteOpcion) -> Element {
    let color = opcion
        .color
        .clone()
        .unwrap_or_else(|| COLOR_RESERVA.to_string());

    rsx! {
        li { class: "ingrediente",
            span {
                class: "ingrediente-swatch",
                "aria-hidden": "true",
                style: "background:{color}",
            }
            span { class: "ingrediente-nombre", "{opcion.nombre}" }
            span { class: "ingrediente-meta", "{meta(&opcion)}" }
        }
    }
}

/// Equivalencias: conversor de ml a oz y cucharadas, y referencia de unidades.
#[component]
fn Equivalencias() -> Element {
    rsx! {
        section { class: "ingredientes-seccion", "aria-labelledby": "equivalencias-titulo",
            h2 { id: "equivalencias-titulo", "Equivalencias" }

            Conversor {}

            div { class: "referencia",
                p { class: "referencia-titulo", "Medidas de referencia" }
                ul { class: "referencia-lista",
                    li { span { class: "referencia-clave", "1 onza (oz)" } span { "= 29,57 ml" } }
                    li { span { class: "referencia-clave", "1 cucharada (cda)" } span { "= 15 ml" } }
                    li { span { class: "referencia-clave", "1 litro (L)" } span { "= 1000 ml" } }
                }
                p { class: "referencia-nota", "Partes: una proporcion, no una medida fija. Usa la misma medida (una onza, una copa...) para cada ingrediente de la receta." }
            }
        }
    }
}

/// Conversor: escribe mililitros y mira al instante las onzas y las cucharadas.
#[component]
fn Conversor() -> Element {
    let mut ml_texto = use_signal(|| "30".to_string());

    let ml: f64 = ml_texto
        .read()
        .parse::<f64>()
        .unwrap_or(0.0)
        .clamp(0.0, 100_000.0);

    rsx! {
        div { class: "conversor",
            form {
                class: "conversor-campo",
                onsubmit: move |evento| evento.prevent_default(),
                label { class: "solo-lector", r#for: "conversor-ml", "Cantidad en mililitros" }
                input {
                    id: "conversor-ml",
                    r#type: "number",
                    inputmode: "decimal",
                    min: "0",
                    step: "1",
                    value: "{ml_texto}",
                    oninput: move |evento| ml_texto.set(evento.value()),
                }
                span { class: "conversor-unidad", "ml" }
            }

            dl { class: "conversor-resultados",
                div { class: "conversor-resultado",
                    dt { "Mililitros" }
                    dd { "{formatear_ml(ml)}" }
                }
                div { class: "conversor-resultado",
                    dt { "Onzas" }
                    dd { "{formatear_oz(ml)}" }
                }
                div { class: "conversor-resultado",
                    dt { "Cucharadas" }
                    dd { "{formatear_cdas(ml)}" }
                }
            }
        }
    }
}

/// Datos secundarios de un ingrediente: unidad, alcohol y dilucion.
fn meta(opcion: &IngredienteOpcion) -> String {
    let mut partes: Vec<String> = Vec::new();
    partes.push(opcion.unidad.clone());

    if opcion.graduacion_pct > 0.0 {
        partes.push(format!("{:.0} % ABV", opcion.graduacion_pct));
    }

    if opcion.dilucion {
        partes.push("diluye".to_string());
    }

    partes.join(" · ")
}

/// Agrupa los ingredientes por familia conservando el orden del catalogo.
fn agrupar(lista: &[IngredienteOpcion]) -> Vec<(Option<String>, Vec<IngredienteOpcion>)> {
    let mut grupos: Vec<(Option<String>, Vec<IngredienteOpcion>)> = Vec::new();

    for opcion in lista {
        match grupos.last_mut() {
            Some((tipo, items)) if tipo == &opcion.tipo => items.push(opcion.clone()),
            _ => grupos.push((opcion.tipo.clone(), vec![opcion.clone()])),
        }
    }

    grupos
}

/// Nombre visible de una familia del catalogo; los sin familia caen en «Otros».
fn tipo_legible(tipo: &Option<String>) -> String {
    let texto = match tipo.as_deref() {
        None => "Otros".to_string(),
        Some(clave) => clave.replace('_', " "),
    };

    let mut letras = texto.chars();

    match letras.next() {
        Some(primera) => primera.to_uppercase().collect::<String>() + letras.as_str(),
        None => texto,
    }
}

/// Filtra el glosario por nombre.
fn filtrar(lista: &[IngredienteOpcion], busqueda: &str) -> Vec<IngredienteOpcion> {
    let texto = busqueda.trim().to_lowercase();

    lista
        .iter()
        .filter(|opcion| texto.is_empty() || opcion.nombre.to_lowercase().contains(&texto))
        .cloned()
        .collect()
}

/// Mililitros en un formato legible, con litros cuando la cantidad crece.
fn formatear_ml(ml: f64) -> String {
    if ml >= 1000.0 {
        format!("{:.2} L", ml / 1000.0)
    } else if ml.fract().abs() < 0.05 {
        format!("{ml:.0} ml")
    } else {
        format!("{ml:.1} ml")
    }
}

/// Mililitros convertidos a onzas liquidas.
fn formatear_oz(ml: f64) -> String {
    format!("{:.1} oz", ml / proporciones::OZ_ML)
}

/// Mililitros convertidos a cucharadas de cocina.
fn formatear_cdas(ml: f64) -> String {
    let cdas = ml / proporciones::CUCHARA_ML;

    if cdas.fract().abs() < 0.05 {
        format!("{cdas:.0} cdas")
    } else {
        format!("{cdas:.1} cdas")
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn opcion(nombre: &str, tipo: Option<&str>) -> IngredienteOpcion {
        IngredienteOpcion {
            id: nombre.to_string(),
            nombre: nombre.to_string(),
            unidad: "ml".to_string(),
            graduacion_pct: 0.0,
            color: None,
            dilucion: false,
            tipo: tipo.map(str::to_string),
        }
    }

    #[test]
    fn agrupa_los_ingredientes_por_familia() {
        let lista = vec![
            opcion("Espresso", Some("cafe")),
            opcion("Leche", Some("lacteos")),
            opcion("Ron", Some("alcohol")),
            opcion("Agua", None),
        ];
        let grupos = agrupar(&lista);

        assert_eq!(grupos.len(), 4);
        assert_eq!(tipo_legible(&grupos[0].0), "Cafe");
        assert_eq!(tipo_legible(&grupos[3].0), "Otros");
    }

    #[test]
    fn filtra_por_nombre_sin_distinguir_mayusculas() {
        let lista = vec![
            opcion("Espresso", None),
            opcion("Leche", None),
            opcion("Agua", None),
        ];
        let filtradas = filtrar(&lista, "le");

        assert_eq!(filtradas.len(), 1);
        assert_eq!(filtradas[0].nombre, "Leche");
    }

    #[test]
    fn convierte_mililitros_a_otras_unidades() {
        assert_eq!(formatear_ml(1500.0), "1.50 L");
        assert_eq!(formatear_oz(proporciones::OZ_ML), "1.0 oz");
        assert_eq!(formatear_cdas(30.0), "2 cdas");
    }
}
