//! Prueba de conexión con el backend y con la base de datos.

use api::EstadoSalud;
use dioxus::prelude::*;

use crate::SERVIDOR;

/// Pregunta al backend por su estado: la app va a Render y Render a Supabase.
#[component]
pub fn PruebaSalud() -> Element {
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
