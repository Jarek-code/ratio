//! Pantalla de acceso de Ratio.

use dioxus::prelude::*;

use crate::marca::Marca;

/// Dibuja el formulario de acceso.
///
/// La pantalla es únicamente visual. El botón «Entrar» valida correo y
/// contraseña, pero no autentica;
#[component]
pub fn Login() -> Element {
    // Correo que escribe la persona; se usa como valor del input y para validar.
    let mut correo = use_signal(String::new);

    // Contraseña escrita; nunca se envía a ningún servidor en esta versión.
    let mut contrasena = use_signal(String::new);

    // Controla si la contraseña se muestra como texto o como puntos.
    let mut ver_contrasena = use_signal(|| false);

    // Mensaje de error de validación; `None` significa que no hay error.
    let mut error = use_signal(|| None::<String>);

    // Aviso para funciones que todavía no tienen backend (entrar o crear cuenta).
    let mut aviso = use_signal(|| None::<String>);

    // Maneja el envío del formulario: evita la recarga, valida y muestra un aviso.
    let enviar = move |evento: FormEvent| {
        // Sin esto, el navegador recargaría la página al pulsar «Entrar».
        evento.prevent_default();

        // Se limpian espacios del correo para validar y comparar.
        let email = correo().trim().to_string();
        let clave = contrasena();

        // Validación mínima del correo: no vacío y con una arroba.
        if email.is_empty() || !email.contains('@') {
            error.set(Some("Escribe un correo válido.".into()));
            return;
        }

        // La contraseña no puede quedar vacía.
        if clave.is_empty() {
            error.set(Some("Escribe tu contraseña.".into()));
            return;
        }

        // Los datos son válidos, pero la autenticación aún no está conectada.
        error.set(None);
        aviso.set(Some(
            "El inicio de sesión estará disponible próximamente.".into(),
        ));
    };

    // Estructura visual de la pantalla: marca, tarjeta, campos y avisos.
    rsx! {
        main { class: "login-page",
            section { class: "login-shell",
                // Encabezado de marca: logo, nombre de la app y frase corta.
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

                // Tarjeta del formulario; `onsubmit` llama al manejador `enviar`.
                form { class: "login-card", onsubmit: enviar,
                    // Título y explicación breve del acceso.
                    div { class: "login-card__head",
                        h2 { "Entrar" }
                        p { "Usa tu correo para volver a tu recetario." }
                    }

                    // Solo aparece cuando la validación encuentra un problema.
                    if let Some(mensaje) = error() {
                        div { class: "alerta", role: "alert", "{mensaje}" }
                    }

                    // Campo de correo: guarda lo escrito y marca error de accesibilidad.
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

                    // Campo de contraseña con botón para alternar visibilidad.
                    label { class: "field", r#for: "contrasena",
                        span { "Contraseña" }
                        div { class: "password",
                            input {
                                id: "contrasena",
                                r#type: if ver_contrasena() { "text" } else { "password" },
                                name: "password",
                                autocomplete: "current-password",
                                placeholder: "Tu contraseña",
                                value: "{contrasena}",
                                oninput: move |evento| contrasena.set(evento.value()),
                            }
                            // Cambia entre «Ver» y «Ocultar» sin enviar el formulario.
                            button {
                                r#type: "button",
                                class: "password__toggle",
                                aria_label: if ver_contrasena() { "Ocultar contraseña" } else { "Mostrar contraseña" },
                                onclick: move |_| ver_contrasena.toggle(),
                                if ver_contrasena() { "Ocultar" } else { "Ver" }
                            }
                        }
                    }

                    // Envía el formulario y ejecuta la validación de `enviar`.
                    button { class: "submit", r#type: "submit", "Entrar" }

                    // Opción de alta; por ahora solo muestra un aviso.
                    p { class: "signup",
                        "¿No tienes cuenta? "
                        button {
                            r#type: "button",
                            class: "link",
                            onclick: move |_| {
                                aviso.set(Some("La creación de cuenta estará disponible próximamente.".into()))
                            },
                            "Crear cuenta"
                        }
                    }

                    // Aviso temporal de las acciones que aún no tienen backend.
                    if let Some(mensaje) = aviso() {
                        p { class: "pendiente", role: "status", aria_live: "polite", "{mensaje}" }
                    }

                    // Nota legal sobre contenido con alcohol.
                    p { class: "legal", "Contenido con alcohol solo para personas mayores de edad." }
                }
            }
            // Pie de pantalla con el lema de la aplicación.
            p { class: "footnote", "Ratio · Proporción visible, recetas reproducibles." }
        }
    }
}
