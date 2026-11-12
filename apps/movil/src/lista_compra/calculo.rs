//! Cálculo de la lista de compra (C7): suma de ingredientes, categorías y filtros.
//!
//! No depende de la interfaz: recibe el catálogo y la selección y devuelve las líneas.

use std::collections::HashMap;

use api::biblioteca::RecetaResumen;

/// Grupo de compra al que pertenece un ingrediente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Categoria {
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
    pub(super) fn etiqueta(self) -> &'static str {
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
pub(super) struct Compra {
    /// Nombre visible que se conserva del primer ingrediente encontrado.
    pub(super) nombre: String,
    /// Grupo en el que se pinta.
    pub(super) categoria: Categoria,
    /// Cantidad total en mililitros despues de multiplicar por comensales.
    pub(super) ml: f64,
}

/// Suma los ingredientes de las recetas elegidas y los agrupa por tipo.
pub(super) fn agregar(lista: &[RecetaResumen], ids: &[String], comensales: u32) -> Vec<Compra> {
    let mut acumulado: HashMap<String, Compra> = HashMap::new();

    for receta in lista.iter().filter(|receta| ids.contains(&receta.id)) {
        // C7 solo maneja ml: los componentes mg siguen fuera de la lista hasta que
        // la pantalla soporte cantidades con unidad propia.
        for componente in receta
            .componentes
            .iter()
            .filter(|componente| componente.unidad == "ml")
        {
            let clave = clave_compra(&componente.nombre);
            let entrada = acumulado.entry(clave).or_insert_with(|| Compra {
                nombre: componente.nombre.clone(),
                categoria: categoria(&componente.nombre),
                ml: 0.0,
            });
            entrada.ml += componente.cantidad * f64::from(comensales);
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
pub(super) fn agrupar(compras: &[Compra]) -> Vec<(Categoria, Vec<Compra>)> {
    let mut grupos: Vec<(Categoria, Vec<Compra>)> = Vec::new();

    for compra in compras {
        match grupos.last_mut() {
            Some((categoria, items)) if *categoria == compra.categoria => {
                items.push(compra.clone())
            }
            _ => grupos.push((compra.categoria, vec![compra.clone()])),
        }
    }

    grupos
}

/// Clave estable para sumar y marcar el mismo ingrediente entre recetas.
pub(super) fn clave_compra(nombre: &str) -> String {
    nombre.trim().to_lowercase()
}

/// Clasifica un ingrediente en el grupo de compra que le corresponde.
fn categoria(nombre: &str) -> Categoria {
    let nombre = nombre.to_lowercase();

    if [
        "café",
        "cafe",
        "espresso",
        "expreso",
        "cold brew",
        "té",
        "te ",
        "matcha",
        "canela",
        "nuez moscada",
        "vainilla",
    ]
    .iter()
    .any(|clave| nombre.contains(clave))
    {
        Categoria::Cafe
    } else if ["leche", "crema", "nata", "yogur", "mantequilla"]
        .iter()
        .any(|clave| nombre.contains(clave))
    {
        Categoria::Lacteos
    } else if [
        "ron", "gin", "ginebra", "vodka", "whisky", "tequila", "mezcal", "vermut", "campari",
        "aperol", "licor", "brandy", "coñac", "vino", "cerveza",
    ]
    .iter()
    .any(|clave| nombre.contains(clave))
    {
        Categoria::Alcohol
    } else if [
        "azúcar",
        "azucar",
        "jarabe",
        "miel",
        "sirope",
        "edulcorante",
    ]
    .iter()
    .any(|clave| nombre.contains(clave))
    {
        Categoria::Endulzantes
    } else {
        Categoria::Otros
    }
}

/// Filtra el catalogo del selector por nombre y conserva las recetas ya elegidas.
pub(super) fn filtrar_recetas(lista: &[RecetaResumen], busqueda: &str, ids: &[String]) -> Vec<RecetaResumen> {
    let texto = busqueda.trim().to_lowercase();

    lista
        .iter()
        .filter(|receta| {
            ids.contains(&receta.id)
                || texto.is_empty()
                || receta.receta.nombre.to_lowercase().contains(&texto)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::lista_compra::Unidad;

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
            componentes: ingredientes
                .iter()
                .enumerate()
                .map(
                    |(posicion, (nombre, cantidad))| api::biblioteca::ComponenteFicha {
                        ingrediente_id: format!("{id}-{posicion}"),
                        posicion: posicion as i16,
                        nombre: nombre.to_string(),
                        unidad: "ml".to_string(),
                        cantidad: *cantidad,
                        graduacion_pct: 0.0,
                        color: None,
                        dilucion: false,
                    },
                )
                .collect(),
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
        let leche = compras
            .iter()
            .find(|compra| compra.nombre == "Leche")
            .unwrap();

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
