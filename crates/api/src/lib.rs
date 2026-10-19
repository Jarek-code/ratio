//! Contratos y server functions, compilados para el móvil y para el servidor.

use dioxus::prelude::*;
use proporciones::{Detalle, Receta};
use serde::{Deserialize, Serialize};

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

/// Recalcula una receta en el servidor con el mismo motor que usa el cliente.
#[server]
pub async fn analizar(receta: Receta) -> Result<Detalle, ServerFnError> {
    Ok(proporciones::detalle(&receta))
}
