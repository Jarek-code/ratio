//! Pantalla La Barra (D2): el muro de la comunidad y el hilo de una receta, parte de la app.
//!
//! El muro tiene cinco secciones: las recetas recientes, las de las cuentas que se siguen,
//! las populares, la conversación de toda la comunidad y la actividad propia. Las listas se
//! piden por páginas con «Cargar más». Cada autor lleva a su perfil y cada tarjeta a su ficha.

// LIBRERÍAS

use std::future::Future; // Petición de una página

use api::barra::ComentarioMuro; // Comentario de la conversación
use api::barra::Pagina; // Página de una lista
use api::barra::Pestana; // Pestaña del muro en el servidor
use api::barra::TarjetaBarra; // Receta publicada con sus contadores
use dioxus::prelude::*; // Componentes, señales y recursos

use crate::actividad::NoLeidas; // Novedades sin leer, para el punto de la pestaña
use crate::barra::BarraInferior; // Barra inferior compartida
use crate::barra::Destino; // Destino activo de la barra inferior
use crate::red::AvisoRed; // Aviso de fallo con su botón de reintentar
use crate::rutas::Rutas; // Rutas de la ficha, el hilo y los perfiles
use crate::sesion::SenalSesion; // Sesión con cuenta y token

// CONSTANTES Y VARIABLES GLOBALES

/// Hoja del sistema visual que comparten las pantallas.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Color de una capa que no trae el suyo.
const COLOR_RESERVA: &str = "#B9A48C";

// ESTRUCTURAS

/// Sección visible de La Barra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seccion
{
    /// Recetas publicadas, de la más nueva a la más vieja.
    Recientes,
    /// Recetas de las cuentas que sigo.
    Siguiendo,
    /// Recetas con más actividad en los últimos 30 días.
    Populares,
    /// Últimos comentarios de toda la comunidad.
    Conversacion,
    /// Novedades sobre mis recetas, mis comentarios y mis reportes.
    Actividad,
}

impl Seccion
{
    /// Las secciones en el orden en que se pintan.
    const TODAS: [Seccion; 5] = [
        Seccion::Recientes,
        Seccion::Siguiendo,
        Seccion::Populares,
        Seccion::Conversacion,
        Seccion::Actividad,
    ];

    /// Clave de la sección en la dirección de la pantalla.
    fn clave(self) -> &'static str
    {
        match self
        {
            Seccion::Recientes => "recientes",
            Seccion::Siguiendo => "siguiendo",
            Seccion::Populares => "populares",
            Seccion::Conversacion => "conversacion",
            Seccion::Actividad => "actividad",
        }
    }

    /// Nombre visible de la sección.
    fn etiqueta(self) -> &'static str
    {
        match self
        {
            Seccion::Recientes => "Recientes",
            Seccion::Siguiendo => "Siguiendo",
            Seccion::Populares => "Populares",
            Seccion::Conversacion => "Conversación",
            Seccion::Actividad => "Actividad",
        }
    }

    /// Sección de una clave; una clave vacía o desconocida abre Recientes.
    fn de_clave(clave: &str) -> Seccion
    {
        Seccion::TODAS.into_iter().find(|seccion| seccion.clave() == clave).unwrap_or(Seccion::Recientes)
    }
}

/// Momento de una lista paginada.
#[derive(Clone, PartialEq)]
enum Estado
{
    /// Hay una página en camino.
    Cargando,
    /// La última página llegó; `hay_mas` dice si queda otra.
    Listo
    {
        /// Si queda otra página por pedir.
        hay_mas: bool,
    },
    /// La última página falló.
    Fallo(ServerFnError),
}

/// Elemento de una lista paginada: se reconoce por su identificador.
trait ConId
{
    /// Identificador único del elemento.
    fn id(&self) -> &str;
}

impl ConId for TarjetaBarra
{
    fn id(&self) -> &str
    {
        &self.receta.id
    }
}

impl ConId for ComentarioMuro
{
    fn id(&self) -> &str
    {
        &self.id
    }
}

/// Lista que se va llenando página a página.
struct Lista<T: 'static>
{
    /// Elementos ya recibidos, en orden.
    elementos: Signal<Vec<T>>,
    /// Momento de la última petición.
    estado: Signal<Estado>,
    /// Pide la página siguiente.
    cargar: Callback<()>,
}

// FUNCIONES

/// Lleva una lista paginada: pide la primera página al montar y las siguientes con `cargar`.
///
/// `pedir` recibe el número de página (0 = primera). Un elemento que ya estaba en la lista no
/// se repite, aunque la paginación se desplace porque alguien publicó mientras tanto.
fn usar_paginas<T, F, Fut>(pedir: F) -> Lista<T>
where
    T: ConId + Clone + 'static,
    F: Fn(u32) -> Fut + Copy + 'static,
    Fut: Future<Output = Result<Pagina<T>, ServerFnError>> + 'static,
{
    let mut elementos = use_signal(Vec::<T>::new); // Elementos recibidos
    let mut estado = use_signal(|| Estado::Cargando); // Momento de la lista
    let mut pagina = use_signal(|| 0_u32); // Siguiente página por pedir

    let cargar = use_callback(move |_: ()|
    {
        estado.set(Estado::Cargando);
        spawn(async move
        {
            let numero = *pagina.peek();
            match pedir(numero).await
            {
                Ok(nueva) =>
                {
                    // Agregar solo lo que todavía no estaba
                    let mut lista = elementos.write();
                    for elemento in nueva.elementos
                    {
                        if !lista.iter().any(|otro| otro.id() == elemento.id())
                        {
                            lista.push(elemento);
                        }
                    }
                    drop(lista);
                    pagina.set(numero + 1);
                    estado.set(Estado::Listo { hay_mas: nueva.hay_mas });
                }
                Err(causa) => estado.set(Estado::Fallo(causa)),
            }
        });
    });

    // Pedir la primera página una sola vez
    use_hook(|| cargar.call(()));

    Lista { elementos, estado, cargar }
}

/// Nombre visible del tipo de bebida.
fn tipo_legible(tipo: &str) -> &'static str
{
    match tipo
    {
        "cocteleria" => "Coctelería",
        _ => "Café",
    }
}

/// Texto de un contador con su singular: «1 comentario», «3 comentarios».
fn contar(cuantos: i64, singular: &str, plural: &str) -> String
{
    if cuantos == 1 { format!("1 {singular}") } else { format!("{cuantos} {plural}") }
}

/// Pie de una lista paginada: carga en curso, fallo con reintento o botón «Cargar más».
#[component]
fn PieLista(estado: Estado, vacia: bool, cargando: String, al_cargar: EventHandler<()>) -> Element
{
    rsx!
    {
        match estado
        {
            Estado::Cargando => rsx!
            {
                p { class: "estado", role: "status", if vacia { "{cargando}" } else { "Cargando más…" } }
            },
            Estado::Fallo(causa) => rsx!
            {
                // Fallo de red o del servidor: explicar qué pasó y ofrecer reintentar
                AvisoRed
                {
                    causa,
                    reintentar: move |_| al_cargar.call(()),
                }
            },
            Estado::Listo { hay_mas: true } => rsx!
            {
                button { class: "chip muro-mas", r#type: "button", onclick: move |_| al_cargar.call(()), "Cargar más" }
            },
            Estado::Listo { hay_mas: false } => rsx! {},
        }
    }
}

/// Tarjeta de una receta publicada: autor, vaso, proporción y contadores.
///
/// La usan el muro y los perfiles. El autor lleva a su perfil, el cuerpo a la ficha, los
/// comentarios al hilo y las versiones al historial.
#[component]
pub fn TarjetaMuro(tarjeta: TarjetaBarra) -> Element
{
    let resumen = &tarjeta.receta;
    let svg_val = proporciones::svg(&resumen.receta, &proporciones::Opciones { escalas: false, mini: true }); // Vaso en miniatura
    let capas_res = proporciones::capas(&resumen.receta); // Capas y total servido
    let graduacion = proporciones::graduacion(&resumen.receta); // Alcohol de la bebida
    let descripcion = capas_res
        .capas
        .iter()
        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
        .collect::<Vec<_>>()
        .join(", "); // Texto accesible del vaso
    let foto_url = tarjeta.foto.as_deref().and_then(crate::fotos::url_publica); // Primera foto, si se puede mostrar
    let inicial = resumen.autor.chars().next().map(|letra| letra.to_uppercase().to_string()).unwrap_or_default();

    let mut datos_vec = vec![
        crate::biblioteca::familia_legible(&resumen.familia),
        format!("{:.0} ml", capas_res.total_ml),
    ]; // Datos breves de la bebida
    if graduacion.abv > 0.0
    {
        datos_vec.push(format!("{:.1} % ABV", graduacion.abv));
    }

    let valoracion = match tarjeta.porcentaje_me_salio()
    {
        Some(porcentaje) => format!("{porcentaje} % me salió ({})", tarjeta.valoraciones),
        None => "Sin valoraciones".to_string(),
    }; // Reproducibilidad de la receta

    rsx!
    {
        li
        {
            class: "muro-tarjeta",

            div
            {
                class: "muro-cabeza",
                span { class: "muro-avatar", "aria-hidden": "true", "{inicial}" }
                div
                {
                    class: "muro-autor",
                    Link
                    {
                        to: Rutas::PerfilPublico { alias: resumen.autor.clone() },
                        class: "barra-autor",
                        "aria-label": "Ver el perfil de {resumen.autor}",
                        "{resumen.autor}"
                    }
                    span { class: "barra-meta", "{api::barra::hace(tarjeta.hace_s)} · {tipo_legible(&resumen.tipo)}" }
                }
                span
                {
                    class: if tarjeta.derivada { "muro-insignia muro-insignia--version" } else { "muro-insignia" },
                    if tarjeta.derivada { "Versión" } else { "Original" }
                }
            }

            Link
            {
                to: Rutas::Ficha { id: resumen.id.clone() },
                class: "muro-cuerpo",

                if let Some(url) = foto_url
                {
                    img { class: "muro-foto", src: "{url}", alt: "Foto de {resumen.receta.nombre}", loading: "lazy" }
                }
                div
                {
                    class: "muro-resumen",
                    div
                    {
                        class: "perfil-mini",
                        role: "img",
                        "aria-label": "{resumen.receta.nombre}: {descripcion}",
                        dangerous_inner_html: svg_val,
                    }
                    div
                    {
                        h2 { "{resumen.receta.nombre}" }
                        if let Some(origen) = tarjeta.origen.clone()
                        {
                            p { class: "barra-meta", "Versión de {origen}" }
                        }
                        p { class: "barra-meta", "{datos_vec.join(\" · \")}" }
                    }
                }
                div
                {
                    class: "muro-proporcion",
                    "aria-hidden": "true",
                    for (indice, capa) in capas_res.capas.iter().enumerate()
                    {
                        span
                        {
                            key: "{indice}",
                            style: "flex-grow:{capa.ml};background:{capa.color.clone().unwrap_or_else(|| COLOR_RESERVA.to_string())}",
                        }
                    }
                }
            }

            div
            {
                class: "muro-contadores",
                span { class: "muro-dato", "{valoracion}" }
                Link
                {
                    to: Rutas::Hilo { id: resumen.id.clone() },
                    class: "muro-dato muro-dato--enlace",
                    "{contar(tarjeta.comentarios, \"comentario\", \"comentarios\")}"
                }
                if tarjeta.versiones > 0
                {
                    Link
                    {
                        to: Rutas::Historial { id: resumen.id.clone() },
                        class: "muro-dato muro-dato--enlace",
                        "{contar(tarjeta.versiones, \"versión\", \"versiones\")}"
                    }
                }
            }
        }
    }
}

/// Lista paginada de recetas de una pestaña del muro.
#[component]
fn Muro(pestana: Pestana) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para la regla de edad y para «Siguiendo»
    let lista = usar_paginas(move |pagina|
    {
        let token = crate::sesion::token(&sesion);
        async move { crate::red::con_reintentos(move || api::barra::muro(token.clone(), pestana, pagina)).await }
    });
    let tarjetas_vec = (lista.elementos)();
    let estado = (lista.estado)();
    let cargar = lista.cargar;

    // «Siguiendo» necesita saber quién pregunta
    if pestana == Pestana::Siguiendo && sesion.read().is_none()
    {
        return rsx!
        {
            div
            {
                class: "estado",
                p { "Entra para ver lo que publican las cuentas que sigues." }
                Link { to: Rutas::Entrar, class: "enlace", "Entrar" }
            }
        };
    }

    rsx!
    {
        // Lista vacía y ya cargada: explicar por qué
        if tarjetas_vec.is_empty() && estado == (Estado::Listo { hay_mas: false })
        {
            p
            {
                class: "estado",
                match pestana
                {
                    Pestana::Siguiendo => "Sigue a alguien desde su perfil para ver aquí lo que publica.",
                    Pestana::Populares => "Todavía no hay recetas con actividad reciente.",
                    Pestana::Recientes => "Todavía no hay recetas publicadas. Publica la tuya desde Mi recetario.",
                }
            }
        }

        ul
        {
            class: "muro",
            for tarjeta in tarjetas_vec.iter()
            {
                TarjetaMuro { key: "{tarjeta.receta.id}", tarjeta: tarjeta.clone() }
            }
        }

        PieLista
        {
            estado,
            vacia: tarjetas_vec.is_empty(),
            cargando: "Cargando La Barra…",
            al_cargar: move |_| cargar.call(()),
        }
    }
}

/// Lista paginada con los últimos comentarios de toda la comunidad.
#[component]
fn Conversacion() -> Element
{
    let sesion = use_context::<SenalSesion>(); // Token para la regla de edad
    let lista = usar_paginas(move |pagina|
    {
        let token = crate::sesion::token(&sesion);
        async move { crate::red::con_reintentos(move || api::barra::conversacion(token.clone(), pagina)).await }
    });
    let comentarios_vec = (lista.elementos)();
    let estado = (lista.estado)();
    let cargar = lista.cargar;

    rsx!
    {
        // Lista vacía y ya cargada: invitar a abrir la conversación
        if comentarios_vec.is_empty() && estado == (Estado::Listo { hay_mas: false })
        {
            p { class: "estado", "Nadie ha comentado todavía. Abre una receta y cuenta cómo te salió." }
        }

        ul
        {
            class: "barra-hilo",
            for comentario in comentarios_vec.iter()
            {
                li
                {
                    key: "{comentario.id}",
                    class: "barra-comentario",

                    p
                    {
                        class: "barra-meta",
                        Link
                        {
                            to: Rutas::PerfilPublico { alias: comentario.autor.clone() },
                            class: "barra-autor",
                            "aria-label": "Ver el perfil de {comentario.autor}",
                            "{comentario.autor}"
                        }
                        if comentario.es_respuesta { " respondió · " } else { " comentó · " }
                        "{api::barra::hace(comentario.hace_s)}"
                    }
                    Link
                    {
                        to: Rutas::Hilo { id: comentario.receta_id.clone() },
                        class: "barra-evento",
                        "aria-label": "Abrir el hilo de {comentario.receta_nombre}",

                        p { strong { "{comentario.receta_nombre}" } }
                        if let Some(ancla) = comentario.ancla.clone()
                        {
                            p { class: "barra-ancla-texto", "{ancla}" }
                        }
                        p { "{comentario.texto}" }
                    }
                }
            }
        }

        PieLista
        {
            estado,
            vacia: comentarios_vec.is_empty(),
            cargando: "Cargando la conversación…",
            al_cargar: move |_| cargar.call(()),
        }
    }
}

/// Pantalla D2: el muro de La Barra con sus cinco secciones.
///
/// `pestana` es la clave de la sección abierta; vacía abre Recientes.
#[component]
pub fn PantallaLaBarra(pestana: String) -> Element
{
    let no_leidas = use_context::<NoLeidas>().0; // Novedades sin leer
    let activa = Seccion::de_clave(&pestana);

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                p { class: "eyebrow", "Comunidad" }
                h1 { "La Barra" }
            }

            nav
            {
                class: "chips",
                "aria-label": "Secciones de La Barra",

                for seccion in Seccion::TODAS
                {
                    Link
                    {
                        key: "{seccion.clave()}",
                        to: Rutas::LaBarra { pestana: seccion.clave().to_string() },
                        class: if seccion == activa { "chip chip--activo" } else { "chip" },
                        "aria-current": if seccion == activa { "page" } else { "false" },

                        "{seccion.etiqueta()}"
                        if seccion == Seccion::Actividad && no_leidas() > 0
                        {
                            span { class: "destino-punto", role: "img", "aria-label": "{no_leidas()} sin leer" }
                        }
                    }
                }
            }

            crate::local::AvisoPendientes {}

            // Cada sección lleva su clave para empezar con una lista limpia al cambiar
            match activa
            {
                Seccion::Recientes => rsx! { Muro { key: "{activa.clave()}", pestana: Pestana::Recientes } },
                Seccion::Siguiendo => rsx! { Muro { key: "{activa.clave()}", pestana: Pestana::Siguiendo } },
                Seccion::Populares => rsx! { Muro { key: "{activa.clave()}", pestana: Pestana::Populares } },
                Seccion::Conversacion => rsx! { Conversacion {} },
                Seccion::Actividad => rsx! { crate::actividad::ContenidoActividad {} },
            }
        }

        BarraInferior { activo: Destino::LaBarra }
    }
}

/// Pantalla del hilo de una receta (D2 y D3): su vaso y toda la conversación.
#[component]
pub fn PantallaHilo(id: String) -> Element
{
    let sesion = use_context::<SenalSesion>(); // Cuenta y token
    let id_recurso = id.clone();
    let mut recurso = use_resource(use_reactive!(|(id_recurso,)|
    {
        let token = crate::sesion::token(&sesion);
        let cuenta = crate::sesion::cuenta(&sesion);
        crate::red::con_reintentos(move || crate::local::receta(id_recurso.clone(), token.clone(), cuenta.clone()))
    })); // Receta del hilo

    rsx!
    {
        document::Stylesheet { href: CSS_BASE }

        main
        {
            class: "descubrir",

            header
            {
                class: "seccion-cabeza",
                Link { to: Rutas::la_barra(), class: "enlace", aria_label: "Volver a La Barra", "‹ La Barra" }
                Link { to: Rutas::Ficha { id: id.clone() }, class: "enlace", "Ver la receta" }
            }

            // Elegir contenido según el estado de la petición
            match &*recurso.read()
            {
                None => rsx!
                {
                    p { class: "estado", role: "status", "Cargando el hilo…" }
                },
                Some(Err(causa)) => rsx!
                {
                    // Fallo de red o del servidor: explicar qué pasó y ofrecer reintentar
                    AvisoRed
                    {
                        causa: causa.clone(),
                        reintentar: move |_| recurso.restart(),
                    }
                },
                Some(Ok(None)) => rsx!
                {
                    p { class: "estado", "Esta receta no existe o ya no está disponible." }
                },
                Some(Ok(Some(receta))) =>
                {
                    let svg_val = proporciones::svg(&receta.receta, &proporciones::Opciones { escalas: false, mini: true }); // Vaso en miniatura
                    let descripcion = proporciones::capas(&receta.receta)
                        .capas
                        .iter()
                        .map(|capa| format!("{} {:.0} %", capa.nombre.to_lowercase(), capa.porcentaje))
                        .collect::<Vec<_>>()
                        .join(", "); // Texto accesible del vaso

                    rsx!
                    {
                        p { class: "eyebrow", "Hilo" }
                        div
                        {
                            class: "perfil-receta",
                            div
                            {
                                class: "perfil-mini",
                                role: "img",
                                "aria-label": "{receta.receta.nombre}: {descripcion}",
                                dangerous_inner_html: svg_val,
                            }
                            div
                            {
                                h1 { "{receta.receta.nombre}" }
                                p
                                {
                                    class: "barra-meta",
                                    "por "
                                    Link
                                    {
                                        to: Rutas::PerfilPublico { alias: receta.autor.clone() },
                                        class: "barra-autor",
                                        "{receta.autor}"
                                    }
                                }
                            }
                        }

                        crate::foro::LaBarra { receta: receta.clone() }
                    }
                },
            }
        }

        BarraInferior { activo: Destino::LaBarra }
    }
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas
{
    // LIBRERÍAS

    use super::contar; // Contador con singular y plural
    use super::Seccion; // Secciones de La Barra

    // FUNCIONES

    /// Comprueba que cada sección vuelve de su clave y que lo desconocido abre Recientes.
    #[test]
    fn seccion_desde_la_direccion()
    {
        for seccion in Seccion::TODAS
        {
            assert_eq!(Seccion::de_clave(seccion.clave()), seccion);
        }
        assert_eq!(Seccion::de_clave(""), Seccion::Recientes);
        assert_eq!(Seccion::de_clave("otra"), Seccion::Recientes);
    }

    /// Comprueba el singular y el plural de los contadores.
    #[test]
    fn contador_con_singular()
    {
        assert_eq!(contar(0, "comentario", "comentarios"), "0 comentarios");
        assert_eq!(contar(1, "comentario", "comentarios"), "1 comentario");
        assert_eq!(contar(4, "versión", "versiones"), "4 versiones");
    }
}
