//! Pruebas del motor de proporciones (A1), parte compartida.
//!
//! Comprueba en una taza, un vaso alto y una copa de coctelería que la altura de cada capa
//! devuelve su volumen con error menor al 0,1 %, que las capas suman el total, que el
//! llenado se detiene en el 94 % y que una receta mayor que el vaso se marca recortada.

// LIBRERÍAS

use proporciones::Componente; // Ingrediente con su cantidad
use proporciones::Receta; // Receta lista para medir
use proporciones::Vaso; // Silueta del vaso
use proporciones::LLENADO_MAX; // Tope de llenado del vaso

// CONSTANTES Y VARIABLES GLOBALES

/// Error relativo máximo aceptado entre el volumen pedido y el que devuelve la altura.
const ERROR_MAX: f64 = 0.001;

/// Tolerancia para comparar sumas de números con decimales.
const TOLERANCIA: f64 = 1e-9;

// FUNCIONES

/// Devuelve una taza de cappuccino de 160 ml, más ancha arriba que abajo.
fn taza() -> Vaso
{
    Vaso::nuevo(160.0, 42.0, 30.0, 70.0, 3.0)
}

/// Devuelve un vaso alto de 350 ml, casi cilíndrico.
fn vaso_alto() -> Vaso
{
    Vaso::nuevo(350.0, 36.0, 30.0, 150.0, 3.0)
}

/// Devuelve una copa de coctelería de 180 ml, con fondo casi en punta.
fn copa() -> Vaso
{
    Vaso::nuevo(180.0, 60.0, 6.0, 60.0, 2.0)
}

/// Devuelve los tres vasos de prueba con su nombre, para los mensajes de error.
fn vasos() -> Vec<(&'static str, Vaso)>
{
    vec![("taza", taza()), ("vaso alto", vaso_alto()), ("copa", copa())]
}

/// Arma una receta de tres capas que ocupa `fraccion` de la capacidad del vaso.
///
/// Las capas se reparten 20 %, 50 % y 30 % del volumen, y la de en medio lleva alcohol.
fn receta_al(vaso: Vaso, fraccion: f64) -> Receta
{
    let total_val = vaso.capacidad_ml * fraccion; // Volumen total de la receta
    let ingredientes_vec = vec![
        Componente::nuevo("Base", total_val * 0.2, 0.0),
        Componente::nuevo("Licor", total_val * 0.5, 40.0),
        Componente::dilucion("Hielo", total_val * 0.3),
    ];

    Receta::nueva("Prueba", vaso, ingredientes_vec)
}

// PUNTO DE PARTIDA

/// Comprueba que cada capa devuelve su volumen con error menor al 0,1 % en los tres vasos.
#[test]
fn error_por_capa_menor_al_limite()
{
    // Recorrer los vasos con una receta que no llega al tope
    for (nombre, vaso) in vasos()
    {
        let detalle_res = proporciones::detalle(&receta_al(vaso, 0.8)); // Geometría resuelta

        // Revisar el error de cada capa
        for capa in detalle_res.capas.iter()
        {
            assert!(
                capa.error < ERROR_MAX,
                "{nombre}: layer {} error {} exceeds limit",
                capa.capa.nombre,
                capa.error
            );
        }
        assert!(detalle_res.error_max < ERROR_MAX, "{nombre}: max error exceeds limit");
    }
}

/// Comprueba que las capas suman el total, sus porcentajes suman 100 y se apilan sin huecos.
#[test]
fn capas_suman_el_total()
{
    // Recorrer los vasos
    for (nombre, vaso) in vasos()
    {
        let capas_res = proporciones::capas(&receta_al(vaso, 0.8)); // Reparto en capas
        let total_ml_acum: f64 = capas_res.capas.iter().map(|capa| capa.ml).sum(); // Suma de mililitros
        let total_pct_acum: f64 = capas_res.capas.iter().map(|capa| capa.porcentaje).sum(); // Suma de porcentajes

        assert!((total_ml_acum - capas_res.total_ml).abs() < TOLERANCIA, "{nombre}: layers do not add up to total");
        assert!((total_pct_acum - 100.0).abs() < TOLERANCIA, "{nombre}: percentages do not add up to 100");

        // Comprobar que cada capa empieza donde termina la anterior
        let mut desde_acum = 0.0; // Mililitros acumulados debajo de la capa actual
        for capa in capas_res.capas.iter()
        {
            assert!((capa.desde_ml - desde_acum).abs() < TOLERANCIA, "{nombre}: gap below {}", capa.nombre);
            desde_acum += capa.ml;
        }
    }
}

/// Comprueba que una receta por debajo del tope no recorta ninguna capa.
#[test]
fn bajo_el_tope_no_recorta()
{
    // Recorrer los vasos con una receta al 90 % de la capacidad
    for (nombre, vaso) in vasos()
    {
        let receta_val = receta_al(vaso, 0.9); // Receta por debajo del 94 %
        let capas_res = proporciones::capas(&receta_val); // Reparto en capas
        let detalle_res = proporciones::detalle(&receta_val); // Geometría resuelta

        assert!((capas_res.llenado - 0.9).abs() < TOLERANCIA, "{nombre}: fill should be 90 %");
        assert!(detalle_res.capas.iter().all(|capa| !capa.recortada), "{nombre}: no layer should be clipped");
    }
}

/// Comprueba que una receta del tamaño exacto del vaso se dibuja solo hasta el 94 %.
#[test]
fn llenado_se_detiene_en_el_tope()
{
    // Recorrer los vasos con una receta al 100 % de la capacidad
    for (nombre, vaso) in vasos()
    {
        let receta_val = receta_al(vaso, 1.0); // Receta que llena el vaso
        let capas_res = proporciones::capas(&receta_val); // Reparto en capas
        let detalle_res = proporciones::detalle(&receta_val); // Geometría resuelta
        let tope_val = proporciones::detalle(&receta_al(receta_val.vaso.clone(), LLENADO_MAX)).fraccion_llenado; // Altura del 94 %

        assert_eq!(capas_res.llenado, LLENADO_MAX, "{nombre}: fill must stop at the limit");

        // Revisar que ninguna capa se dibuja por encima del tope
        for capa in detalle_res.capas.iter()
        {
            assert!(
                capa.fraccion_dibujo_hasta <= tope_val + 1e-6,
                "{nombre}: layer {} drawn above the limit",
                capa.capa.nombre
            );
        }
        assert!(detalle_res.capas.last().is_some_and(|capa| capa.recortada), "{nombre}: top layer should be clipped");
    }
}

/// Comprueba que una receta mayor que el vaso marca como recortadas las capas que no caben.
#[test]
fn desbordamiento_marca_capas_recortadas()
{
    // Recorrer los vasos con una receta al 150 % de la capacidad
    for (nombre, vaso) in vasos()
    {
        let receta_val = receta_al(vaso, 1.5); // Receta que no cabe
        let capas_res = proporciones::capas(&receta_val); // Reparto en capas
        let detalle_res = proporciones::detalle(&receta_val); // Geometría resuelta

        assert!(capas_res.total_ml > capas_res.capacidad_ml, "{nombre}: recipe should overflow");
        assert_eq!(capas_res.llenado, LLENADO_MAX, "{nombre}: fill must stop at the limit");
        assert!(detalle_res.capas.first().is_some_and(|capa| !capa.recortada), "{nombre}: bottom layer fits");
        assert!(detalle_res.capas.last().is_some_and(|capa| capa.recortada), "{nombre}: top layer should be clipped");
    }
}
