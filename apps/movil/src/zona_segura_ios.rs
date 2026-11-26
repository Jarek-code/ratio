//! Márgenes seguros de iOS: barra de estado arriba e indicador de inicio abajo.
//!
//! Por defecto iOS pone márgenes automáticos al scroll del WebView y WebKit no cuadra con ellos
//! los elementos fijos: al llegar al final de una página larga la barra inferior sube y deja
//! un hueco, y al hacer scroll el contenido pasa por detrás del reloj. Además
//! `env(safe-area-inset-*)` vale 0. Por eso el WebView queda a pantalla completa, sin márgenes,
//! y la página los pone ella misma con lo que mide la ventana:
//! - arriba, `padding-top` en la raíz y una franja fija del color del fondo sobre el reloj,
//!   como la barra de estado opaca de Android;
//! - abajo, `--seguro-abajo` para la barra inferior (Android no lo define y usa `env()`).

// LIBRERÍAS

use std::sync::Once; // Los márgenes del WebView se fijan una sola vez

use objc2::MainThreadMarker; // UIKit solo se lee en el hilo principal
use objc2::msg_send; // `keyWindow` sin el envoltorio obsoleto
use objc2::rc::Retained; // Ventana devuelta por UIKit
use objc2::runtime::AnyClass; // Clase WKWebView, para reconocer el WebView
use objc2::runtime::NSObjectProtocol; // `isKindOfClass` de las vistas
use objc2_ui_kit::UIApplication; // Aplicación, para hallar la ventana
use objc2_ui_kit::UIEdgeInsets; // Margen del scroll del WebView
use objc2_ui_kit::UIScrollView; // Scroll del WebView
use objc2_ui_kit::UIScrollViewContentInsetAdjustmentBehavior; // Sin margen automático
use objc2_ui_kit::UIView; // Vistas donde se busca el WebView
use objc2_ui_kit::UIWindow; // Ventana que conoce el margen seguro

// CONSTANTES Y VARIABLES GLOBALES

/// Guion que da a `html` y `body` el color de fondo del tema (`--crema` de `.app`).
///
/// Lo que asoma al estirar la página más allá de su borde (rebote) es el fondo del documento,
/// que no ve el `data-theme` de `.app` y quedaría claro en el tema oscuro. Solo se cambia el color: los
/// degradados del `body` siguen igual. Se repite al segundo porque al abrir la app la hoja
/// con `--crema` puede no haber cargado todavía.
pub const PINTAR_FONDO: &str = "
    const pintar = () => {
        const app = document.querySelector('.app');
        if (!app) return;
        const color = getComputedStyle(app).getPropertyValue('--crema');
        document.documentElement.style.backgroundColor = color;
        document.body.style.backgroundColor = color;
    };
    pintar();
    setTimeout(pintar, 1000);
";

// FUNCIONES

/// Estilo de la raíz (márgenes) y estilo de la franja sobre el reloj, si ya hay ventana.
pub fn estilos() -> Option<(String, String)> {
    let mtm = MainThreadMarker::new()?;
    let aplicacion = UIApplication::sharedApplication(mtm);
    // SAFETY: keyWindow devuelve la ventana principal o nil
    let ventana: Option<Retained<UIWindow>> = unsafe { msg_send![&aplicacion, keyWindow] };

    let ventana = ventana?;
    let margen = ventana.safeAreaInsets();

    // WebView a pantalla completa: sin márgenes automáticos, que descuadran lo fijo
    static FIJAR: Once = Once::new();
    FIJAR.call_once(|| {
        if let Some(web) = buscar_webview(&ventana) {
            // SAFETY: `scrollView` de WKWebView devuelve siempre su UIScrollView
            let scroll: Retained<UIScrollView> = unsafe { msg_send![&web, scrollView] };
            scroll.setContentInsetAdjustmentBehavior(
                UIScrollViewContentInsetAdjustmentBehavior::Never,
            );
            scroll.setContentInset(UIEdgeInsets {
                top: 0.0,
                left: 0.0,
                bottom: 0.0,
                right: 0.0,
            });
        }
    });

    Some((
        format!(
            "--seguro-abajo: {}px; padding-top: {}px;",
            margen.bottom, margen.top
        ),
        format!(
            "position: fixed; top: 0; right: 0; left: 0; z-index: 50; height: {}px; \
             background: var(--crema); pointer-events: none;",
            margen.top
        ),
    ))
}

/// Busca el WebView (WKWebView) entre las vistas de la ventana.
fn buscar_webview(vista: &UIView) -> Option<Retained<UIView>> {
    let web_kit = AnyClass::get(c"WKWebView");
    for hija in vista.subviews().iter() {
        // wry usa una subclase propia de WKWebView
        if web_kit.is_some_and(|clase| hija.isKindOfClass(clase)) {
            return Some(hija);
        }
        if let Some(web) = buscar_webview(&hija) {
            return Some(web);
        }
    }

    None
}
