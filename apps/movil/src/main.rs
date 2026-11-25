//! Ratio · cliente móvil: rutas, backend y componente raíz.

mod actividad;
mod ajustar;
mod ajustes;
mod api_textos;
mod barra;
mod biblioteca;
mod comparar;
mod crear;
mod descubrir;
mod ficha;
mod foro;
mod fotos;
mod guardados;
mod historial;
mod i18n;
mod la_barra;
mod lista_compra;
mod local;
mod login;
mod marca;
mod mis_recetas;
mod moderacion;
mod perfil;
mod preparar;
mod push;
mod red;
mod reportar;
mod rutas;
mod salud;
#[cfg(target_os = "ios")]
mod selector_ios;
mod sesion;

use dioxus::prelude::*;
use dioxus::router::Router;

use crate::rutas::Rutas;

/// Backend desplegado en Render.
pub const SERVIDOR_POR_DEFECTO: &str = "https://ratio-689g.onrender.com";
/// Backend de las server functions; se cambia al compilar con `RATIO_SERVIDOR`.
pub const SERVIDOR: &str = match option_env!("RATIO_SERVIDOR") {
    Some(url) => url,
    None => SERVIDOR_POR_DEFECTO,
};

/// Componente raíz: el router decide la pantalla.
#[component]
fn App() -> Element {
    // La sesión guardada se recupera al abrir y se renueva mientras la app siga abierta
    let sesion = use_context_provider(|| Signal::new(sesion::cargar()));
    use_future(move || sesion::mantener(sesion));
    // Los cambios hechos sin conexión se envían cuando vuelve la red
    let pendientes = use_context_provider(|| Signal::new(0_usize));
    use_future(move || local::vaciar(sesion, pendientes));
    // Los datos de Storage para subir fotos se piden al servidor una vez
    use_future(fotos::cargar_configuracion);
    // Las novedades sin leer de La Barra se consultan cada minuto
    let no_leidas = use_context_provider(|| actividad::NoLeidas(Signal::new(0_u32)));
    use_future(move || actividad::vigilar(sesion, no_leidas));
    // El token push del aparato se registra cuando hay sesión (Android)
    use_future(move || push::registrar(sesion));
    // Unidad, tema y bienvenida, leídas del archivo de ajustes
    let ajustes = use_context_provider(|| Signal::new(ajustes::cargar()));
    let tema = ajustes.read().tema;
    // El idioma guardado manda desde el primer texto que se pinta
    i18n::fijar_actual(ajustes.read().idioma);

    rsx! {
        // Fuentes usadas por el diseño de Penpot.
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=Inter:wght@400;500;600;700&display=swap",
        }
        // El tema se aplica aquí: las hojas leen `data-theme` para cambiar sus colores
        div { class: "app", "data-theme": "{tema.atributo()}",
            Router::<Rutas> {}
        }
    }
}

fn main() {
    // Plazo de conexión y de respuesta de las peticiones al servidor
    red::configurar();
    dioxus::fullstack::set_server_url(SERVIDOR);
    dioxus::launch(App);
}
