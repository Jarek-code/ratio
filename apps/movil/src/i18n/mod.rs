//! Textos de la interfaz en los tres idiomas que ofrece la app.
//!
//! Cada texto visible vive una sola vez, con su versión en español, inglés y mandarín
//! simplificado. Las pantallas lo piden con [`ahora`] (idioma en uso) o con [`t`] (idioma
//! explícito), y las fechas se escriben con [`fecha`], que respeta el orden de cada idioma.
//!
//! Los textos con un dato dentro llevan `{}` en el lugar del valor: el llamador lo rellena con
//! [`con`], que sustituye los huecos en orden. Así el orden de las palabras puede cambiar
//! entre idiomas sin tocar el código.

mod en;
mod es;
mod textos;
mod zh;

pub use textos::Texto;

use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

use serde::Deserialize;
use serde::Serialize;

// ESTRUCTURAS

/// Idioma de la interfaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Idioma {
    /// Español de México, el idioma de origen.
    #[default]
    Espanol,
    /// Inglés.
    Ingles,
    /// Mandarín simplificado.
    Mandarin,
}

impl Idioma {
    /// Los idiomas en el orden en que se ofrecen.
    pub const TODOS: [Idioma; 3] = [Idioma::Espanol, Idioma::Ingles, Idioma::Mandarin];

    /// Nombre del idioma escrito en ese mismo idioma.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Idioma::Espanol => "Español",
            Idioma::Ingles => "English",
            Idioma::Mandarin => "中文",
        }
    }

    /// Código corto del idioma (ISO 639-1).
    pub fn clave(self) -> &'static str {
        match self {
            Idioma::Espanol => "es",
            Idioma::Ingles => "en",
            Idioma::Mandarin => "zh",
        }
    }

    /// Número con el que se guarda el idioma en uso.
    fn como_u8(self) -> u8 {
        match self {
            Idioma::Espanol => 0,
            Idioma::Ingles => 1,
            Idioma::Mandarin => 2,
        }
    }

    /// Idioma a partir del número guardado; un valor raro cae en español.
    fn desde_u8(valor: u8) -> Idioma {
        match valor {
            1 => Idioma::Ingles,
            2 => Idioma::Mandarin,
            _ => Idioma::Espanol,
        }
    }
}

/// Un texto de la interfaz.

// FUNCIONES

/// Idioma en uso, para el texto que no se arma dentro de un componente.
static EN_USO: AtomicU8 = AtomicU8::new(0);

/// Guarda el idioma elegido para las funciones que no reciben la señal de ajustes.
pub fn fijar_actual(idioma: Idioma) {
    EN_USO.store(idioma.como_u8(), Ordering::Relaxed);
}

/// Idioma en uso.
pub fn actual() -> Idioma {
    Idioma::desde_u8(EN_USO.load(Ordering::Relaxed))
}

/// Texto en el idioma en uso.
pub fn ahora(texto: Texto) -> &'static str {
    t(actual(), texto)
}

/// Texto en el idioma elegido.
pub fn t(idioma: Idioma, texto: Texto) -> &'static str {
    match idioma {
        Idioma::Espanol => es::texto(texto),
        Idioma::Ingles => en::texto(texto),
        Idioma::Mandarin => zh::texto(texto),
    }
}

/// Texto con sus huecos `{}` rellenos, en orden.
pub fn con(idioma: Idioma, texto: Texto, datos: &[&str]) -> String {
    let mut salida = t(idioma, texto).to_string();
    for dato in datos {
        if let Some(hueco) = salida.find("{}") {
            salida.replace_range(hueco..hueco + 2, dato);
        }
    }

    salida
}

/// Fecha completa en el idioma elegido.
///
/// `mes` va de 1 (enero) a 12 (diciembre). El orden de los datos y los separadores cambian
/// con el idioma: «12 oct 2026», «Oct 12, 2026» o «2026年10月12日».
pub fn fecha(idioma: Idioma, dia: u32, mes: u32, anio: i32) -> String {
    match idioma {
        Idioma::Espanol => es::fecha(dia, mes, anio),
        Idioma::Ingles => en::fecha(dia, mes, anio),
        Idioma::Mandarin => zh::fecha(dia, mes, anio),
    }
}

/// Fecha completa en el idioma en uso.
pub fn fecha_ahora(dia: u32, mes: u32, anio: i32) -> String {
    fecha(actual(), dia, mes, anio)
}

/// «Hace cuánto» en el idioma en uso, a partir de los segundos transcurridos.
///
/// El servidor manda el dato en segundos y aquí se eligen las palabras: «hace 5 min»,
/// «5 min ago» o «5 分钟前».
pub fn hace(segundos: i64) -> String {
    let segundos = segundos.max(0);
    let idioma = actual();

    match segundos {
        0..60 => t(idioma, Texto::HaceMomentos).to_string(),
        60..3_600 => con(idioma, Texto::HaceMinutos, &[&(segundos / 60).to_string()]),
        3_600..86_400 => con(idioma, Texto::HaceHoras, &[&(segundos / 3_600).to_string()]),
        _ => con(idioma, Texto::HaceDias, &[&(segundos / 86_400).to_string()]),
    }
}

/// Escribe una fecha neutra (`2026-11-12`) en el idioma en uso.
///
/// El servidor manda las fechas así porque no sabe qué idioma eligió la persona. Si el texto
/// no tiene esa forma se devuelve tal cual: un dato viejo o raro se ve, no se rompe.
pub fn fecha_iso(neutra: &str) -> String {
    let mut partes = neutra.split('-');

    let (Some(anio), Some(mes), Some(dia), None) =
        (partes.next(), partes.next(), partes.next(), partes.next())
    else {
        return neutra.to_string();
    };

    match (anio.parse::<i32>(), mes.parse::<u32>(), dia.parse::<u32>()) {
        (Ok(anio), Ok(mes), Ok(dia)) => fecha_ahora(dia, mes, anio),
        _ => neutra.to_string(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Los tres idiomas tienen que cubrir todos los textos y ninguno puede quedar vacío.
    #[test]
    fn todos_los_textos_estan_en_los_tres_idiomas() {
        let textos = [
            Texto::NavAria,
            Texto::Reintentar,
            Texto::RedTiempo,
            Texto::Entrar,
            Texto::Correo,
            Texto::Ajustes,
            Texto::BienvenidaPaso,
            Texto::Empezar,
        ];

        for texto in textos {
            for idioma in Idioma::TODOS {
                assert!(!t(idioma, texto).trim().is_empty(), "{idioma:?} {texto:?}");
            }
        }
    }

    /// Los huecos se rellenan en orden y sin dejar marcas.
    #[test]
    fn los_huecos_se_rellenan() {
        for idioma in Idioma::TODOS {
            let plantilla = t(idioma, Texto::RedTiempo);
            assert!(plantilla.contains("{}"), "{plantilla}");

            let relleno = con(idioma, Texto::RedTiempo, &["20", "3"]);
            assert!(relleno.contains("20"), "{relleno}");
            assert!(!relleno.contains("{}"), "{relleno}");
        }
    }

    /// Un texto sin huecos se devuelve tal cual, en el idioma que sea.
    #[test]
    fn sin_huecos_no_cambia_nada() {
        for idioma in Idioma::TODOS {
            assert_eq!(
                con(idioma, Texto::Reintentar, &["1"]),
                t(idioma, Texto::Reintentar)
            );
        }
    }
}
