//! Push nativo de iOS: permiso, token de APNs y toque en el aviso.
//!
//! Hace lo mismo que `MainActivity.kt` en Android: deja el token en `push-token.txt` y, al
//! tocar un aviso, la ruta en `abrir-ruta.txt`; `push.rs` lee los dos archivos igual en ambas
//! plataformas. El delegado de la app es de tao (`AppDelegate`) y no recibe el token de APNs,
//! así que se le añaden en tiempo de ejecución los dos métodos que iOS llama al registrarse.

// LIBRERÍAS

use std::ffi::CStr; // Nombres y firmas de los métodos añadidos

use block2::DynBlock; // Bloques de cierre que manda iOS
use block2::RcBlock; // Bloque propio para la respuesta del permiso
use objc2::MainThreadMarker; // Prueba de que se está en el hilo principal
use objc2::MainThreadOnly; // Creación del delegado en el hilo principal
use objc2::define_class; // Clase de Objective-C escrita en Rust
use objc2::ffi; // `class_addMethod` sobre el delegado de tao
use objc2::msg_send; // Mensajes de Objective-C sin envoltorio
use objc2::rc::Retained; // Referencias con conteo de Objective-C
use objc2::runtime::AnyClass; // Clase del delegado de la app
use objc2::runtime::AnyObject; // Delegado de la app, sin tipo concreto
use objc2::runtime::Bool; // Respuesta del permiso
use objc2::runtime::Imp; // Puntero a función de un método añadido
use objc2::runtime::NSObject; // Superclase del delegado de avisos
use objc2::runtime::NSObjectProtocol; // Protocolo base del delegado de avisos
use objc2::runtime::ProtocolObject; // Delegado visto como su protocolo
use objc2::runtime::Sel; // Selector de los métodos añadidos
use objc2::sel; // Selectores escritos en Rust
use objc2_foundation::NSData; // Token de APNs
use objc2_foundation::NSError; // Fallo del registro o del permiso
use objc2_ui_kit::UIApplication; // Aplicación, para registrarse en APNs
use objc2_user_notifications::UNAuthorizationOptions; // Qué permiso se pide
use objc2_user_notifications::UNNotification; // Aviso que llega con la app abierta
use objc2_user_notifications::UNNotificationPresentationOptions; // Cómo mostrarlo abierta
use objc2_user_notifications::UNNotificationResponse; // Aviso tocado
use objc2_user_notifications::UNUserNotificationCenter; // Centro de avisos de iOS
use objc2_user_notifications::UNUserNotificationCenterDelegate; // Avisos del centro

// CONSTANTES Y VARIABLES GLOBALES

/// Ruta que se deja al tocar un aviso; la misma que escribe `MainActivity.kt`.
const RUTA_ACTIVIDAD: &str = "/actividad";

/// Firma de los dos métodos añadidos: `void (id self, SEL _cmd, id app, id dato)`.
const FIRMA: &CStr = c"v@:@@";

// ESTRUCTURAS

define_class!(
    // SAFETY: NSObject no tiene requisitos para heredarla y el delegado no implementa Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RatioDelegadoAvisos"]
    /// Delegado del centro de avisos: los muestra con la app abierta y anota los tocados.
    struct DelegadoAvisos;

    unsafe impl NSObjectProtocol for DelegadoAvisos {}

    unsafe impl UNUserNotificationCenterDelegate for DelegadoAvisos {
        /// Con la app abierta iOS no muestra el aviso salvo que se le pida, como en Android.
        #[unsafe(method(userNotificationCenter:willPresentNotification:withCompletionHandler:))]
        fn al_llegar(
            &self,
            _centro: &UNUserNotificationCenter,
            _aviso: &UNNotification,
            terminar: &DynBlock<dyn Fn(UNNotificationPresentationOptions)>,
        ) {
            terminar.call((UNNotificationPresentationOptions::Banner
                | UNNotificationPresentationOptions::List
                | UNNotificationPresentationOptions::Sound,));
        }

        /// La persona tocó el aviso: se deja la ruta para que `rutas::Atajos` abra Actividad.
        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        fn al_tocar(
            &self,
            _centro: &UNUserNotificationCenter,
            _respuesta: &UNNotificationResponse,
            terminar: &DynBlock<dyn Fn()>,
        ) {
            if let Some(ruta) = crate::push::archivo_ruta() {
                escribir(&ruta, RUTA_ACTIVIDAD);
            }
            terminar.call(());
        }
    }
);

// FUNCIONES

/// Pide permiso para avisar y se registra en APNs. Se llama una vez, al abrir la app.
///
/// Sin cuenta de Apple de pago (o sin el entitlement `aps-environment`) iOS responde con un
/// fallo en vez del token; queda en el log y la app sigue sin push, igual que sin FCM.
pub fn iniciar() {
    let Some(mtm) = MainThreadMarker::new() else {
        dioxus::logger::tracing::warn!("push iOS: no se inició fuera del hilo principal");
        return;
    };

    // El centro guarda al delegado como referencia débil: debe vivir lo que viva la app
    let delegado: Retained<DelegadoAvisos> = unsafe { msg_send![DelegadoAvisos::alloc(mtm), init] };
    let centro = UNUserNotificationCenter::currentNotificationCenter();
    centro.setDelegate(Some(ProtocolObject::from_ref(&*delegado)));
    std::mem::forget(delegado);

    // Mismo permiso que pide Android 13+ al abrir la app
    let respuesta = RcBlock::new(|concedido: Bool, _fallo: *mut NSError| {
        dioxus::logger::tracing::info!("push iOS: permiso concedido = {}", concedido.as_bool());
    });
    centro.requestAuthorizationWithOptions_completionHandler(
        UNAuthorizationOptions::Alert
            | UNAuthorizationOptions::Sound
            | UNAuthorizationOptions::Badge,
        &respuesta,
    );

    // El token llega a los métodos añadidos al delegado de tao; sin ellos iOS lo descarta
    let aplicacion = UIApplication::sharedApplication(mtm);
    // SAFETY: `delegate` devuelve el delegado de la app o nil
    let delegado_app: Option<Retained<AnyObject>> = unsafe { msg_send![&aplicacion, delegate] };
    let Some(delegado_app) = delegado_app else {
        dioxus::logger::tracing::warn!("push iOS: la app no tiene delegado");
        return;
    };
    let clase = delegado_app.class();
    // SAFETY: las dos funciones tienen exactamente la firma `FIRMA` que iOS usa al llamarlas
    unsafe {
        anadir(
            clase,
            sel!(application:didRegisterForRemoteNotificationsWithDeviceToken:),
            {
                type Metodo = unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject, &NSData);
                std::mem::transmute::<Metodo, Imp>(token_recibido)
            },
        );
        anadir(
            clase,
            sel!(application:didFailToRegisterForRemoteNotificationsWithError:),
            {
                type Metodo = unsafe extern "C-unwind" fn(&AnyObject, Sel, &AnyObject, &NSError);
                std::mem::transmute::<Metodo, Imp>(registro_fallido)
            },
        );
    }

    aplicacion.registerForRemoteNotifications();
}

/// Añade un método a la clase del delegado; si tao ya lo tuviera, se deja el suyo.
///
/// # Safety
///
/// `funcion` debe aceptar los argumentos que describe `FIRMA`.
unsafe fn anadir(clase: &AnyClass, selector: Sel, funcion: Imp) {
    let clase = clase as *const AnyClass as *mut AnyClass;
    // SAFETY: lo garantiza quien llama; añadir un método a una clase registrada es válido
    let anadido = unsafe { ffi::class_addMethod(clase, selector, funcion, FIRMA.as_ptr()) };
    if !anadido.as_bool() {
        dioxus::logger::tracing::warn!(
            "push iOS: el delegado ya tenía {}",
            selector.name().to_string_lossy()
        );
    }
}

/// iOS entregó el token de APNs: se guarda en hexadecimal para que `push.rs` lo registre.
unsafe extern "C-unwind" fn token_recibido(
    _delegado: &AnyObject,
    _selector: Sel,
    _aplicacion: &AnyObject,
    token: &NSData,
) {
    let hex: String = token
        .to_vec()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if let Some(ruta) = crate::push::archivo() {
        escribir(&ruta, &hex);
    }
}

/// iOS no pudo registrarse en APNs (simulador, sin entitlement o sin red).
unsafe extern "C-unwind" fn registro_fallido(
    _delegado: &AnyObject,
    _selector: Sel,
    _aplicacion: &AnyObject,
    fallo: &NSError,
) {
    dioxus::logger::tracing::warn!(
        "push iOS: sin token de APNs: {}",
        fallo.localizedDescription()
    );
}

/// Escribe un archivo de la carpeta privada; un fallo solo queda en el log.
fn escribir(ruta: &std::path::Path, texto: &str) {
    if let Err(causa) = std::fs::write(ruta, texto) {
        dioxus::logger::tracing::warn!("push iOS: no se pudo escribir {}: {causa}", ruta.display());
    }
}
