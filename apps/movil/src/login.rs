use dioxus::prelude::*;

use crate::i18n::Texto; // Texto traducible
use crate::i18n::t; // Texto en el idioma elegido
use crate::marca::Marca; // Logo compartido, dibujado inline
use crate::red::mensaje; // Texto de un fallo del servidor o de la red

/// Hoja de estilos del acceso.
const CSS_LOGIN: Asset = asset!("/assets/login.css");

/// Hoja del sistema visual que comparten las pantallas: trae la paleta y el tema oscuro.
const CSS_BASE: Asset = asset!("/assets/descubrir.css");

/// Qué hace el formulario: entrar con una cuenta o crear una nueva.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modo {
    /// Acceso con una cuenta existente.
    Entrar,
    /// Alta de una cuenta nueva.
    Crear,
}

/// Largo mínimo y máximo del alias, igual que el CHECK de la tabla `usuario`.
const ALIAS_MINIMO: usize = 3;
const ALIAS_MAXIMO: usize = 24;

/// Revisa el correo antes de mandarlo: vacío, sin arroba o sin dominio.
///
/// Devuelve el aviso que se le muestra a la persona, o `None` si el correo tiene forma de
/// correo. La comprobación no sustituye a la de Supabase: solo evita un viaje inútil.
fn revisar_correo(correo: &str) -> Option<Texto> {
    let correo = correo.trim();

    if correo.is_empty() {
        return Some(Texto::FaltaCorreo);
    }
    if correo.contains(char::is_whitespace) {
        return Some(Texto::CorreoConEspacios);
    }

    let Some((usuario, dominio)) = correo.split_once('@') else {
        return Some(Texto::CorreoInvalido);
    };

    let dominio_valido = dominio.contains('.')
        && !dominio.starts_with('.')
        && !dominio.ends_with('.')
        && !dominio.contains('@');
    if usuario.is_empty() || !dominio_valido {
        return Some(Texto::CorreoInvalido);
    }

    None
}

/// Revisa la contraseña antes de mandarla.
fn revisar_contrasena(contrasena: &str) -> Option<Texto> {
    if contrasena.is_empty() {
        return Some(Texto::FaltaContrasena);
    }
    if contrasena.chars().count() < 6 {
        return Some(Texto::ContrasenaCorta);
    }

    None
}

/// Revisa el alias del alta: obligatorio, con largo acotado y sin espacios al inicio ni al final.
fn revisar_alias(alias: &str) -> Option<Texto> {
    let largo = alias.chars().count();

    if largo == 0 {
        return Some(Texto::FaltaAlias);
    }
    if largo < ALIAS_MINIMO {
        return Some(Texto::AliasCorto);
    }
    if largo > ALIAS_MAXIMO {
        return Some(Texto::AliasLargo);
    }
    if alias.contains('@') {
        return Some(Texto::AliasConArroba);
    }

    None
}

/// Acceso y alta contra Supabase Auth; guarda la sesión y avisa con `on_entrar`.
#[component]
pub fn Login(on_entrar: EventHandler<()>) -> Element {
    let mut sesion = use_context::<crate::sesion::SenalSesion>();
    let mut modo = use_signal(|| Modo::Entrar);
    let mut correo = use_signal(String::new);
    let mut contrasena = use_signal(String::new);
    let mut alias = use_signal(String::new);
    let mut adulto = use_signal(|| false);
    let mut ver_contrasena = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut aviso = use_signal(|| None::<String>);
    let mut enviando = use_signal(|| false);
    let mut recuperando = use_signal(|| false);

    let idioma = use_context::<crate::ajustes::SenalAjustes>()().idioma;
    let crear = modo() == Modo::Crear;
    let titulo = if crear {
        t(idioma, Texto::CrearCuenta)
    } else {
        t(idioma, Texto::Entrar)
    };
    let bajada = if crear {
        t(idioma, Texto::BajadaCrear)
    } else {
        t(idioma, Texto::BajadaEntrar)
    };

    let enviar = move |evento: FormEvent| {
        evento.prevent_default();

        if enviando() {
            return;
        }

        let email = correo().trim().to_string();
        let clave = contrasena();
        let es_alta = modo() == Modo::Crear;
        let nombre = alias().trim().to_string();
        let es_adulto = adulto();

        // Los avisos se traducen aquí: las revisiones solo dicen cuál es el problema
        let aviso_texto = |texto: Texto, datos: &[&str]| crate::i18n::con(idioma, texto, datos);

        if let Some(texto) = revisar_correo(&email) {
            error.set(Some(aviso_texto(texto, &[])));
            return;
        }
        if let Some(texto) = revisar_contrasena(&clave) {
            error.set(Some(aviso_texto(
                texto,
                &[&api::auth::CONTRASENA_MINIMA.to_string()],
            )));
            return;
        }
        if es_alta && let Some(texto) = revisar_alias(&nombre) {
            let datos = if texto == Texto::AliasLargo {
                [api::auth::ALIAS_MAXIMO.to_string()]
            } else {
                [api::auth::ALIAS_MINIMO.to_string()]
            };
            error.set(Some(aviso_texto(texto, &[&datos[0]])));
            return;
        }
        if es_alta && !es_adulto {
            error.set(Some(t(idioma, Texto::FaltaEdad).to_string()));
            return;
        }

        error.set(None);
        enviando.set(true);

        spawn(async move {
            // El acceso se puede repetir sin efectos dobles; el alta no, para no crear dos cuentas
            let resultado = if es_alta {
                crate::red::con_plazo(api::auth::registrar(api::auth::Registro {
                    correo: email,
                    contrasena: clave,
                    alias: nombre,
                    mayor_de_edad: es_adulto,
                }))
                .await
            } else {
                crate::red::con_reintentos(move || {
                    api::auth::entrar(api::auth::Credenciales {
                        correo: email.clone(),
                        contrasena: clave.clone(),
                    })
                })
                .await
            };

            enviando.set(false);

            match resultado {
                Ok(iniciada) => {
                    crate::sesion::iniciar(&mut sesion, iniciada);
                    on_entrar.call(());
                }
                Err(causa) => error.set(Some(mensaje(&causa))),
            }
        });
    };

    rsx! {
        document::Stylesheet { href: CSS_BASE }
        document::Stylesheet { href: CSS_LOGIN }

        main { class: "login-page",
            section { class: "login-shell",
                div { class: "login-brand",
                    div { class: "brand-row",
                        Marca {}
                        div {
                            p { class: "eyebrow", "{t(idioma, Texto::MarcaRubro)}" }
                            h1 { "Ratio" }
                        }
                    }
                    p { class: "tagline", "{t(idioma, Texto::MarcaLema)}" }
                }

                form { class: "login-card", onsubmit: enviar,
                    div { class: "login-card__head",
                        h2 { "{titulo}" }
                        p { "{bajada}" }
                    }

                    if let Some(mensaje) = error() {
                        div { class: "alerta", role: "alert", "{mensaje}" }
                    }

                    if let Some(mensaje) = aviso() {
                        p { class: "aviso-login", role: "status", "{mensaje}" }
                    }

                    label { class: "field", r#for: "correo",
                        span { "{t(idioma, Texto::Correo)}" }
                        input {
                            id: "correo",
                            r#type: "email",
                            name: "email",
                            autocomplete: "email",
                            autocapitalize: "none",
                            inputmode: "email",
                            spellcheck: "false",
                            placeholder: "{t(idioma, Texto::PlaceholderCorreo)}",
                            value: "{correo}",
                            aria_invalid: if error().is_some() { "true" } else { "false" },
                            oninput: move |evento| correo.set(evento.value()),
                        }
                    }

                    label { class: "field", r#for: "contrasena",
                        span { "{t(idioma, Texto::Contrasena)}" }
                        div { class: "password",
                            input {
                                id: "contrasena",
                                r#type: if ver_contrasena() { "text" } else { "password" },
                                name: "password",
                                autocomplete: if crear { "new-password" } else { "current-password" },
                                placeholder: if crear { t(idioma, Texto::PlaceholderContrasenaCrear) } else { t(idioma, Texto::PlaceholderContrasenaEntrar) },
                                value: "{contrasena}",
                                oninput: move |evento| contrasena.set(evento.value()),
                            }
                            button {
                                r#type: "button",
                                class: "password__toggle",
                                aria_label: if ver_contrasena() { t(idioma, Texto::OcultarContrasena) } else { t(idioma, Texto::MostrarContrasena) },
                                onclick: move |_| ver_contrasena.toggle(),
                                if ver_contrasena() { "{t(idioma, Texto::OcultarContrasena)}" } else { "{t(idioma, Texto::MostrarContrasena)}" }
                            }
                        }
                    }

                    if crear {
                        label { class: "field", r#for: "alias",
                            span { "{t(idioma, Texto::Alias)}" }
                            input {
                                id: "alias",
                                r#type: "text",
                                name: "alias",
                                autocomplete: "nickname",
                                placeholder: "{t(idioma, Texto::PlaceholderAlias)}",
                                value: "{alias}",
                                oninput: move |evento| alias.set(evento.value()),
                            }
                        }

                        label { class: "check",
                            input {
                                r#type: "checkbox",
                                checked: adulto(),
                                onchange: move |evento| adulto.set(evento.checked()),
                            }
                            span { "{t(idioma, Texto::DeclaracionEdad)}" }
                        }
                    }

                    if !crear {
                        button {
                            class: "link recuperar",
                            r#type: "button",
                            disabled: recuperando(),
                            onclick: move |_| {
                                if recuperando() {
                                    return;
                                }

                                let email = correo().trim().to_string();
                                // El correo se revisa aquí: sin él no hay a dónde mandar el enlace
                                if let Some(motivo) = revisar_correo(&email) {
                                    error.set(Some(t(idioma, motivo).to_string()));
                                    return;
                                }

                                error.set(None);
                                aviso.set(None);
                                recuperando.set(true);
                                spawn(async move {
                                    let resultado = crate::red::con_plazo(api::auth::recuperar(email)).await;
                                    recuperando.set(false);

                                    match resultado {
                                        Ok(()) => aviso.set(Some(
                                            t(idioma, Texto::CorreoEnviado).to_string(),
                                        )),
                                        Err(causa) => error.set(Some(mensaje(&causa))),
                                    }
                                });
                            },
                            if recuperando() { "{t(idioma, Texto::MandandoCorreo)}" } else { "{t(idioma, Texto::OlvidasteContrasena)}" }
                        }
                    }

                    button {
                        class: "submit",
                        r#type: "submit",
                        disabled: enviando(),
                        if enviando() { "{t(idioma, Texto::UnMomento)}" } else { "{titulo}" }
                    }

                    p { class: "switch-modo",
                        if crear {
                            "{t(idioma, Texto::YaTienesCuenta)}"
                            button {
                                r#type: "button",
                                class: "link",
                                onclick: move |_| {
                                    error.set(None);
                                    aviso.set(None);
                                    modo.set(Modo::Entrar);
                                },
                                "{t(idioma, Texto::Entrar)}"
                            }
                        } else {
                            "{t(idioma, Texto::NoTienesCuenta)}"
                            button {
                                r#type: "button",
                                class: "link",
                                onclick: move |_| {
                                    error.set(None);
                                    aviso.set(None);
                                    modo.set(Modo::Crear);
                                },
                                "{t(idioma, Texto::CrearCuenta)}"
                            }
                        }
                    }

                    p { class: "legal", "{t(idioma, Texto::AvisoLegal)}" }
                }
            }
            p { class: "footnote", "{t(idioma, Texto::PieAcceso)}" }
        }
    }
}
