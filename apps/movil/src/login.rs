use dioxus::prelude::*;

use crate::marca::Marca; // Logo compartido, dibujado inline
use crate::red::mensaje; // Texto de un fallo del servidor o de la red

/// Hoja de estilos del acceso; también define la paleta que reutiliza el historial.
const CSS_LOGIN: Asset = asset!("/assets/login.css");

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
fn revisar_correo(correo: &str) -> Option<&'static str> {
    let correo = correo.trim();

    if correo.is_empty() {
        return Some("Escribe tu correo.");
    }
    if correo.contains(char::is_whitespace) {
        return Some("El correo no lleva espacios.");
    }

    let Some((usuario, dominio)) = correo.split_once('@') else {
        return Some("Escribe un correo válido, con arroba y dominio.");
    };

    let dominio_valido = dominio.contains('.')
        && !dominio.starts_with('.')
        && !dominio.ends_with('.')
        && !dominio.contains('@');
    if usuario.is_empty() || !dominio_valido {
        return Some("Escribe un correo válido, con arroba y dominio.");
    }

    None
}

/// Revisa la contraseña antes de mandarla.
fn revisar_contrasena(contrasena: &str) -> Option<&'static str> {
    if contrasena.is_empty() {
        return Some("Escribe tu contraseña.");
    }
    if contrasena.chars().count() < 6 {
        return Some("La contraseña necesita al menos 6 caracteres.");
    }

    None
}

/// Revisa el alias del alta: obligatorio, con largo acotado y sin espacios al inicio ni al final.
fn revisar_alias(alias: &str) -> Option<&'static str> {
    let largo = alias.chars().count();

    if largo == 0 {
        return Some("Escribe un alias.");
    }
    if largo < ALIAS_MINIMO {
        return Some("El alias necesita 3 caracteres o más.");
    }
    if largo > ALIAS_MAXIMO {
        return Some("El alias no puede pasar de 24 caracteres.");
    }
    if alias.contains('@') {
        return Some("El alias no lleva arroba: es el nombre que verán los demás.");
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
    let mut enviando = use_signal(|| false);

    let crear = modo() == Modo::Crear;
    let titulo = if crear { "Crear cuenta" } else { "Entrar" };
    let bajada = if crear {
        "Tu alias y tu edad quedan en el perfil; la contraseña la guarda Supabase cifrada."
    } else {
        "Usa tu correo para volver a tu recetario."
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

        if let Some(aviso) = revisar_correo(&email) {
            error.set(Some(aviso.to_string()));
            return;
        }
        if let Some(aviso) = revisar_contrasena(&clave) {
            error.set(Some(aviso.to_string()));
            return;
        }
        if es_alta && let Some(aviso) = revisar_alias(&nombre) {
            error.set(Some(aviso.to_string()));
            return;
        }
        if es_alta && !es_adulto {
            error.set(Some(
                "Necesitamos que declares que eres mayor de edad.".into(),
            ));
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
        document::Stylesheet { href: CSS_LOGIN }

        main { class: "login-page",
            section { class: "login-shell",
                div { class: "login-brand",
                    div { class: "brand-row",
                        Marca {}
                        div {
                            p { class: "eyebrow", "Café · Coctelería" }
                            h1 { "Ratio" }
                        }
                    }
                    p { class: "tagline", "La proporción se ve en el vaso." }
                }

                form { class: "login-card", onsubmit: enviar,
                    div { class: "login-card__head",
                        h2 { "{titulo}" }
                        p { "{bajada}" }
                    }

                    if let Some(mensaje) = error() {
                        div { class: "alerta", role: "alert", "{mensaje}" }
                    }

                    label { class: "field", r#for: "correo",
                        span { "Correo" }
                        input {
                            id: "correo",
                            r#type: "email",
                            name: "email",
                            autocomplete: "email",
                            autocapitalize: "none",
                            inputmode: "email",
                            spellcheck: "false",
                            placeholder: "tu@correo.com",
                            value: "{correo}",
                            aria_invalid: if error().is_some() { "true" } else { "false" },
                            oninput: move |evento| correo.set(evento.value()),
                        }
                    }

                    label { class: "field", r#for: "contrasena",
                        span { "Contraseña" }
                        div { class: "password",
                            input {
                                id: "contrasena",
                                r#type: if ver_contrasena() { "text" } else { "password" },
                                name: "password",
                                autocomplete: if crear { "new-password" } else { "current-password" },
                                placeholder: if crear { "Al menos 6 caracteres" } else { "Tu contraseña" },
                                value: "{contrasena}",
                                oninput: move |evento| contrasena.set(evento.value()),
                            }
                            button {
                                r#type: "button",
                                class: "password__toggle",
                                aria_label: if ver_contrasena() { "Ocultar contraseña" } else { "Mostrar contraseña" },
                                onclick: move |_| ver_contrasena.toggle(),
                                if ver_contrasena() { "Ocultar" } else { "Ver" }
                            }
                        }
                    }

                    if crear {
                        label { class: "field", r#for: "alias",
                            span { "Alias" }
                            input {
                                id: "alias",
                                r#type: "text",
                                name: "alias",
                                autocomplete: "nickname",
                                placeholder: "Como te verán en La Barra",
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
                            span { "Tengo 18 años o más y quiero ver recetas con alcohol." }
                        }
                    }

                    button {
                        class: "submit",
                        r#type: "submit",
                        disabled: enviando(),
                        if enviando() { "Un momento…" } else { "{titulo}" }
                    }

                    p { class: "switch-modo",
                        if crear {
                            "¿Ya tienes cuenta? "
                            button {
                                r#type: "button",
                                class: "link",
                                onclick: move |_| {
                                    error.set(None);
                                    modo.set(Modo::Entrar);
                                },
                                "Entrar"
                            }
                        } else {
                            "¿No tienes cuenta? "
                            button {
                                r#type: "button",
                                class: "link",
                                onclick: move |_| {
                                    error.set(None);
                                    modo.set(Modo::Crear);
                                },
                                "Crear cuenta"
                            }
                        }
                    }

                    p { class: "legal", "Contenido con alcohol solo para personas mayores de edad." }
                }
            }
            p { class: "footnote", "Ratio · Proporción visible, recetas reproducibles." }
        }
    }
}
