//! Contratos y server functions, compilados para el móvil y para el servidor.

use dioxus::prelude::*;
use proporciones::{Detalle, Receta};
use serde::{Deserialize, Serialize};

pub mod auth; // Acceso, alta y verificación de la sesión
#[cfg(feature = "server")]
pub mod base; // Pool y plazo de las consultas (solo servidor)
pub mod biblioteca; // Catálogo completo de la pantalla Biblioteca (B2)
#[cfg(feature = "server")]
pub mod db;
#[cfg(feature = "server")]
mod filas; // Filas de vaso y capa compartidas por las consultas
pub mod descubrir; // Sugerencias del día de la pantalla Descubrir (B1)
pub mod fallos; // Fallos con código: dato inválido, reintento, sesión o no encontrado
pub mod foro; // La Barra: publicar, hilo, comentar, valorar, actividad y reportes (D1-D7)
pub mod guardados; // Recetas guardadas por la cuenta (C5)
pub mod guardar; // C2: guardar una version privada ajustada
pub mod historial; // Árbol de versiones de una bebida (B6)
pub mod ingredientes; // Catálogo de ingredientes para añadir capas desde C1
pub mod perfil; // Perfil público y seguimiento (E2, E3)
pub mod receta; // Ficha completa de una receta (B3)
pub mod recetario; // Recetario propio de la cuenta (C4)
pub mod vasos; // Catálogo de vasos para el editor de proporciones (C1)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rol {
    Usuario,
    Moderador,
}

/// Quién llama, según el JWT que ya verificó el servidor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sesion {
    pub usuario_id: String,
    pub alias: String,
    pub rol: Rol,
    pub mayor_de_edad: bool,
}

/// Convierte la cantidad de un ingrediente a mililitros equivalentes.
///
/// De momento la equivalencia temporal es `1 g = 1 ml`. Como el catálogo guarda los
/// sólidos en `mg`, hay que dividir entre 1000; los líquidos ya vienen en `ml`.
pub fn ml_equivalentes(cantidad: f64, unidad: &str) -> f64 {
    if unidad == "mg" {
        cantidad / 1000.0
    } else {
        cantidad
    }
}

/// Lo que contesta `/salud`: si la base responde y, si no, por qué.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstadoSalud {
    pub base_de_datos: bool,
    pub detalle: String,
}

/// Recalcula una receta en el servidor con el mismo motor que usa el cliente.
/// El endpoint va fijo: sin él Dioxus añade un sufijo que depende de la ruta del crate al
/// compilar, y el cliente y el servidor desplegado no coincidirían.
///
/// # Errores
///
/// Devuelve [`fallos::CODIGO_DATOS`] si la receta llega vacía o sin capacidad: son datos del
/// formulario, no un fallo del servidor.
#[server(endpoint = "analizar")]
pub async fn analizar(receta: Receta) -> Result<Detalle, ServerFnError> {
    if receta.ingredientes.is_empty() {
        return Err(fallos::datos("La receta no tiene ingredientes que analizar."));
    }
    if !(receta.capacidad_ml > 0.0) {
        return Err(fallos::datos("El vaso necesita una capacidad mayor que cero."));
    }

    Ok(proporciones::detalle(&receta))
}

/// Estado del backend, para la pantalla de arranque del cliente.
///
/// Siempre contesta `Ok`: que la base esté caída se cuenta en `base_de_datos` y `detalle`, no
/// como fallo de la petición.
#[server(endpoint = "salud")]
pub async fn salud() -> Result<EstadoSalud, ServerFnError> {
    use std::time::Duration;

    Ok(db::estado(Duration::from_secs(3)).await)
}
