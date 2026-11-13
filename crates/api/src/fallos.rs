//! Fallos que viajan del servidor al cliente con un código que dice qué hacer con ellos.
//!
//! Un mensaje sin código se lee igual que un fallo del servidor, así que la app no sabe si
//! reintentar, pedir la sesión otra vez o avisar de que un dato está mal. Aquí cada fallo
//! lleva su código y su texto en español, listo para mostrarse tal cual.

use dioxus::prelude::*;

/// Un dato que la persona puede corregir: un campo vacío, un correo sin arroba, un id roto.
pub const CODIGO_DATOS: u16 = 422;

/// Un fallo temporal del servidor (base de datos lenta, Supabase sin contestar).
pub const CODIGO_REINTENTO: u16 = 503;

/// La sesión ya no vale: hay que volver a entrar.
pub const CODIGO_SESION: u16 = 401;

/// Lo pedido no existe o la cuenta no puede verlo.
pub const CODIGO_NO_ENCONTRADO: u16 = 404;

/// Fallo con código propio, para que el cliente sepa qué hacer sin mirar el texto.
fn con_codigo(codigo: u16, mensaje: impl ToString) -> ServerFnError {
    ServerFnError::ServerError {
        message: mensaje.to_string(),
        code: codigo,
        details: None,
    }
}

/// Aviso de un dato que la persona puede corregir.
pub fn datos(mensaje: impl ToString) -> ServerFnError {
    con_codigo(CODIGO_DATOS, mensaje)
}

/// Aviso de un fallo temporal: reintentar tiene sentido.
pub fn reintento(mensaje: impl ToString) -> ServerFnError {
    con_codigo(CODIGO_REINTENTO, mensaje)
}

/// Aviso de que la sesión caducó o no se envió.
pub fn sesion(mensaje: impl ToString) -> ServerFnError {
    con_codigo(CODIGO_SESION, mensaje)
}

/// Aviso de que lo pedido no existe o no es visible para la cuenta.
pub fn no_encontrado(mensaje: impl ToString) -> ServerFnError {
    con_codigo(CODIGO_NO_ENCONTRADO, mensaje)
}

/// Código con el que viajó el fallo; `500` cuando no lo dice.
pub fn codigo(causa: &ServerFnError) -> u16 {
    match causa {
        ServerFnError::ServerError { code, .. } => *code,
        _ => 500,
    }
}

/// Texto del fallo sin los adornos que añade el transporte.
pub fn texto(causa: &ServerFnError) -> String {
    match causa {
        ServerFnError::ServerError { message, .. } => message.clone(),
        ServerFnError::Request(error) => error.to_string(),
        otra => otra.to_string(),
    }
}

/// ¿El mismo envío puede funcionar si se repite? Solo los fallos temporales y los de red.
pub fn es_reintentable(causa: &ServerFnError) -> bool {
    match causa {
        ServerFnError::Request(_) => true, // Sin red, corte a media respuesta o plazo agotado
        ServerFnError::ServerError { code, .. } => *code == CODIGO_REINTENTO || *code >= 500,
        _ => false,
    }
}

/// ¿La cuenta tiene que volver a entrar?
pub fn es_sesion(causa: &ServerFnError) -> bool {
    codigo(causa) == CODIGO_SESION
}

/// ¿El fallo es un dato que la persona puede corregir?
pub fn es_datos(causa: &ServerFnError) -> bool {
    codigo(causa) == CODIGO_DATOS
}

/// Comprueba la forma de un UUID (`8-4-4-4-12`) antes de mandarlo a la base.
///
/// Evita que un identificador vacío o inventado llegue a `$1::uuid` y vuelva como error de
/// consulta, que la persona no puede entender ni corregir.
pub fn es_uuid(texto: &str) -> bool {
    let bytes = texto.as_bytes();
    if bytes.len() != 36 {
        return false;
    }

    bytes.iter().enumerate().all(|(posicion, byte)| match posicion {
        8 | 13 | 18 | 23 => *byte == b'-',
        _ => byte.is_ascii_hexdigit(),
    })
}
