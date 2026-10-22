//! Sesión de la app: el token de Supabase y el perfil, compartidos por contexto.

use dioxus::prelude::*;

/// Sesión iniciada: el token que se reenvía al backend y el perfil que lo acompaña.
#[derive(Debug, Clone, PartialEq)]
pub struct SesionApp {
    /// Token de acceso que emite Supabase Auth.
    pub token: String,
    /// Perfil del usuario, leído de `usuario`.
    pub perfil: api::Sesion,
}

/// Señal con la sesión; `None` mientras nadie haya entrado.
pub type SenalSesion = Signal<Option<SesionApp>>;

/// Token de la sesión actual, si la hay.
pub fn token(sesion: &SenalSesion) -> Option<String> {
    sesion.read().as_ref().map(|guardada| guardada.token.clone())
}
