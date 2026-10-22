//! Ratio · cliente móvil: rutas, backend y componente raíz.

mod descubrir;
mod ficha;
mod login;
mod proximamente;
mod rutas;
mod salud;

use dioxus::prelude::*;

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
    rsx! {
        Router::<Rutas> {}
    }
}

fn main() {
    dioxus::fullstack::set_server_url(SERVIDOR);
    dioxus::launch(App);
}
