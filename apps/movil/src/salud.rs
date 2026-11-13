//! Prueba de conexión con el backend y con la base de datos.

use api::EstadoSalud;
use dioxus::prelude::*;

use crate::SERVIDOR;
use crate::red;

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
                        // La prueba se repite sola si el servidor tarda en despertar
                        resultado.set(Some(red::con_reintentos(api::salud).await));
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
                    p { role: "alert", "Backend responde, base de datos no: {estado.detalle}" }
                },
                Some(Err(causa)) => rsx! {
                    p { role: "alert", "{red::mensaje(causa)} (probado en {SERVIDOR})" }
                    p { class: "nota", "El botón de arriba repite la prueba cuando quieras." }
                },
            }
        }
    }
}
