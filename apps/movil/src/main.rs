// LIBRERÍAS
use dioxus::prelude::*; // Macros rsx! y component, hooks y elementos HTML (única importación con comodín permitida)

// CONSTANTES Y VARIABLES GLOBALES
const CSS_LOGIN: Asset = asset!("/assets/login.css"); // Hoja de estilos en blanco y negro de la pantalla de inicio de sesión

// FUNCIONES
// Valida los datos del formulario de inicio de sesión y devuelve el mensaje del primer error
fn validar_login(correo_val: &str, contrasena_val: &str) -> Option<&'static str>
{
    // Rechazar el correo si no tiene arroba
    if !correo_val.contains('@')
    {
        // Correo inválido: pedir uno con arroba
        return Some("Escribe un correo válido"); // Terminar con error de correo
    }

    // Rechazar la contraseña si está vacía
    if contrasena_val.is_empty()
    {
        // Contraseña vacía: pedir que la escriba
        return Some("Escribe tu contraseña"); // Terminar con error de contraseña
    }

    None // Retornar sin errores
}

// Muestra el formulario de inicio de sesión con correo y contraseña
#[component]
fn PantallaLogin() -> Element
{
    let mut correo_val = use_signal(String::new); // Correo escrito por el usuario
    let mut contrasena_val = use_signal(String::new); // Contraseña escrita por el usuario
    let mut mensaje_val = use_signal(String::new); // Mensaje de error o de estado bajo el botón

    rsx!
    {
        document::Stylesheet { href: CSS_LOGIN }

        // Contenedor centrado de la pantalla
        main
        {
            class: "login",

            h1 { "Ratio" }
            p { class: "subtitulo", "Inicia sesión para continuar" }

            // Formulario que se envía con el botón o con Enter
            form
            {
                onsubmit: move |evento_val|
                {
                    evento_val.prevent_default(); // Evitar que el formulario recargue la vista

                    // Elegir mensaje según la validación de los datos
                    match validar_login(&correo_val.read(), &contrasena_val.read())
                    {
                        Some(error_val) =>
                        {
                            // Datos inválidos: mostrar el error
                            mensaje_val.set(error_val.to_string());
                        }
                        None =>
                        {
                            // Datos válidos: avisar que falta la conexión con Supabase Auth (siguiente paso)
                            mensaje_val.set("Datos completos (falta conectar con Supabase Auth)".to_string());
                        }
                    }
                },

                // Campo de correo
                label { r#for: "correo", "Correo" }
                input
                {
                    id: "correo",
                    r#type: "email",
                    autocomplete: "email",
                    value: "{correo_val}",
                    oninput: move |evento_val| correo_val.set(evento_val.value()),
                }

                // Campo de contraseña
                label { r#for: "contrasena", "Contraseña" }
                input
                {
                    id: "contrasena",
                    r#type: "password",
                    autocomplete: "current-password",
                    value: "{contrasena_val}",
                    oninput: move |evento_val| contrasena_val.set(evento_val.value()),
                }

                button { r#type: "submit", "Entrar" }
            }

            // Mostrar el mensaje solo cuando hay uno
            if !mensaje_val.read().is_empty()
            {
                p { class: "mensaje", role: "alert", "{mensaje_val}" }
            }
        }
    }
}

// Componente raíz que monta la aplicación
#[component]
fn App() -> Element
{
    rsx!
    {
        // Montar el inicio de sesión como pantalla inicial
        PantallaLogin {}
    }
}

// PUNTO DE PARTIDA
fn main()
{
    dioxus::launch(App); // Lanzar la app
}
