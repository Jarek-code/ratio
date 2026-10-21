//! Contratos y server functions, compilados para el móvil y para el servidor.

use dioxus::prelude::*;
use proporciones::{Detalle, Receta};
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
pub mod db;

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

/// Lo que contesta `/salud`: si la base responde y, si no, por qué.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstadoSalud {
    pub base_de_datos: bool,
    pub detalle: String,
}

/// Recalcula una receta en el servidor con el mismo motor que usa el cliente.
/// El endpoint va fijo: sin él Dioxus añade un sufijo que depende de la ruta del crate al
/// compilar, y el cliente y el servidor desplegado no coincidirían.
#[server(endpoint = "analizar")]
pub async fn analizar(receta: Receta) -> Result<Detalle, ServerFnError> {
    Ok(proporciones::detalle(&receta))
}

/// Estado del backend, para la pantalla de arranque del cliente.
#[server(endpoint = "salud")]
pub async fn salud() -> Result<EstadoSalud, ServerFnError> {
    use std::time::Duration;

    Ok(db::estado(Duration::from_secs(3)).await)
}
