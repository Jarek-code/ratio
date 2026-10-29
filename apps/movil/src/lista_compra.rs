//! Pantalla C7: convierte las recetas elegidas en una lista para el mercado.
//!
//! La pantalla no vende ni cobra nada: suma los ingredientes de las recetas
//! seleccionadas, ajusta por comensales y los agrupa por tipo para que la compra
//! se pueda seguir con el dedo. Las casillas viven en memoria porque la lista es
//! una ayuda de preparacion, no un pedido.

use std::collections::HashMap;

use api::biblioteca::{RecetaResumen, catalogo};
use dioxus::prelude::*;

use crate::barra::{BarraInferior, Destino};
use crate::biblioteca::familia_legible;
use crate::rutas::Rutas;

/// Hoja de estilos propia de la lista de compra.
const CSS_LISTA_COMPRA: Asset = asset!("/assets/lista_compra.css");
/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Maximo de comensales que permite el ajuste rapido.
const COMENSALES_MAXIMOS: u32 = 12;

/// Unidad en la que se muestran las cantidades de la lista.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unidad {
    /// Mililitros; se abrevia a litros cuando la lista crece.
    Mililitros,
    /// Onzas liquidas.
    Onzas,
    /// Cucharadas de cocina.
    Cucharadas,
}

impl Unidad {
    /// Unidades disponibles en el selector.
    const TODAS: [Unidad; 3] = [Unidad::Mililitros, Unidad::Onzas, Unidad::Cucharadas];

    /// Texto corto del selector.
    fn etiqueta(self) -> &'static str {
        match self {
            Unidad::Mililitros => "ml",
            Unidad::Onzas => "oz",
            Unidad::Cucharadas => "cdas",
        }
    }

    /// Texto accesible del selector.
    fn nombre(self) -> &'static str {
        match self {
            Unidad::Mililitros => "mililitros",
            Unidad::Onzas => "onzas",
            Unidad::Cucharadas => "cucharadas",
        }
    }

    /// Convierte mililitros a la unidad elegida.
    fn convertir(self, ml: f64) -> f64 {
        match self {
            Unidad::Mililitros => ml,
            Unidad::Onzas => ml / proporciones::OZ_ML,
            Unidad::Cucharadas => ml / proporciones::CUCHARA_ML,
        }
    }

    /// Cantidad de una linea de la lista.
    fn cantidad(self, ml: f64) -> String {
        match self {
            Unidad::Mililitros if ml >= 1000.0 => format!("{:.1} L", ml / 1000.0),
            Unidad::Mililitros if ml.fract().abs() < 0.05 => format!("{ml:.0} ml"),
            Unidad::Mililitros => format!("{ml:.1} ml"),
            Unidad::Onzas => format!("{:.1} oz", self.convertir(ml)),
            Unidad::Cucharadas => format!("{:.0} cdas", self.convertir(ml)),
        }
    }

    /// Total de la lista, con litros cuando corresponde.
    fn total(self, ml: f64) -> String {
        if self == Unidad::Mililitros && ml >= 1000.0 {
            format!("{:.1} L", ml / 1000.0)
        } else {
            self.cantidad(ml)
        }
    }
}

/// Grupo de compra al que pertenece un ingrediente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Categoria {
    /// Leche, crema y derivados.
    Lacteos,
    /// Cafe, te y especias.
    Cafe,
    /// Licores y vinos.
    Alcohol,
    /// Azucar, jarabes y miel.
    Endulzantes,
    /// Hielo, agua y el resto.
    Otros,
}

impl Categoria {
    /// Titulo visible del grupo.
    fn etiqueta(self) -> &'static str {
        match self {
            Categoria::Cafe => "Café y especias",
            Categoria::Lacteos => "Lácteos",
            Categoria::Alcohol => "Alcohol",
            Categoria::Endulzantes => "Endulzantes",
            Categoria::Otros => "Otros",
        }
    }
}

/// Una linea agregada de la lista de compra.
#[derive(Debug, Clone, PartialEq)]
struct Compra {
    /// Nombre visible que se conserva del primer ingrediente encontrado.
    nombre: String,
    /// Grupo en el que se pinta.
    categoria: Categoria,
    /// Cantidad total en mililitros despues de multiplicar por comensales.
    ml: f64,
}

/// Pantalla C7: carga el catalogo, deja elegir recetas y arma la lista.
#[component]
pub fn PantallaListaCompra() -> Element {
    let mut recurso = use_resource(catalogo);

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_LISTA_COMPRA }

        main { class: "lista-compra",
            BarraSuperior {}

            header { class: "lista-compra-cabeza",
                p { class: "eyebrow", "Preparación" }
                h1 { "Lista de compra" }
                p { class: "lista-compra-sub", "Todo lo que necesitas para tus recetas, sumado en una sola lista." }
            }

            match &*recurso.read() {
                None => rsx! {
                    p { class: "estado", role: "status", "Cargando el recetario…" }
                },
                Some(Err(causa)) => rsx! {
                    div { class: "estado estado-error", role: "alert",
                        p { "No pudimos traer las recetas: {causa}" }
                        button {
                            class: "enlace",
                            r#type: "button",
                            onclick: move |_| recurso.restart(),
                            "Reintentar"
                        }
                    }
                },
                Some(Ok(lista)) => rsx! {
                    Contenido { lista: lista.clone() }
                },
            }
        }

        BarraInferior { activo: Destino::Biblioteca }
    }
}

/// Barra superior con la vuelta a la biblioteca.
#[component]
fn BarraSuperior() -> Element {
    rsx! {
        header { class: "lista-compra-barra",
            Link {
                to: Rutas::Biblioteca,
                class: "lista-compra-volver",
                aria_label: "Volver a la biblioteca",
                "‹ Biblioteca"
            }
            span { class: "lista-compra-marca", "Ratio" }
        }
    }
}

/// Cuerpo de C7: resumen, seleccion de recetas y lista agrupada.
#[component]
fn Contenido(lista: Vec<RecetaResumen>) -> Element {
    let mut seleccion = use_signal(Vec::<String>::new);
    let mut marcados = use_signal(Vec::<String>::new);
    let mut busqueda = use_signal(String::new);
    let mut comensales = use_signal(|| 1_u32);
    let mut unidad = use_signal(|| Unidad::Mililitros);

    let ids = seleccion.read().clone();
    let comidas = *comensales.read();
    let unidad_actual = *unidad.read();
    let compras = agregar(&lista, &ids, comidas);
    let total_ml: f64 = compras.iter().map(|compra| compra.ml).sum();
    let marcados_actuales = marcados.read().clone();
    let terminados = compras
        .iter()
        .filter(|compra| marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)))
        .count();
    let seleccionadas: Vec<RecetaResumen> = ids
        .iter()
        .filter_map(|id| lista.iter().find(|receta| &receta.id == id).cloned())
        .collect();
    let filtradas = filtrar_recetas(&lista, &busqueda.read(), &ids);

    rsx! {
        section { class: "compra-resumen", "aria-labelledby": "compra-resumen-titulo",
            div { class: "compra-resumen-cabeza",
                div {
                    p { id: "compra-resumen-titulo", class: "eyebrow", "Resumen" }
                    p { class: "compra-resumen-meta", "{seleccionadas.len()} recetas · {compras.len()} ingredientes" }
                }
                if !marcados_actuales.is_empty() {
                    button {
                        class: "enlace",
                        r#type: "button",
                        onclick: move |_| marcados.write().clear(),
                        "Desmarcar todo"
                    }
                }
            }

            dl { class: "compra-metricas",
                div { class: "compra-metrica",
                    dt { "Recetas" }
                    dd { "{seleccionadas.len()}" }
                }
                div { class: "compra-metrica",
                    dt { "Ingredientes" }
                    dd { "{compras.len()}" }
                }
                div { class: "compra-metrica",
                    dt { "Total" }
                    dd { "{unidad_actual.total(total_ml)}" }
                }
            }

            div { class: "compra-controles",
                div { class: "compra-comensales",
                    span { class: "compra-control-etiqueta", "Comensales" }
                    div { class: "compra-stepper",
                        button {
                            r#type: "button",
                            aria_label: "Quitar un comensal",
                            disabled: comidas <= 1,
                            onclick: move |_| comensales.set(comidas.saturating_sub(1).max(1)),
                            "−"
                        }
                        strong { "{comidas}" }
                        button {
                            r#type: "button",
                            aria_label: "Agregar un comensal",
                            disabled: comidas >= COMENSALES_MAXIMOS,
                            onclick: move |_| comensales.set((comidas + 1).min(COMENSALES_MAXIMOS)),
                            "+"
                        }
                    }
                }

                div { class: "compra-unidades", role: "group", "aria-label": "Unidad de la lista",
                    for opcion in Unidad::TODAS {
                        button {
                            key: "{opcion.etiqueta()}",
                            r#type: "button",
                            class: if opcion == unidad_actual { "compra-unidad activa" } else { "compra-unidad" },
                            aria_label: "{opcion.nombre()}",
                            onclick: move |_| unidad.set(opcion),
                            "{opcion.etiqueta()}"
                        }
                    }
                }
            }
        }

        section { class: "compra-recetas", "aria-labelledby": "compra-recetas-titulo",
            div { class: "compra-seccion-cabeza",
                h2 { id: "compra-recetas-titulo", "Recetas en la lista" }
                if !seleccionadas.is_empty() {
                    button {
                        class: "enlace",
                        r#type: "button",
                        onclick: move |_| {
                            seleccion.write().clear();
                            marcados.write().clear();
                        },
                        "Vaciar"
                    }
                }
            }

            if seleccionadas.is_empty() {
                p { class: "compra-nota", "Elige una o más recetas para empezar." }
            } else {
                div { class: "compra-chips",
                    for receta in seleccionadas.iter().cloned() {
                        button {
                            key: "{receta.id}",
                            class: "compra-chip",
                            r#type: "button",
                            aria_label: "Quitar {receta.receta.nombre}",
                            onclick: {
                                let id = receta.id.clone();
                                move |_| {
                                    seleccion.write().retain(|actual| actual != &id);
                                }
                            },
                            span { "{receta.receta.nombre}" }
                            span { class: "compra-chip-x", "aria-hidden": "true", "×" }
                        }
                    }
                }
            }

            details { class: "compra-picker",
                summary { "Agregar receta" }

                form {
                    class: "compra-buscador",
                    role: "search",
                    onsubmit: move |evento| evento.prevent_default(),
                    label { class: "solo-lector", r#for: "buscar-compra", "Buscar recetas para la lista" }
                    input {
                        id: "buscar-compra",
                        r#type: "search",
                        placeholder: "Buscar por nombre",
                        autocomplete: "off",
                        value: "{busqueda}",
                        oninput: move |evento| busqueda.set(evento.value()),
                    }
                }

                if filtradas.is_empty() {
                    p { class: "compra-nota", "No hay recetas que coincidan." }
                } else {
                    ul { class: "compra-picker-lista",
                        for receta in filtradas {
                            li {
                                button {
                                    key: "{receta.id}",
                                    class: if ids.iter().any(|id| id == &receta.id) { "compra-picker-opcion activa" } else { "compra-picker-opcion" },
                                    r#type: "button",
                                    aria_pressed: if ids.iter().any(|id| id == &receta.id) { "true" } else { "false" },
                                    onclick: {
                                        let id = receta.id.clone();
                                        move |_| {
                                            let mut actual = seleccion.write();
                                            if let Some(posicion) = actual.iter().position(|valor| valor == &id) {
                                                actual.remove(posicion);
                                            } else {
                                                actual.push(id.clone());
                                            }
                                        }
                                    },
                                    span { class: "compra-picker-nombre", "{receta.receta.nombre}" }
                                    span { class: "compra-picker-familia", "{familia_legible(&receta.familia)}" }
                                    span { class: "compra-picker-check", "aria-hidden": "true",
                                        if ids.iter().any(|id| id == &receta.id) { "✓" } else { "+" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        section { class: "compra-lista", "aria-labelledby": "compra-lista-titulo",
            div { class: "compra-seccion-cabeza",
                h2 { id: "compra-lista-titulo", "Tu lista" }
                if !compras.is_empty() {
                    span { class: "compra-cuenta", "{terminados} de {compras.len()} listos" }
                }
            }

            if seleccionadas.is_empty() {
                div { class: "estado compra-vacio",
                    p { "La lista aparecerá aquí en cuanto elijas una receta." }
                }
            } else if compras.is_empty() {
                p { class: "compra-nota", "Estas recetas no tienen ingredientes que comprar." }
            } else {
                for grupo in agrupar(&compras) {
                    section { class: "compra-grupo",
                        h3 { "{grupo.0.etiqueta()}" }
                        ul { class: "compra-items",
                            for compra in grupo.1 {
                                li {
                                    button {
                                        key: "{clave_compra(&compra.nombre)}",
                                        class: if marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)) { "compra-item marcado" } else { "compra-item" },
                                        r#type: "button",
                                        role: "checkbox",
                                        aria_checked: if marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)) { "true" } else { "false" },
                                        onclick: {
                                            let clave = clave_compra(&compra.nombre);
                                            move |_| {
                                                let mut actual = marcados.write();
                                                if let Some(posicion) = actual.iter().position(|valor| valor == &clave) {
                                                    actual.remove(posicion);
                                                } else {
                                                    actual.push(clave.clone());
                                                }
                                            }
                                        },
                                        span { class: "compra-check", "aria-hidden": "true",
                                            if marcados_actuales.iter().any(|clave| clave == &clave_compra(&compra.nombre)) { "✓" }
                                        }
                                        span { class: "compra-nombre", "{compra.nombre}" }
                                        span { class: "compra-cantidad", "{unidad_actual.cantidad(compra.ml)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Suma los ingredientes de las recetas elegidas y los agrupa por tipo.
fn agregar(lista: &[RecetaResumen], ids: &[String], comensales: u32) -> Vec<Compra> {
    let mut acumulado: HashMap<String, Compra> = HashMap::new();

    for receta in lista.iter().filter(|receta| ids.contains(&receta.id)) {
        for ingrediente in &receta.receta.ingredientes {
            let clave = clave_compra(&ingrediente.nombre);
            let entrada = acumulado.entry(clave).or_insert_with(|| Compra {
                nombre: ingrediente.nombre.clone(),
                categoria: categoria(&ingrediente.nombre),
                ml: 0.0,
            });
            entrada.ml += ingrediente.ml * f64::from(comensales);
        }
    }

    let mut compras: Vec<Compra> = acumulado.into_values().collect();
    compras.sort_by(|a, b| {
        a.categoria
            .cmp(&b.categoria)
            .then_with(|| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase()))
    });
    compras
}

/// Agrupa las lineas por categoria conservando el orden de la lista.
fn agrupar(compras: &[Compra]) -> Vec<(Categoria, Vec<Compra>)> {
    let mut grupos: Vec<(Categoria, Vec<Compra>)> = Vec::new();

    for compra in compras {
        match grupos.last_mut() {
            Some((categoria, items)) if *categoria == compra.categoria => items.push(compra.clone()),
            _ => grupos.push((compra.categoria, vec![compra.clone()])),
        }
    }

    grupos
}

/// Clave estable para sumar y marcar el mismo ingrediente entre recetas.
fn clave_compra(nombre: &str) -> String {
    nombre.trim().to_lowercase()
}

/// Clasifica un ingrediente en el grupo de compra que le corresponde.
fn categoria(nombre: &str) -> Categoria {
    let nombre = nombre.to_lowercase();

    if ["café", "cafe", "espresso", "expreso", "cold brew", "té", "te ", "matcha", "canela", "nuez moscada", "vainilla"]
        .iter()
        .any(|clave| nombre.contains(clave))
    {
        Categoria::Cafe
    } else if ["leche", "crema", "nata", "yogur", "mantequilla"].iter().any(|clave| nombre.contains(clave)) {
        Categoria::Lacteos
    } else if ["ron", "gin", "ginebra", "vodka", "whisky", "tequila", "mezcal", "vermut", "campari", "aperol", "licor", "brandy", "coñac", "vino", "cerveza"]
        .iter()
        .any(|clave| nombre.contains(clave))
    {
        Categoria::Alcohol
    } else if ["azúcar", "azucar", "jarabe", "miel", "sirope", "edulcorante"].iter().any(|clave| nombre.contains(clave)) {
        Categoria::Endulzantes
    } else {
        Categoria::Otros
    }
}

/// Filtra el catalogo del selector por nombre y conserva las recetas ya elegidas.
fn filtrar_recetas(lista: &[RecetaResumen], busqueda: &str, ids: &[String]) -> Vec<RecetaResumen> {
    let texto = busqueda.trim().to_lowercase();

    lista
        .iter()
        .filter(|receta| ids.contains(&receta.id) || texto.is_empty() || receta.receta.nombre.to_lowercase().contains(&texto))
        .cloned()
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn receta(id: &str, nombre: &str, ingredientes: &[(&str, f64)]) -> RecetaResumen {
        RecetaResumen {
            id: id.to_string(),
            tipo: "cafe".to_string(),
            familia: "con_leche".to_string(),
            autor: "prueba".to_string(),
            receta: proporciones::Receta {
                nombre: nombre.to_string(),
                vaso: proporciones::Vaso::nuevo(300.0, 40.0, 30.0, 100.0, 2.0),
                capacidad_ml: 300.0,
                ingredientes: ingredientes
                    .iter()
                    .map(|(nombre, ml)| proporciones::Componente::nuevo(*nombre, *ml, 0.0))
                    .collect(),
            },
            minutos: None,
            dificultad: None,
            intensidad: None,
        }
    }

    #[test]
    fn suma_el_mismo_ingrediente_entre_recetas() {
        let lista = vec![
            receta("a", "Uno", &[("Espresso", 30.0), ("Leche", 100.0)]),
            receta("b", "Dos", &[("  leche ", 50.0)]),
        ];
        let compras = agregar(&lista, &["a".to_string(), "b".to_string()], 2);
        let leche = compras.iter().find(|compra| compra.nombre == "Leche").unwrap();

        assert_eq!(leche.ml, 300.0);
        assert_eq!(compras.len(), 2);
    }

    #[test]
    fn clasifica_los_ingredientes_por_grupo() {
        assert_eq!(categoria("Espresso"), Categoria::Cafe);
        assert_eq!(categoria("Leche entera"), Categoria::Lacteos);
        assert_eq!(categoria("Ron blanco"), Categoria::Alcohol);
        assert_eq!(categoria("Jarabe de azúcar"), Categoria::Endulzantes);
        assert_eq!(categoria("Hielo"), Categoria::Otros);
    }

    #[test]
    fn convierte_las_cantidades_a_la_unidad_elegida() {
        assert_eq!(Unidad::Mililitros.cantidad(1500.0), "1.5 L");
        assert_eq!(Unidad::Onzas.cantidad(proporciones::OZ_ML), "1.0 oz");
        assert_eq!(Unidad::Cucharadas.cantidad(30.0), "2 cdas");
        assert_eq!(clave_compra("  Leche "), "leche");
    }
}
