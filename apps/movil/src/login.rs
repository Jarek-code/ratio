use dioxus::prelude::*;

/// Estilos del login; replica el frame `Login · móvil` de Penpot.
///
/// El primer selector debe ser una regla real: Dioxus antepone un nodo de
/// hidratación al primer selector y, si empieza por `:root`, rompe las variables.
pub const CSS: &str = r#"
.__ratio_style_anchor {
    display: contents;
}

/* Paleta tomada del frame de Penpot. */
:root {
    color-scheme: light;
    --crema: #F5EFE5;
    --papel: #FFFDF9;
    --tinta: #2A1B14;
    --tinta-suave: #7A6A5F;
    --borde: #DFD2C2;
    --cobre: #B9633B;
    --cobre-oscuro: #43281C;
}

* {
    box-sizing: border-box;
}

html,
body {
    margin: 0;
    min-height: 100%;
}

body {
    background: var(--crema);
    color: var(--tinta);
    font-family: Inter, "Segoe UI", "Helvetica Neue", system-ui, -apple-system, sans-serif;
    -webkit-font-smoothing: antialiased;
}

/* Fondo, centrado y textura sutil del acceso. */
.login-page {
    position: relative;
    display: flex;
    min-height: 100vh;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 22px;
    overflow: hidden;
    padding: 28px 20px;
    background:
        radial-gradient(circle at 12% 8%, #EADFCF 0, transparent 42%),
        radial-gradient(circle at 90% 92%, #E9D3C4 0, transparent 38%),
        var(--crema);
}

.login-page::before {
    position: absolute;
    inset: 0;
    background-image: linear-gradient(rgba(90, 62, 44, 0.035) 1px, transparent 1px),
        linear-gradient(90deg, rgba(90, 62, 44, 0.035) 1px, transparent 1px);
    background-size: 32px 32px;
    content: "";
    mask-image: radial-gradient(circle at center, black, transparent 78%);
    pointer-events: none;
}

.login-shell {
    position: relative;
    z-index: 1;
    display: flex;
    width: min(100%, 420px);
    flex-direction: column;
    gap: 24px;
}

.login-brand {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
}

.brand-row {
    display: flex;
    align-items: center;
    gap: 12px;
}

.brand-mark {
    width: 40px;
    height: 40px;
    flex: none;
    color: var(--tinta);
}

.eyebrow {
    margin: 0 0 2px;
    color: var(--cobre);
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
}

.login-brand h1 {
    margin: 0;
    font-family: "DM Serif Display", Georgia, "Times New Roman", serif;
    font-size: 42px;
    font-weight: 400;
    letter-spacing: -0.03em;
    line-height: 1;
}

.tagline {
    margin: 0;
    color: var(--tinta-suave);
    font-size: 14px;
    line-height: 1.5;
}

/* Tarjeta principal del formulario. */
.login-card {
    display: flex;
    flex-direction: column;
    gap: 18px;
    border: 1px solid rgba(120, 90, 70, 0.16);
    border-radius: 26px;
    background: rgba(255, 253, 249, 0.88);
    padding: 24px;
    box-shadow: 0 28px 70px -34px rgba(58, 35, 22, 0.5);
    backdrop-filter: blur(14px);
}

.login-card__head h2 {
    margin: 0;
    font-family: "DM Serif Display", Georgia, "Times New Roman", serif;
    font-size: 26px;
    font-weight: 400;
    letter-spacing: -0.02em;
}

.login-card__head p {
    margin: 6px 0 0;
    color: var(--tinta-suave);
    font-size: 13.5px;
    line-height: 1.55;
}

.field {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.field > span {
    color: #5C4A3E;
    font-size: 12px;
    font-weight: 650;
    letter-spacing: 0.015em;
}

.field input {
    width: 100%;
    border: 1px solid var(--borde);
    border-radius: 14px;
    background: var(--papel);
    padding: 14px 15px;
    color: var(--tinta);
    font: inherit;
    font-size: 16px;
    outline: none;
    transition: border-color 150ms ease, box-shadow 150ms ease;
}

.field input:focus {
    border-color: var(--cobre);
    box-shadow: 0 0 0 4px rgba(185, 99, 59, 0.14);
}

.field input::placeholder {
    color: #B3A396;
}

.password {
    position: relative;
}

.password input {
    padding-right: 88px;
}

.password__toggle {
    position: absolute;
    top: 50%;
    right: 7px;
    border: 0;
    border-radius: 10px;
    background: transparent;
    padding: 9px 10px;
    color: #9A5A3A;
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    font-weight: 650;
    transform: translateY(-50%);
}

.password__toggle:hover {
    background: #F3E8DC;
}

.password__toggle:focus-visible,
.submit:focus-visible,
.link:focus-visible {
    outline: 3px solid rgba(185, 99, 59, 0.35);
    outline-offset: 2px;
}

.submit {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    border: 0;
    border-radius: 14px;
    background: var(--tinta);
    padding: 15px 18px;
    color: #FFF9F2;
    cursor: pointer;
    font: inherit;
    font-size: 15px;
    font-weight: 650;
    transition: background 150ms ease, transform 120ms ease;
}

.submit:hover {
    background: var(--cobre-oscuro);
}

.submit:active {
    transform: translateY(1px);
}

/* Enlace secundario para crear una cuenta. */
.link {
    border: 0;
    background: transparent;
    padding: 0;
    color: var(--cobre);
    cursor: pointer;
    font: inherit;
    font-size: 12.5px;
    font-weight: 650;
}

.link:hover {
    text-decoration: underline;
}

.signup {
    margin: 0;
    color: var(--tinta-suave);
    font-size: 12.5px;
    line-height: 1.4;
    text-align: center;
}

.pendiente {
    margin: 0;
    border: 1px solid #E6D3BF;
    border-radius: 10px;
    background: #FBF1E7;
    padding: 9px 11px;
    color: #6D503D;
    font-size: 12px;
    line-height: 1.4;
    text-align: center;
}

.alerta {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    border: 1px solid #F3C9C1;
    border-radius: 14px;
    background: #FCEDEA;
    padding: 12px 14px;
    color: #8E2C22;
    font-size: 13px;
    line-height: 1.45;
}

.alerta::before {
    flex: none;
    content: "!";
    font-weight: 800;
}

.legal {
    margin: 0;
    color: #8B7A6E;
    font-size: 11.5px;
    line-height: 1.5;
    text-align: center;
}

.footnote {
    position: relative;
    z-index: 1;
    margin: 0;
    color: #A08D7E;
    font-size: 11px;
    letter-spacing: 0.04em;
    text-align: center;
}

/* En escritorio, marca y tarjeta se alinean en dos columnas. */
@media (min-width: 860px) {
    .login-page {
        padding: 48px;
    }

    .login-shell {
        display: grid;
        width: min(100%, 880px);
        grid-template-columns: 1fr 420px;
        align-items: center;
        gap: 72px;
    }

    .login-brand h1 {
        font-size: 68px;
    }

    .brand-mark {
        width: 78px;
        height: 78px;
    }

    .tagline {
        max-width: 320px;
        font-size: 16px;
    }

    .login-card {
        padding: 32px;
    }
}

/* Casilla de mayoría de edad y botón mientras se envía. */
.check {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    color: var(--tinta-suave);
    font-size: 13px;
    line-height: 1.45;
    cursor: pointer;
}

.check input {
    width: 18px;
    height: 18px;
    margin: 1px 0 0;
    accent-color: var(--cobre);
    flex: none;
}

.submit:disabled {
    opacity: 0.65;
    cursor: progress;
}

.switch-modo {
    margin: 0;
    color: var(--tinta-suave);
    font-size: 13px;
    text-align: center;
}

@media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
        animation-duration: 0.01ms !important;
        animation-iteration-count: 1 !important;
        scroll-behavior: auto !important;
        transition-duration: 0.01ms !important;
    }
}
"#;

/// Qué hace el formulario: entrar con una cuenta o crear una nueva.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modo {
    /// Acceso con una cuenta existente.
    Entrar,
    /// Alta de una cuenta nueva.
    Crear,
}

/// Texto del error del servidor, sin los prefijos que añade el transporte.
fn mensaje_de_error(causa: ServerFnError) -> String {
    let texto = causa.to_string();
    let texto = texto
        .strip_prefix("error running server function: ")
        .unwrap_or(&texto);
    let texto = texto.strip_suffix(" (details: None)").unwrap_or(texto);

    texto.to_string()
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

        if email.is_empty() || !email.contains('@') {
            error.set(Some("Escribe un correo válido.".into()));
            return;
        }
        if clave.len() < 6 {
            error.set(Some("La contraseña necesita al menos 6 caracteres.".into()));
            return;
        }
        if es_alta && nombre.len() < 3 {
            error.set(Some("Escribe un alias de 3 caracteres o más.".into()));
            return;
        }
        if es_alta && !es_adulto {
            error.set(Some("Necesitamos que declares que eres mayor de edad.".into()));
            return;
        }

        error.set(None);
        enviando.set(true);

        spawn(async move {
            let resultado = if es_alta {
                api::auth::registrar(api::auth::Registro {
                    correo: email,
                    contrasena: clave,
                    alias: nombre,
                    mayor_de_edad: es_adulto,
                })
                .await
            } else {
                api::auth::entrar(api::auth::Credenciales {
                    correo: email,
                    contrasena: clave,
                })
                .await
            };

            enviando.set(false);

            match resultado {
                Ok(iniciada) => {
                    sesion.set(Some(crate::sesion::SesionApp {
                        token: iniciada.token,
                        perfil: iniciada.perfil,
                    }));
                    on_entrar.call(());
                }
                Err(causa) => error.set(Some(mensaje_de_error(causa))),
            }
        });
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=DM+Serif+Display&family=Inter:wght@400;500;600;700&display=swap",
        }
        style { {CSS} }

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

/// Logo del vaso con capas, dibujado inline para no depender de assets.
#[component]
fn Marca() -> Element {
    rsx! {
        svg {
            class: "brand-mark",
            view_box: "0 0 64 64",
            role: "img",
            "aria-label": "Vaso con capas de bebida",
            path {
                d: "M18 9 H46 L43.4 50.5 Q43 56 37.5 56 H26.5 Q21 56 20.6 50.5 Z",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.6",
                stroke_linejoin: "round",
            }
            path {
                d: "M24.3 43.5 H39.7 L38.9 50.2 Q38.7 53.5 35.7 53.5 H28.3 Q25.3 53.5 25.1 50.2 Z",
                fill: "#B9633B",
            }
            path {
                d: "M22.8 31 H41.2 L40.2 43.5 H23.8 Z",
                fill: "#D8A44C",
            }
            path {
                d: "M21.3 20 H42.7 L41.7 31 H22.3 Z",
                fill: "#6E4A32",
            }
            path {
                d: "M27 62 H37",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.6",
                stroke_linecap: "round",
            }
        }
    }
}
