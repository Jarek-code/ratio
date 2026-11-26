//! Comparar (B4) e Ingredientes y equivalencias (B5), parte de la app.
//!
//! Comparar pone dos recetas frente a frente. El vaso de cada una lo dibuja el motor de
//! proporciones ([`proporciones::svg`]): la altura de cada capa sale del volumen real del
//! vaso, no de un porcentaje inventado. La tabla de diferencias compara lo que el motor
//! calcula de verdad —volumen servido, graduación, capas y mililitros de cada ingrediente—,
//! así que dos recetas con los mismos datos nunca muestran una diferencia falsa.
//!
//! Equivalencias muestra cada capa de una receta en mililitros, onzas, partes y cucharadas.
//! Las dos pantallas leen la receta con A2, así que funcionan sin conexión si ya se vio.

// LIBRERÍAS

use api::receta::RecetaCompleta; // Receta con vaso y capas
use dioxus::prelude::*; // Componentes, señales y recursos
use proporciones::Equivalencia; // Cantidad de una capa en varias unidades

use crate::ajustes::SenalAjustes; // Unidad elegida en Ajustes
use crate::ajustes::Unidad; // Mililitros, onzas o partes
use crate::i18n::Texto;
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Rutas para volver a la ficha y elegir otra receta
use crate::sesion::SenalSesion; // Cuenta y token para leer las recetas // Textos traducibles

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja de estilos propia de la comparación.
const CSS_COMPARAR: Asset = asset!("/assets/comparar.css");

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Por debajo de esta diferencia la fila se muestra como «igual»: es ruido de redondeo.
const MARGEN_IGUAL: f64 = 0.05;

// ESTRUCTURAS

/// Medidas de una receta que se pueden poner al lado de las de otra.
struct Medidas {
    /// Volumen servido, en mililitros.
    total_ml: f64,
    /// Alcohol por volumen, en %.
    abv: f64,
    /// Cantidad de capas que dibuja el vaso.
    capas: usize,
    /// Cada ingrediente con su cantidad en todas las unidades.
    ingredientes: Vec<Equivalencia>,
}

impl Medidas {
    /// Saca las medidas del motor: las mismas cuentas que dibujan el vaso.
    fn de(receta: &proporciones::Receta) -> Self {
        let capas_res = proporciones::capas(receta);

        Self {
            total_ml: capas_res.total_ml,
            abv: capas_res.abv,
            capas: capas_res.capas.len(),
            ingredientes: proporciones::equivalencias(receta),
        }
    }

    /// Mililitros del ingrediente con ese nombre, si la receta lo lleva.
    fn ml_de(&self, nombre: &str) -> Option<f64> {
        self.ingredientes
            .iter()
            .find(|equivalencia| equivalencia.nombre == nombre)
            .map(|equivalencia| equivalencia.ml)
    }

    /// Nombres de los ingredientes en el orden en que aparecen en el vaso.
    fn nombres(&self) -> Vec<&str> {
        self.ingredientes
            .iter()
            .map(|equivalencia| equivalencia.nombre.as_str())
            .collect()
    }
}

/// Diferencia de una fila, ya decidida: a favor de A, en contra o ninguna.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Delta {
    /// A queda por encima de B.
    Mayor(f64),
    /// A queda por debajo de B.
    Menor(f64),
    /// Las dos recetas coinciden.
    Igual,
}

impl Delta {
    /// Decide la diferencia entre dos cantidades.
    fn de(a: f64, b: f64) -> Self {
        let diferencia = a - b;

        if diferencia.abs() < MARGEN_IGUAL {
            Delta::Igual
        } else if diferencia > 0.0 {
            Delta::Mayor(diferencia)
        } else {
            Delta::Menor(diferencia)
        }
    }

    /// Clase del color: verde si A gana, cobre si pierde, gris si empatan.
    fn clase(&self) -> &'static str {
        match self {
            Delta::Mayor(_) => "cmp-delta cmp-delta--mayor",
            Delta::Menor(_) => "cmp-delta cmp-delta--menor",
            Delta::Igual => "cmp-delta cmp-delta--igual",
        }
    }

    /// Texto de la diferencia, con su flecha y sus decimales.
    fn texto(&self, decimales: usize) -> String {
        match self {
            Delta::Mayor(diferencia) => format!("↑ +{diferencia:.decimales$}"),
            Delta::Menor(diferencia) => format!("↓ {diferencia:.decimales$}"),
            Delta::Igual => crate::i18n::ahora(Texto::Igual).to_string(),
        }
    }
}

/// Una fila de la tabla: qué se compara y qué tiene cada receta.
struct Fila {
    /// Nombre de la medida o del ingrediente.
    nombre: String,
    /// Valor en la receta A, ya escrito.
    a: String,
    /// Valor en la receta B, ya escrito.
    b: String,
    /// Diferencia entre las dos.
    delta: Delta,
}

// FUNCIONES

/// Texto accesible del vaso: nombre y porcentaje de cada capa.
fn descripcion(receta: &proporciones::Receta) -> String {
    let partes_vec: Vec<String> = proporciones::capas(receta)
        .capas
        .iter()
        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
        .collect(); // Una parte por capa

    format!("{}: {}", receta.nombre, partes_vec.join(", "))
}

/// Arma las filas de la tabla de diferencias con las medidas de las dos recetas.
///
/// Primero las tres medidas del motor (volumen, graduación y capas) y después un renglón por
/// ingrediente, en el orden en que aparecen en el vaso de A y luego los que solo tiene B.
fn filas_de(a: &Medidas, b: &Medidas, unidad: Unidad) -> Vec<Fila> {
    let mut filas_vec = vec![
        Fila {
            nombre: crate::i18n::ahora(Texto::Volumen).to_string(),
            a: crate::i18n::con(
                crate::i18n::actual(),
                Texto::MililitrosCorto,
                &[&format!("{:.0}", a.total_ml)],
            ),
            b: crate::i18n::con(
                crate::i18n::actual(),
                Texto::MililitrosCorto,
                &[&format!("{:.0}", b.total_ml)],
            ),
            delta: Delta::de(a.total_ml, b.total_ml),
        },
        Fila {
            nombre: crate::i18n::ahora(Texto::Graduacion).to_string(),
            a: if a.abv > 0.0 {
                format!("{:.1} %", a.abv)
            } else {
                "sin alcohol".to_string()
            },
            b: if b.abv > 0.0 {
                format!("{:.1} %", b.abv)
            } else {
                "sin alcohol".to_string()
            },
            delta: Delta::de(a.abv, b.abv),
        },
        Fila {
            nombre: { crate::i18n::ahora(Texto::Capas) }.to_string(),
            a: format!("{}", a.capas),
            b: format!("{}", b.capas),
            delta: Delta::de(a.capas as f64, b.capas as f64),
        },
    ];

    // Ingredientes de las dos recetas, sin repetir y en orden de aparición
    let mut nombres_vec = a.nombres();
    for nombre in b.nombres() {
        if !nombres_vec.contains(&nombre) {
            nombres_vec.push(nombre);
        }
    }

    for nombre in nombres_vec {
        let valor_a = a
            .ingredientes
            .iter()
            .find(|equivalencia| equivalencia.nombre == nombre)
            .map(|equivalencia| unidad.cantidad(equivalencia))
            .unwrap_or_else(|| "—".to_string()); // La receta A no lo lleva
        let valor_b = b
            .ingredientes
            .iter()
            .find(|equivalencia| equivalencia.nombre == nombre)
            .map(|equivalencia| unidad.cantidad(equivalencia))
            .unwrap_or_else(|| "—".to_string()); // La receta B no lo lleva

        filas_vec.push(Fila {
            nombre: nombre.to_string(),
            a: valor_a,
            b: valor_b,
            delta: Delta::de(
                a.ml_de(nombre).unwrap_or(0.0),
                b.ml_de(nombre).unwrap_or(0.0),
            ), // La diferencia se mide en ml, la unidad del motor
        });
    }

    filas_vec
}

/// Pantalla B4: compara la receta `id` con la receta `con`; sin `con`, deja elegirla.
#[component]
pub fn PantallaComparar(id: String, con: String) -> Element {
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let ajustes = use_context::<SenalAjustes>(); // Unidad de las cantidades
    let navegador = navigator();

    // Receta de la ficha, con la copia local cuando no hay red
    let id_a = id.clone();
    let mut recurso_a = use_resource(use_reactive!(|(id_a,)| {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        crate::red::con_reintentos(move || {
            crate::local::receta(id_a.clone(), token.clone(), cuenta.clone())
        })
    }));

    // Receta elegida para comparar; sin elegir, no hay nada que pedir
    let con_b = con.clone();
    let mut recurso_b = use_resource(use_reactive!(|(con_b,)| {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        async move {
            // Sin segunda receta todavía: no hay nada que pedir al servidor
            if con_b.trim().is_empty() {
                return Ok(None);
            }

            crate::red::con_reintentos(move || {
                crate::local::receta(con_b.clone(), token.clone(), cuenta.clone())
            })
            .await
        }
    }));

    // Catálogo de los dos selectores
    let mut catalogo_res = use_resource(move || {
        let token = crate::sesion::token(&sesion);
        crate::red::con_reintentos(move || api::biblioteca::catalogo(token.clone()))
    });

    let unidad = ajustes().unidad;
    let catalogo = catalogo_res
        .read()
        .as_ref()
        .and_then(|res| res.as_ref().ok())
        .cloned(); // Recetas para elegir
    let con_para_a = con.clone(); // Segunda receta tal como está, para el selector de A

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_COMPARAR }

        main
        {
            class: "cmp",

            header
            {
                class: "cmp-cabeza",
                crate::rutas::Volver { etiqueta: crate::i18n::ahora(Texto::VolverALaFicha).to_string(), respaldo: Rutas::Ficha { id: id.clone() } }
            }

            section
            {
                class: "cmp-tarjeta",

                h1 { "Comparar" }
                p { class: "cmp-subtitulo", "{crate::i18n::ahora(Texto::CompararSubtitulo)}" }

                // Sin catálogo no se pueden cambiar las recetas: se avisa y se ofrece reintentar
                if let Some(Err(causa)) = &*catalogo_res.read()
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| catalogo_res.restart(),
                    }
                }

                div
                {
                    class: "cmp-selectores",

                    SelectorReceta
                    {
                        etiqueta: "Receta A",
                        actual: id.clone(),
                        catalogo: catalogo.clone(),
                        al_elegir: move |elegida: String|
                        {
                            navegador.push(Rutas::Comparar { id: elegida, con: con_para_a.clone() });
                        },
                    }

                    SelectorReceta
                    {
                        etiqueta: "Receta B",
                        actual: con.clone(),
                        catalogo: catalogo.clone(),
                        al_elegir: move |elegida: String|
                        {
                            navegador.push(Rutas::Comparar { id: id.clone(), con: elegida });
                        },
                    }
                }
            }

            // Vasos y tabla: solo cuando están las dos recetas
            match (&*recurso_a.read(), &*recurso_b.read())
            {
                (None, _) => rsx! { p { class: "cmp-estado", role: "status", "{crate::i18n::ahora(Texto::CargandoRecetaA)}" } },
                (Some(Err(causa)), _) => rsx!
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso_a.restart(),
                    }
                },
                (Some(Ok(None)), _) => rsx! { p { class: "cmp-estado", "{crate::i18n::ahora(Texto::RecetaNoDisponible)}" } },
                (_, None) => rsx! { p { class: "cmp-estado", role: "status", "{crate::i18n::ahora(Texto::CargandoRecetaB)}" } },
                (_, Some(Err(causa))) => rsx!
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso_b.restart(),
                    }
                },
                (_, Some(Ok(None))) => rsx!
                {
                    section
                    {
                        class: "cmp-tarjeta cmp-vacia",
                        p { class: "cmp-subtitulo", "{crate::i18n::ahora(Texto::EligeRecetaB)}" }
                    }
                },
                (Some(Ok(Some(a))), Some(Ok(Some(b)))) => rsx!
                {
                    Comparacion { a: a.clone(), b: b.clone(), unidad }
                },
            }
        }
    }
}

/// Selector de una de las dos recetas, con el catálogo del servidor.
#[component]
fn SelectorReceta(
    /// Texto de la etiqueta: «Receta A» o «Receta B».
    etiqueta: String,
    /// Identificador elegido ahora mismo (vacío = sin elegir).
    actual: String,
    /// Recetas del catálogo, si ya llegaron.
    catalogo: Option<Vec<api::biblioteca::RecetaResumen>>,
    /// Qué hacer al elegir otra receta.
    al_elegir: EventHandler<String>,
) -> Element {
    let etiqueta_id = etiqueta.to_lowercase().replace(' ', "-");
    let opciones_vec: Vec<(String, String)> = catalogo
        .map(|lista| {
            lista
                .into_iter()
                .map(|receta| (receta.id, receta.receta.nombre))
                .collect()
        })
        .unwrap_or_default(); // Vacío mientras el catálogo no llega
    let sin_catalogo = opciones_vec.is_empty();

    rsx! {
        div
        {
            class: "cmp-selector",

            label { class: "cmp-selector-etiqueta", r#for: "{etiqueta_id}", "{etiqueta}" }

            select
            {
                id: "{etiqueta_id}",
                disabled: sin_catalogo,
                value: actual.clone(),
                onchange: move |evento| al_elegir.call(evento.value()),

                option { value: "", selected: actual.is_empty(), "{crate::i18n::ahora(Texto::EligeUnaReceta)}" }
                for (id_opcion, nombre) in opciones_vec
                {
                    option
                    {
                        key: "{id_opcion}",
                        value: "{id_opcion}",
                        selected: id_opcion == actual,
                        "{nombre}"
                    }
                }
            }

            if sin_catalogo
            {
                p { class: "cmp-selector-aviso", "{crate::i18n::ahora(Texto::SinCatalogoComparar)}" }
            }
        }
    }
}

/// Las dos tazas y la tabla de diferencias.
#[component]
fn Comparacion(
    /// Receta de la ficha.
    a: RecetaCompleta,
    /// Receta elegida para comparar.
    b: RecetaCompleta,
    /// Unidad en la que se escriben las cantidades.
    unidad: Unidad,
) -> Element {
    let filas_vec = filas_de(&Medidas::de(&a.receta), &Medidas::de(&b.receta), unidad);
    // Nombre de la unidad en el idioma elegido, para la frase con dato
    let unidad_nombre = crate::i18n::t(
        crate::i18n::actual(),
        match unidad {
            Unidad::Mililitros => Texto::Mililitros,
            Unidad::Onzas => Texto::Onzas,
            Unidad::Partes => Texto::PartesUnidad,
        },
    );

    rsx! {
        div
        {
            class: "cmp-tazas",

            Taza { receta: a.clone(), activa: true }
            Taza { receta: b.clone(), activa: false }
        }

        section
        {
            class: "cmp-tarjeta cmp-diferencias",

            h2 { "{crate::i18n::ahora(Texto::DiferenciasAB)}" }
            p {
                class: "cmp-subtitulo",
                {crate::i18n::con(crate::i18n::actual(), Texto::MedidasDelMotor, &[unidad_nombre])}
            }

            div
            {
                class: "cmp-tabla",

                div
                {
                    class: "cmp-fila cmp-fila-cabecera",
                    span { class: "cmp-nutriente", "{crate::i18n::ahora(Texto::Medida)}" }
                    span { class: "cmp-valor", "A" }
                    span { class: "cmp-valor", "B" }
                    span { class: "cmp-delta", "Δ" }
                }

                for fila in filas_vec
                {
                    div
                    {
                        key: "{fila.nombre}",
                        class: "cmp-fila",
                        span { class: "cmp-nutriente", "{fila.nombre}" }
                        span { class: "cmp-valor", "{fila.a}" }
                        span { class: "cmp-valor", "{fila.b}" }
                        span { class: fila.delta.clase(), "{fila.delta.texto(1)}" }
                    }
                }
            }
        }

        div
        {
            class: "cmp-pie",
            Link { to: Rutas::Equivalencias { id: a.id.clone() }, class: "chip", "{crate::i18n::ahora(Texto::IngredientesOtrasUnidades)}" }
            crate::rutas::VolverChip { etiqueta: crate::i18n::ahora(Texto::VolverALaReceta).to_string(), respaldo: Rutas::Ficha { id: a.id.clone() } }
        }
    }
}

/// Una taza de la comparación: el vaso que dibuja el motor, su nombre y sus datos.
#[component]
fn Taza(
    /// Receta que se dibuja.
    receta: RecetaCompleta,
    /// Si es la receta A (la de la ficha) se resalta.
    activa: bool,
) -> Element {
    let capas_res = proporciones::capas(&receta.receta);
    let svg_val = proporciones::svg(
        &receta.receta,
        &proporciones::Opciones {
            escalas: false,
            mini: false,
        },
    ); // Vaso real del motor

    rsx! {
        article
        {
            class: if activa { "cmp-taza cmp-taza--activa" } else { "cmp-taza" },

            h3 { class: "cmp-taza-nombre", "{receta.receta.nombre}" }

            // El vaso lo dibuja el motor; el SVG ya viene con su propio texto accesible
            div
            {
                class: "cmp-vaso",
                dangerous_inner_html: svg_val,
            }
            p { class: "solo-lector", "{descripcion(&receta.receta)}" }

            p
            {
                class: "cmp-taza-dato",
                strong { "{capas_res.total_ml:.0} ml" }
                " · {receta.vaso_nombre}"
            }

            p
            {
                class: "cmp-taza-dato",
                if capas_res.abv > 0.0 { "{capas_res.abv:.1} % ABV" } else { "{crate::i18n::ahora(Texto::SinAlcoholCorto)}" }
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use proporciones::Componente;
    use proporciones::Receta;
    use proporciones::Vaso;

    /// Receta de prueba: un vaso alto con dos ingredientes.
    fn receta(nombre: &str, cafe: f64, leche: f64) -> Receta {
        Receta::nueva(
            nombre,
            Vaso::nuevo(300.0, 40.0, 32.0, 120.0, 2.0),
            vec![
                Componente::nuevo("Café", cafe, 0.0),
                Componente::nuevo("Leche", leche, 0.0),
            ],
        )
    }

    /// Las tres primeras filas son las medidas del motor y luego va cada ingrediente.
    #[test]
    fn filas_llevan_las_medidas_del_motor() {
        let a = Medidas::de(&receta("A", 120.0, 30.0));
        let b = Medidas::de(&receta("B", 60.0, 30.0));
        let filas_vec = filas_de(&a, &b, Unidad::Mililitros);

        assert_eq!(filas_vec.len(), 5); // Volumen, graduación, capas y los dos ingredientes
        assert_eq!(filas_vec[0].nombre, "Volumen"); // Español, el idioma por defecto
        assert_eq!(filas_vec[0].a, "150 ml");
        assert_eq!(filas_vec[0].b, "90 ml");
        assert_eq!(filas_vec[0].delta, Delta::Mayor(60.0));
        assert_eq!(filas_vec[1].nombre, "Graduación");
        assert_eq!(filas_vec[2].nombre, { "Capas" });
        assert_eq!(filas_vec[3].nombre, "Café");
        assert_eq!(filas_vec[3].delta, Delta::Mayor(60.0));
        assert_eq!(filas_vec[4].nombre, "Leche");
        assert_eq!(filas_vec[4].delta, Delta::Igual); // La leche es la misma en las dos
    }

    /// Un ingrediente que solo tiene una de las dos recetas se compara contra cero.
    #[test]
    fn ingredientes_que_faltan_cuentan_como_cero() {
        let solo_cafe = Receta::nueva(
            "Solo café",
            Vaso::nuevo(300.0, 40.0, 32.0, 120.0, 2.0),
            vec![Componente::nuevo("Café", 100.0, 0.0)],
        );
        let a = Medidas::de(&solo_cafe);
        let b = Medidas::de(&receta("Con leche", 100.0, 50.0));
        let filas_vec = filas_de(&a, &b, Unidad::Mililitros);

        let leche = filas_vec
            .iter()
            .find(|fila| fila.nombre == "Leche")
            .expect("la fila de la leche");

        assert_eq!(leche.a, "—"); // La receta A no lleva leche
        assert_eq!(leche.delta, Delta::Menor(-50.0));
    }

    /// Dos recetas iguales no muestran ninguna diferencia.
    #[test]
    fn recetas_iguales_empatan() {
        let a = Medidas::de(&receta("A", 100.0, 50.0));
        let b = Medidas::de(&receta("B", 100.0, 50.0));

        assert!(
            filas_de(&a, &b, Unidad::Mililitros)
                .iter()
                .all(|fila| fila.delta == Delta::Igual)
        );
    }
}

/// Pantalla B5: cada capa de la receta en mililitros, onzas, partes y cucharadas.
#[component]
pub fn PantallaEquivalencias(id: String) -> Element {
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let ajustes = use_context::<SenalAjustes>(); // Unidad destacada
    let id_recurso = id.clone();
    let mut recurso = use_resource(use_reactive!(|(id_recurso,)| {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        crate::red::con_reintentos(move || {
            crate::local::receta(id_recurso.clone(), token.clone(), cuenta.clone())
        })
    })); // Receta de la ficha
    let unidad = ajustes().unidad;
    // Nombre de la unidad en el idioma elegido, para las frases con dato
    let unidad_nombre = crate::i18n::t(
        crate::i18n::actual(),
        match unidad {
            crate::ajustes::Unidad::Mililitros => Texto::Mililitros,
            crate::ajustes::Unidad::Onzas => Texto::Onzas,
            crate::ajustes::Unidad::Partes => Texto::PartesUnidad,
        },
    );

    rsx! {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                crate::rutas::Volver { etiqueta: "Volver a la ficha".to_string(), respaldo: Rutas::Ficha { id: id.clone() } }
            }
            p { class: "eyebrow", "Recetario" }
            h1 { "{crate::i18n::ahora(Texto::IngredientesYEquivalencias)}" }

            match &*recurso.read()
            {
                None => rsx! { p { class: "estado", role: "status", "{crate::i18n::ahora(Texto::CargandoReceta)}" } },
                Some(Err(causa)) => rsx!
                {
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso.restart(),
                    }
                },
                Some(Ok(None)) => rsx! { p { class: "estado", "{crate::i18n::ahora(Texto::RecetaNoDisponible)}" } },
                Some(Ok(Some(receta))) =>
                {
                    let equivalencias_vec = proporciones::equivalencias(&receta.receta);

                    rsx!
                    {
                        h2 { "{receta.receta.nombre}" }
                        p {
                            class: "nota",
                            {crate::i18n::con(crate::i18n::actual(), Texto::NotaEquivalencias, &[unidad_nombre])}
                        }

                        table
                        {
                            class: "comparar-tabla",
                            caption { class: "solo-lector", {"CadaIngredienteEnUnidades"} }
                            thead
                            {
                                tr
                                {
                                    th { scope: "col", "{crate::i18n::ahora(Texto::Ingrediente)}" }
                                    th { scope: "col", "ml" }
                                    th { scope: "col", "oz" }
                                    th { scope: "col", "{crate::i18n::ahora(Texto::PartesUnidad)}" }
                                    th { scope: "col", "{crate::i18n::ahora(Texto::Cucharadas)}" }
                                }
                            }
                            tbody
                            {
                                for eq in equivalencias_vec.iter()
                                {
                                    tr
                                    {
                                        key: "{eq.nombre}",
                                        th { scope: "row", "{eq.nombre}" if eq.dilucion { " (dilución)" } }
                                        td { "{eq.ml:.0}" }
                                        td { "{eq.oz:.1}" }
                                        td { "{eq.partes:.1}" }
                                        td { "{eq.cucharadas:.1}" }
                                    }
                                }
                            }
                        }

                        Link { to: Rutas::Ajustes, class: "chip", "{crate::i18n::ahora(Texto::CambiarUnidadPreferida)}" }
                    }
                }
            }
        }
    }
}
