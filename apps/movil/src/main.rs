// LIBRERÍAS
use dioxus::prelude::*; // Macros rsx! y component, hooks y elementos HTML (única importación con comodín permitida)
use api::EstadoSalud; // Estado del backend que devuelve la server function salud

// CONSTANTES Y VARIABLES GLOBALES
const CSS_LOGIN: Asset = asset!("/assets/login.css"); // Hoja de estilos en blanco y negro de la pantalla de inicio de sesión
const SERVIDOR: &str = match option_env!("RATIO_SERVIDOR") { Some(url) => url, None => "http://127.0.0.1:8080" }; // Backend al que se piden las server functions (Render en producción)

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

/// Prueba de salud completa: la app pregunta al backend (Render) y el backend a Supabase.
#[component]
fn PruebaSalud() -> Element
{
    let mut resultado = use_signal(|| None::<Result<EstadoSalud, ServerFnError>>); // Última respuesta recibida
    let mut cargando = use_signal(|| false); // Petición en curso

    rsx!
    {
        section
        {
            class: "salud",

            // Lanzar la prueba contra el backend
            button
            {
                r#type: "button",
                disabled: *cargando.read(),
                onclick: move |_|
                {
                    cargando.set(true);
                    spawn(async move
                    {
                        resultado.set(Some(api::salud().await));
                        cargando.set(false);
                    });
                },
                if *cargando.read() { "Comprobando…" } else { "Probar conexión" }
            }

            // Mostrar el resultado de la última prueba
            match &*resultado.read()
            {
                None => rsx! {},
                Some(Ok(estado)) if estado.base_de_datos => rsx! { p { "Backend y base de datos: conectados" } },
                Some(Ok(estado)) => rsx! { p { "Backend responde, base de datos no: {estado.detalle}" } },
                Some(Err(causa)) => rsx! { p { "Sin respuesta del backend: {causa}" } },
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

        // Prueba de conexión hasta abajo de la pantalla
        PruebaSalud {}
    }
}

// PUNTO DE PARTIDA
fn main()
{
    dioxus::fullstack::set_server_url(SERVIDOR); // Fijar el backend de las server functions
    dioxus::launch(App); // Lanzar la app
}
