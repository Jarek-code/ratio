//! Ratio · cliente móvil: acceso, prueba de conexión y pantalla Descubrir.

mod descubrir;
mod login;

use api::EstadoSalud;
use dioxus::prelude::*;

/// Backend desplegado en Render.
const SERVIDOR_POR_DEFECTO: &str = "https://ratio-689g.onrender.com";
/// Backend de las server functions; se cambia al compilar con `RATIO_SERVIDOR`.
const SERVIDOR: &str = match option_env!("RATIO_SERVIDOR") {
    Some(url) => url,
    None => SERVIDOR_POR_DEFECTO,
};

/// Prueba de salud completa: la app pregunta al backend (Render) y el backend a Supabase.
#[component]
fn PruebaSalud() -> Element {
    let mut resultado = use_signal(|| None::<Result<EstadoSalud, ServerFnError>>);
    let mut cargando = use_signal(|| false);

    rsx! {
        section { class: "salud",
            button {
                r#type: "button",
                disabled: *cargando.read(),
                onclick: move |_| {
                    cargando.set(true);
                    spawn(async move {
                        resultado.set(Some(api::salud().await));
                        cargando.set(false);
                    });
                },
                if *cargando.read() { "Comprobando…" } else { "Probar conexión" }
            }

            p { class: "servidor", "Servidor: {SERVIDOR}" }

            match &*resultado.read() {
                None => rsx! {},
                Some(Ok(estado)) if estado.base_de_datos => rsx! {
                    p { "Backend y base de datos: conectados" }
                },
                Some(Ok(estado)) => rsx! {
                    p { "Backend responde, base de datos no: {estado.detalle}" }
                },
                Some(Err(causa)) => rsx! {
                    p { "Sin respuesta del backend: {causa} (probado en {SERVIDOR})" }
                },
            }
        }
    }
}

/// Componente raíz: primero el acceso y, al entrar, la pantalla Descubrir.
#[component]
fn App() -> Element {
    let mut dentro = use_signal(|| false);

    rsx! {
        if *dentro.read() {
            descubrir::PantallaDescubrir {}
        } else {
            login::Login { on_entrar: move |_| dentro.set(true) }
            PruebaSalud {}
        }
    }
}

fn main() {
    dioxus::fullstack::set_server_url(SERVIDOR);
    dioxus::launch(App);
}
