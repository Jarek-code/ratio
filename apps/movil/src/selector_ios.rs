//! Selector de fotos nativo de iOS (D1), parte de la app.
//!
//! Dioxus 0.7 intercepta `<input type="file">` y en iOS no abre nada, así que la app presenta
//! directamente `UIImagePickerController` de UIKit sobre la pantalla visible y recibe la foto
//! elegida de la galería como JPEG. La galería no pide permiso en iOS con este selector.

// LIBRERÍAS

use std::cell::RefCell; // Canal guardado dentro del delegado

use objc2::define_class; // Clase de Objective-C escrita en Rust
use objc2::msg_send; // Mensajes de Objective-C sin envoltorio
use objc2::rc::Retained; // Referencias con conteo de Objective-C
use objc2::runtime::AnyObject; // Objeto genérico del diccionario de la foto
use objc2::runtime::NSObject; // Superclase del delegado
use objc2::runtime::NSObjectProtocol; // Protocolo base del delegado
use objc2::DefinedClass; // Acceso a las variables del delegado
use objc2::MainThreadMarker; // Prueba de que se está en el hilo principal
use objc2::MainThreadOnly; // Creación de objetos de UIKit en el hilo principal
use objc2_foundation::NSDictionary; // Información de la foto elegida
use objc2_ui_kit::UIApplication; // Aplicación, para hallar la ventana visible
use objc2_ui_kit::UIImage; // Foto elegida
use objc2_ui_kit::UIImagePickerController; // Selector de UIKit
use objc2_ui_kit::UIImagePickerControllerDelegate; // Avisos del selector
use objc2_ui_kit::UIImagePickerControllerInfoKey; // Claves del diccionario de la foto
use objc2_ui_kit::UIImagePickerControllerOriginalImage; // Clave de la foto original
use objc2_ui_kit::UIImagePickerControllerSourceType; // Origen: la galería
use objc2_ui_kit::UINavigationControllerDelegate; // Protocolo que también exige el selector
use objc2_ui_kit::UIViewController; // Pantalla sobre la que se presenta el selector
use objc2_ui_kit::UIWindow; // Ventana visible de la app
use tokio::sync::oneshot; // Entrega de la foto a la tarea que la pidió

// CONSTANTES Y VARIABLES GLOBALES

/// Calidad del JPEG: 0,7 deja una foto de cámara por debajo de los 5 MB del bucket.
const CALIDAD_JPEG: f64 = 0.7;

/// Origen «galería» del selector (valor 0 de `UIImagePickerControllerSourceType`).
///
/// Apple recomienda PHPicker, pero su librería no está en Cargo.lock; el valor numérico evita
/// la constante marcada como obsoleta y UIKit lo sigue aceptando.
const GALERIA: UIImagePickerControllerSourceType = UIImagePickerControllerSourceType(0);

// ESTRUCTURAS

/// Variables del delegado: el canal por el que se entrega la foto, o `None` si se canceló.
struct Estado
{
    /// Extremo de envío; se toma una sola vez.
    enviar: RefCell<Option<oneshot::Sender<Option<Vec<u8>>>>>,
}

define_class!(
    // SAFETY: NSObject no tiene requisitos para heredarla y el delegado no implementa Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RatioDelegadoFotos"]
    #[ivars = Estado]
    /// Delegado que recibe la foto elegida o la cancelación del selector.
    struct Delegado;

    unsafe impl NSObjectProtocol for Delegado {}

    unsafe impl UINavigationControllerDelegate for Delegado {}

    unsafe impl UIImagePickerControllerDelegate for Delegado
    {
        /// Convierte la foto elegida a JPEG y la entrega.
        #[unsafe(method(imagePickerController:didFinishPickingMediaWithInfo:))]
        unsafe fn elegida(
            &self,
            selector: &UIImagePickerController,
            info: &NSDictionary<UIImagePickerControllerInfoKey, AnyObject>,
        )
        {
            // SAFETY: la clave es una constante de UIKit, presente mientras viva la app
            let clave = unsafe { UIImagePickerControllerOriginalImage };
            let bytes = info
                .objectForKey(clave)
                .and_then(|objeto| objeto.downcast::<UIImage>().ok())
                .and_then(|imagen| imagen.jpeg_representation(CALIDAD_JPEG))
                .map(|datos| datos.to_vec()); // JPEG de la foto original
            self.entregar(selector, bytes);
        }

        /// Entrega `None` cuando la persona cierra el selector sin elegir.
        #[unsafe(method(imagePickerControllerDidCancel:))]
        fn cancelada(&self, selector: &UIImagePickerController)
        {
            self.entregar(selector, None);
        }
    }
);

// FUNCIONES

impl Delegado
{
    /// Crea el delegado con el canal por el que entregará la foto.
    fn nuevo(mtm: MainThreadMarker, enviar: oneshot::Sender<Option<Vec<u8>>>) -> Retained<Self>
    {
        let delegado = Self::alloc(mtm).set_ivars(Estado
        {
            enviar: RefCell::new(Some(enviar)),
        });
        // SAFETY: init de NSObject no recibe argumentos y devuelve el mismo objeto
        unsafe { msg_send![super(delegado), init] }
    }

    /// Cierra el selector y entrega el resultado una sola vez.
    fn entregar(&self, selector: &UIImagePickerController, bytes: Option<Vec<u8>>)
    {
        selector.dismissViewControllerAnimated_completion(true, None);

        if let Some(enviar) = self.ivars().enviar.borrow_mut().take()
        {
            let _ = enviar.send(bytes); // La tarea pudo terminar antes: no hay a quién avisar
        }
    }
}

/// Pantalla visible sobre la que presentar el selector: la raíz de la ventana principal o lo
/// que esté presentado encima.
fn pantalla_visible(mtm: MainThreadMarker) -> Option<Retained<UIViewController>>
{
    let aplicacion = UIApplication::sharedApplication(mtm);
    // SAFETY: keyWindow devuelve la ventana principal o nil; se lee sin el envoltorio obsoleto
    let ventana: Option<Retained<UIWindow>> = unsafe { msg_send![&aplicacion, keyWindow] };
    let mut actual = ventana?.rootViewController()?;

    // Subir hasta la pantalla que está encima de todas
    while let Some(encima) = actual.presentedViewController()
    {
        actual = encima;
    }

    Some(actual) // Pantalla visible
}

/// Abre la galería de iOS y devuelve la foto elegida como JPEG, o `None` si se canceló.
///
/// Devuelve un mensaje en español listo para mostrar si no se puede abrir la galería.
pub async fn elegir_de_galeria() -> Result<Option<Vec<u8>>, String>
{
    let Some(mtm) = MainThreadMarker::new() else
    {
        return Err("No se pudo abrir la galería en este momento.".to_string()); // UIKit exige el hilo principal
    };
    if !UIImagePickerController::isSourceTypeAvailable(GALERIA, mtm)
    {
        return Err("Este dispositivo no tiene galería de fotos.".to_string());
    }
    let Some(pantalla) = pantalla_visible(mtm) else
    {
        return Err("No se pudo abrir la galería en este momento.".to_string()); // Sin ventana visible
    };

    let (enviar, recibir) = oneshot::channel();
    let delegado = Delegado::nuevo(mtm, enviar); // Vive hasta que llega la foto
    let selector = UIImagePickerController::new(mtm);
    let delegado_objeto: &AnyObject = &delegado;

    // SAFETY: el delegado implementa los dos protocolos que exige el selector y sigue vivo
    // mientras se espera la respuesta, porque esta función lo conserva hasta el final
    unsafe
    {
        selector.setSourceType(GALERIA);
        selector.setDelegate(Some(delegado_objeto));
    }
    pantalla.presentViewController_animated_completion(&selector, true, None);

    let foto = recibir.await.ok().flatten(); // Foto elegida o cancelación
    drop(delegado);

    Ok(foto)
}
