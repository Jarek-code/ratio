//! Retroceso del sistema: la barra de navegación de Android vuelve una pantalla en vez de
//! cerrar la app.
//!
//! La app guarda su historial en memoria, no en el historial del WebView, así que el botón
//! del sistema no puede apoyarse en él. En su lugar, Android avisa a la página con un evento
//! del documento y aquí se le pide al navegador de la app que vuelva. La página también
//! publica si hay algo a dónde volver, para que Android sepa cuándo cerrar de verdad.

use dioxus::prelude::*;

use crate::rutas::Rutas;

/// Promesa que no termina: mantiene abierto el canal mientras la app viva.
const ESCUCHA: &str = r#"
new Promise((listo) => {
    document.addEventListener('ratio-atras', () => dioxus.send('atras'));
});
"#;

/// Atiende el botón de retroceso de Android y publica si hay pantalla anterior.
#[component]
pub fn AtrasDelSistema() -> Element {
    let navegador = navigator();
    let ruta = use_route::<Rutas>(); // Cambia de valor en cada pantalla

    // Android lee esta marca para saber si cerrar la app o volver una pantalla
    use_effect(use_reactive!(|(ruta,)| {
        let _ = ruta; // Solo marca el cambio de pantalla
        let puede = navegador.can_go_back();
        spawn(async move {
            let mut aviso = document::eval(&format!("window.__ratioPuedeVolver = {puede};"));
            let _ = aviso.recv::<()>().await;
        });
    }));

    // Cada aviso de Android vuelve una pantalla, con el mismo respaldo que el botón «›»
    use_future(move || {
        let navegador = navigator();
        async move {
            let mut canal = document::eval(ESCUCHA);
            while canal.recv::<String>().await.is_ok() {
                if navegador.can_go_back() {
                    navegador.go_back();
                }
            }
        }
    });

    rsx! {}
}
