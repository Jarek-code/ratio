//! Textos en español.
//!
//! Un brazo por texto del catálogo: si falta uno, el compilador lo dice, así que añadir un
//! texto nuevo en `textos.rs` obliga a traducirlo en los tres idiomas.

use super::Texto;

/// Nombres cortos de los meses, de enero a diciembre.
pub(super) const MESES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

/// Fecha completa, con el orden y los separadores de este idioma.
pub(super) fn fecha(dia: u32, mes: u32, anio: i32) -> String {
    let nombre = MESES[(mes.clamp(1, 12) - 1) as usize];

    // por ejemplo, 12 oct 2026
    format!("{dia} {nombre} {anio}")
}

/// Texto en este idioma.
pub(super) fn texto(texto: Texto) -> &'static str {
    match texto {
        Texto::ComentoTuReceta => "comentó tu receta",
        Texto::RespondioTuComentario => "respondió tu comentario en",
        Texto::PreparoTuReceta => "preparó tu receta",
        Texto::PublicoUnaVersion => "publicó una versión:",
        Texto::RevisoUnReporte => "revisó un reporte sobre",
        Texto::Nueva => "Nueva",
        Texto::CargandoActividad => "Cargando tu actividad…",
        Texto::ActividadSinSesion => "Entra para ver quién comenta y prepara tus recetas.",
        Texto::Entrar => "Entrar",
        Texto::ActividadVacia => {
            "Todavía no hay novedades. Publica una receta en La Barra para empezar."
        }
        Texto::MasFuerte => "Más fuerte",
        Texto::MasDebil => "Más débil",
        Texto::AtajosDeProporcion => "Atajos de proporción",
        Texto::SeAplicanATodas => "Se aplican a todas las capas",
        Texto::Escala => "Escala",
        Texto::VolumenDeLaBebida => "Volumen de la bebida en mililitros",
        Texto::EscalaNota => "La escala termina en 500 ml y mueve todas las capas.",
        Texto::NuevosIngredientes => "Nuevos ingredientes",
        Texto::LiquidosOSolidos => "Líquidos o sólidos · 1 g = 1 ml",
        Texto::IngredienteDelCatalogo => "Ingrediente del catálogo",
        Texto::EligeIngrediente => "Elige un ingrediente (ml o mg)",
        Texto::SinMasIngredientes => "No hay más ingredientes disponibles en el catálogo.",
        Texto::Quitar => "Quitar",
        Texto::NombreDeLaVersion => "Nombre de la versión",
        Texto::Restablecer => "Restablecer",
        Texto::IniciaSesionGuardarVersion => "Inicia sesión para guardar una versión.",
        Texto::VersionGuardada => "Versión guardada. Ya aparece en Mi recetario.",
        Texto::VersionSinConexion => {
            "Sin conexión: la versión se enviará cuando vuelva la red y aparecerá en Mi recetario."
        }
        Texto::Guardando => "Guardando…",
        Texto::GuardarVersionPrivada => "Guardar versión privada",
        Texto::AbrirVersion => "Abrir versión",
        Texto::CargandoProporciones => "Cargando las proporciones…",
        Texto::RecetaNoPublicada => "Esta receta no existe o ya no está publicada.",
        Texto::NoHayVasos => "No hay vasos disponibles para ajustar la receta.",
        Texto::AjustarProporciones => "Ajustar proporciones",
        Texto::Total => "Total",
        Texto::DelVaso => "Del vaso",
        Texto::SinAlcohol => "Sin alcohol",
        Texto::IntensidadBase => "Intensidad base",
        Texto::VasoAlLimite => {
            "El vaso queda al límite: el dibujo se corta al 94 % de la capacidad."
        }
        Texto::TodasLasCapasACero => {
            "Todas las capas están a cero. Sube alguna para ver la bebida."
        }
        Texto::Vaso => "Vaso",
        Texto::ElegirElVaso => "Elegir el vaso",
        Texto::BienvenidaTitulo1 => "La proporción se ve en el vaso",
        Texto::BienvenidaTexto1 => {
            "Cada receta se dibuja como capas dentro de su vaso real, con su porcentaje y sus mililitros."
        }
        Texto::BienvenidaTitulo2 => "Ajusta y haz tu versión",
        Texto::BienvenidaTexto2 => {
            "Cambia cantidades o vaso; guarda tu versión y publícala en La Barra con crédito al original."
        }
        Texto::BienvenidaTitulo3 => "Contenido con alcohol",
        Texto::BienvenidaTexto3 => {
            "Los cócteles solo se muestran si al crear tu cuenta declaras que eres mayor de edad. Bebe con responsabilidad."
        }
        Texto::BienvenidaPaso => "Bienvenida · {} de {}",
        Texto::BienvenidaAvisoEdad => {
            "Ratio no vende alcohol ni verifica tu edad: confía en tu declaración."
        }
        Texto::Atras => "Atrás",
        Texto::Empezar => "Empezar",
        Texto::Siguiente => "Siguiente",
        Texto::VolverPerfil => "Volver al perfil",
        Texto::DestinoPerfil => "Perfil",
        Texto::Ajustes => "Ajustes",
        Texto::Unidades => "Unidades",
        Texto::UnidadesNota => {
            "Cómo se muestran las cantidades en la ficha y en las equivalencias."
        }
        Texto::Tema => "Tema",
        Texto::TemaNota => {
            "El tema oscuro cambia el fondo y el texto de las pantallas del recetario."
        }
        Texto::IdiomaSeccion => "Idioma",
        Texto::IdiomaNota => "Cambia el idioma de toda la aplicación.",
        Texto::BienvenidaSeccion => "Bienvenida",
        Texto::VolverBienvenida => "Volver a ver la bienvenida al salir",
        Texto::Mililitros => "Mililitros (ml)",
        Texto::Onzas => "Onzas (oz)",
        Texto::Partes => "Partes",
        Texto::Claro => "Claro",
        Texto::Oscuro => "Oscuro",
        Texto::DestinoDescubrir => "Descubrir",
        Texto::DestinoBiblioteca => "Biblioteca",
        Texto::DestinoRecetario => "Recetario",
        Texto::DestinoLaBarra => "La Barra",
        Texto::NavAria => "Navegación principal",
        Texto::NovedadesSinLeer => "{} novedades sin leer",
        Texto::OrdenNombre => "Nombre",
        Texto::OrdenIntensidad => "Intensidad",
        Texto::OrdenDificultad => "Dificultad",
        Texto::OrdenTiempo => "Tiempo",
        Texto::ProporcionEnLaFicha => "Todas las recetas publicadas, con su proporción a la vista.",
        Texto::ListaDeCompra => "Lista de compra",
        Texto::CargandoCatalogo => "Cargando el catálogo…",
        Texto::SinCoincidencias => {
            "No hay recetas que coincidan. Prueba con otra palabra o quita el filtro."
        }
        Texto::CuentaRecetas => "{} recetas",
        Texto::BuscarPorNombreCorto => "Buscar por nombre",
        Texto::FiltrarPorFamilia => "Filtrar por familia",
        Texto::TodasLasFamilias => "Todas",
        Texto::Cocteles => "Cócteles",
        Texto::OrdenarLaLista => "Ordenar la lista",
        Texto::OrdenarTitulo => "Ordenar",
        Texto::DificultadNivel => "Dificultad {}",
        Texto::ConLeche => "Con leche",
        Texto::SinAlcoholTexto => "Sin alcohol",
        Texto::Igual => "— igual",
        Texto::Graduacion => "Graduación",
        Texto::Volumen => "Volumen",
        Texto::Capas => "Capas",
        Texto::VolverALaFicha => "Volver a la ficha",
        Texto::CompararSubtitulo => {
            "Pon dos recetas frente a frente: el vaso, el volumen y la graduación."
        }
        Texto::CargandoRecetaA => "Cargando la receta A…",
        Texto::RecetaNoDisponible => "Esta receta no existe o ya no está disponible.",
        Texto::CargandoRecetaB => "Cargando la receta B…",
        Texto::EligeRecetaB => "Elige la receta B para ver las dos tazas y sus diferencias.",
        Texto::EligeUnaReceta => "Elige una receta…",
        Texto::SinCatalogoComparar => "Sin el catálogo no se pueden cambiar las recetas.",
        Texto::PartesUnidad => "Partes",
        Texto::DiferenciasAB => "Diferencias · A frente a B",
        Texto::MedidasDelMotor => "Medidas del motor de proporciones, con las cantidades en {}.",
        Texto::Medida => "Medida",
        Texto::IngredientesOtrasUnidades => "Ingredientes en otras unidades",
        Texto::VolverALaReceta => "Volver a la receta",
        Texto::SinAlcoholCorto => "Sin alcohol",
        Texto::IngredientesYEquivalencias => "Ingredientes y equivalencias",
        Texto::CargandoReceta => "Cargando la receta…",
        Texto::NotaEquivalencias => {
            "Unidad preferida: {}. Las partes se cuentan contra el ingrediente más pequeño que no es hielo ni agua; 1 cucharada = 15 ml."
        }
        Texto::Ingrediente => "Ingrediente",
        Texto::Cucharadas => "Cdas",
        Texto::CambiarUnidadPreferida => "Cambiar la unidad preferida",
        Texto::PerfilDe => "Perfil de {}",
        Texto::MiPerfil => "Mi perfil",
        Texto::MiRecetario => "Mi recetario",
        Texto::Guardados => "Guardados",
        Texto::Actividad => "Actividad",
        Texto::Salir => "Salir",
        Texto::EntrarACuenta => "Entrar a tu cuenta",
        Texto::BuscarRecetas => "Buscar recetas",
        Texto::BuscarPorNombre => "Buscar por nombre de bebida",
        Texto::Buscar => "Buscar",
        Texto::FamiliasDeBebida => "Familias de bebida",
        Texto::Espresso => "Espresso",
        Texto::VerReceta => "Ver receta",
        Texto::SugerenciaDelDia => "Sugerencia del día",
        Texto::BuscandoSugerencia => "Buscando la sugerencia de hoy…",
        Texto::SinRecetasPublicadas => "Todavía no hay recetas publicadas. Vuelve pronto.",
        Texto::SugerenciaAnterior => "Sugerencia anterior",
        Texto::SugerenciaSiguiente => "Sugerencia siguiente",
        Texto::SigueDondeLoDejaste => "Sigue donde lo dejaste",
        Texto::AunNoHayVistas => {
            "Aún no has abierto ninguna receta. Empieza por la sugerencia de hoy."
        }
        Texto::QuePreparasHoy => "¿Qué vas a preparar hoy?",
        Texto::RecetaNoPublicadaFicha => "Esta receta no existe o ya no está publicada.",
        Texto::VersionDe => "Versión de ",
        Texto::Capacidad => "Capacidad",
        Texto::ProporcionesEnElVaso => "Proporciones en el vaso",
        Texto::Acciones => "Acciones",
        Texto::Preparar => "Preparar",
        Texto::AjustarAria => "Ajustar las proporciones o derivar una versión propia",
        Texto::AjustarODerivar => "Ajustar o derivar",
        Texto::CompararBoton => "Comparar",
        Texto::Ingredientes => "Ingredientes",
        Texto::Reportar => "Reportar",
        Texto::IniciaSesionGuardar => "Inicia sesión para guardar la receta.",
        Texto::SinConexionSeGuardara => "Sin conexión: se guardará cuando vuelva la red.",
        Texto::Guardado => "Guardado",
        Texto::Guardar => "Guardar",
        Texto::Preparacion => "Preparación",
        Texto::NotasDeSabor => "Notas de sabor",
        Texto::CadaVersionDeriva => {
            "Cada versión deriva de otra y conserva el crédito a su original."
        }
        Texto::VerHistorialDeVersiones => "Ver el historial de versiones",
        Texto::VuelveAEntrarPublicar => "Vuelve a entrar para publicar.",
        Texto::SinConexionPublicacion => "Sin conexión: se aplicará cuando vuelva la red.",
        Texto::UnMomento => "Un momento…",
        Texto::RetirarDeLaBarra => "Retirar de La Barra",
        Texto::PublicarEnLaBarra => "Publicar en La Barra",
        Texto::SinConexionValoracion => {
            "Sin conexión: tu valoración se enviará cuando vuelva la red."
        }
        Texto::TeSalioEstaReceta => "¿Te salió esta receta?",
        Texto::MeSalio => "Me salió",
        Texto::NoMeSalio => "No me salió",
        Texto::CuantoTeGusto => "Aparte, ¿cuánto te gustó? De 1, nada, a 5, mucho",
        Texto::TeGustoOpcional => "Aparte: ¿te gustó? (opcional)",
        Texto::GustoNoCambia => {
            "Tu gusto se guarda en tu diario y no cambia el porcentaje de «me salió»."
        }
        Texto::EsTuReceta => "Es tu receta: quienes la preparen dirán si les salió.",
        Texto::VuelveAEntrarFoto => "Vuelve a entrar para adjuntar una foto.",
        Texto::FotoNoPreparada => "No se pudo preparar la foto en este dispositivo.",
        Texto::EscribeTuComentario => "Escribe tu comentario antes de publicarlo.",
        Texto::VuelveAEntrarComentar => "Vuelve a entrar para comentar.",
        Texto::PropuestaInvalida => {
            "La propuesta debe ser un número mayor que cero y necesita una capa o un paso."
        }
        Texto::SinConexionComentario => {
            "Sin conexión: tu comentario se publicará cuando vuelva la red."
        }
        Texto::Cancelar => "Cancelar",
        Texto::TuComentario => "Tu comentario",
        Texto::QueCambiarias => "¿Qué cambiarías o qué te funcionó?",
        Texto::Sobre => "Sobre",
        Texto::TodaLaReceta => "Toda la receta",
        Texto::FotoQueVasAAjuntar => "Foto que vas a adjuntar",
        Texto::QuitarFoto => "Quitar foto",
        Texto::AdjuntarFoto => "Adjuntar foto",
        Texto::Publicando => "Publicando…",
        Texto::Responder => "Responder",
        Texto::Comentar => "Comentar",
        Texto::VuelveAEntrarReportar => "Vuelve a entrar para reportar.",
        Texto::GraciasModeracion => "Gracias: el equipo de moderación lo revisará.",
        Texto::YaHabiasReportado => {
            "Ya habías reportado esto: el equipo de moderación lo está revisando."
        }
        Texto::SinConexionReporte => "Sin conexión: el reporte se enviará cuando vuelva la red.",
        Texto::Motivo => "Motivo",
        Texto::Detalle => "Detalle",
        Texto::CuentanosQuePasaOpcional => "Cuéntanos qué pasa (opcional)",
        Texto::EnviarReporte => "Enviar reporte",
        Texto::LaBarra => "La Barra",
        Texto::CargandoComentarios => "Cargando comentarios…",
        Texto::SinComentariosAun => "Nadie ha comentado todavía. ¿Cómo te salió?",
        Texto::VerComentarioYResponder => "Ver el comentario y responder",
        Texto::ComentariosPendientes => "Comentarios pendientes de enviar",
        Texto::PendienteDeEnviar => " · Pendiente de enviar",
        Texto::EntraParaComentar => "Entra para comentar y valorar.",
        Texto::FotoSinStorage => {
            "Esta compilación no tiene Storage configurado: no se pueden subir fotos."
        }
        Texto::FotoAppSinConfigurar => "Esta versión de la app no tiene configuradas las fotos.",
        Texto::FotoTipoInvalido => "Elige una foto JPG, PNG o WebP.",
        Texto::FotoPesaDemasiado => "La foto pesa más de 5 MB.",
        Texto::FotoSinNombre => "No se pudo nombrar la foto.",
        Texto::FotoSinPreparar => "No se pudo preparar la subida de la foto.",
        Texto::FotoSinConexion => "Sin conexión: intenta subir la foto cuando vuelva la red.",
        Texto::FotoRechazada => "Storage rechazó la foto.",
        Texto::FotoNoLeida => "No se pudo leer la foto.",
        Texto::SubiendoFoto => "Subiendo…",
        Texto::FotoSinSesion => "Vuelve a entrar para subir fotos.",
        Texto::Fotos => "Fotos",
        Texto::AgregarFoto => "Agregar foto",
        Texto::TuLista => "Tu lista",
        Texto::GuardadosSub => "Las recetas que quieres volver a preparar.",
        Texto::CargandoGuardados => "Cargando tus guardados…",
        Texto::VolverADescubrir => "Volver a Descubrir",
        Texto::GuardadosSinSesion => "Inicia sesión para ver las recetas que guardaste.",
        Texto::GuardadosVacio => "Todavía no has guardado recetas. Abre una ficha y usa «Guardar».",
        Texto::IrALaBiblioteca => "Ir a la biblioteca",
        Texto::NRecetasGuardadas => "1 receta guardada",
        Texto::SesionCaducada => "La sesión caducó; vuelve a entrar.",
        Texto::SinConexionSeQuitara => "Sin conexión: se quitará cuando vuelva la red.",
        Texto::Quitando => "Quitando…",
        Texto::TodasFiltro => "Todas",
        Texto::Publicas => "Públicas",
        Texto::Privadas => "Privadas",
        Texto::Publica => "Pública",
        Texto::PrivadaFiltro => "Privada",
        Texto::VolverALaFichaReceta => "Volver a la ficha de la receta",
        Texto::HistorialDeVersiones => "Historial de versiones",
        Texto::CargandoHistorial => "Cargando el historial…",
        Texto::SinVersionesBebida => "Esta bebida todavía no tiene versiones publicadas.",
        Texto::RecetaOriginal => "Receta original",
        Texto::VersionUna => "Versión",
        Texto::Versiones => "Versiones",
        Texto::FiltrarVersiones => "Filtrar versiones",
        Texto::LineaDeTiempoVersiones => "Línea de tiempo de versiones",
        Texto::TocaAbrir => "Toca «Abrir» para ver la ficha de esa versión.",
        Texto::DatosDeLaBase => "Datos de la base de Ratio",
        Texto::Derivada => "Derivada",
        Texto::DerivaNoDisponible => "Deriva de una versión que no está disponible",
        Texto::SinCambiosFrenteAlOrigen => "Sin cambios frente a su origen",
        Texto::RecetaOriginalSinCambios => "Receta original · sin cambios",
        Texto::Abrir => "Abrir →",
        Texto::Recientes => "Recientes",
        Texto::Siguiendo => "Siguiendo",
        Texto::Populares => "Populares",
        Texto::Conversacion => "Conversación",
        Texto::CocteleriaSeccion => "Coctelería",
        Texto::CafeSeccion => "Café",
        Texto::CargandoMas => "Cargando más…",
        Texto::CargarMas => "Cargar más",
        Texto::SinValoraciones => "Sin valoraciones",
        Texto::Original => "Original",
        Texto::ComentarioUno => "Comentario",
        Texto::ComentarioVarios => "Comentarios",
        Texto::VersionVarias => "Versiones",
        Texto::BarraSinSesion => "Entra para ver lo que publican las cuentas que sigues.",
        Texto::SigueAAlguien => "Sigue a alguien desde su perfil para ver aquí lo que publica.",
        Texto::SinActividadReciente => "Todavía no hay recetas con actividad reciente.",
        Texto::BarraSinRecetas => {
            "Todavía no hay recetas publicadas. Publica la tuya desde Mi recetario."
        }
        Texto::CargandoLaBarra => "Cargando La Barra…",
        Texto::BarraSinComentarios => {
            "Nadie ha comentado todavía. Abre una receta y cuenta cómo te salió."
        }
        Texto::RespondioEn => "respondió · ",
        Texto::ComentoEn => "comentó · ",
        Texto::CargandoConversacion => "Cargando la conversación…",
        Texto::Comunidad => "Comunidad",
        Texto::SeccionesDeLaBarra => "Secciones de La Barra",
        Texto::VolverALaBarra => "Volver a La Barra",
        Texto::VolverALaBarraBoton => "‹ La Barra",
        Texto::VerLaReceta => "Ver la receta",
        Texto::CargandoHilo => "Cargando el hilo…",
        Texto::RecetaNoDisponibleBarra => "Esta receta no existe o ya no está disponible.",
        Texto::Hilo => "Hilo",
        Texto::PorTexto => "por ",
        Texto::CafeYEspecias => "Café y especias",
        Texto::Lacteos => "Lácteos",
        Texto::IngredienteCafe => "café",
        Texto::IngredienteTe => "té",
        Texto::IngredienteConac => "coñac",
        Texto::IngredienteAzucar => "azúcar",
        Texto::CargandoRecetarioCorto => "Cargando el recetario…",
        Texto::DesmarcarTodo => "Desmarcar todo",
        Texto::Comensales => "Comensales",
        Texto::QuitarComensal => "Quitar un comensal",
        Texto::AgregarComensal => "Agregar un comensal",
        Texto::Vaciar => "Vaciar",
        Texto::EligeRecetasParaLista => "Elige una o más recetas para empezar.",
        Texto::AgregarReceta => "Agregar receta",
        Texto::BuscarRecetasParaLista => "Buscar recetas para la lista",
        Texto::ListaApareceraAqui => "La lista aparecerá aquí en cuanto elijas una receta.",
        Texto::RecetasSinIngredientes => "Estas recetas no tienen ingredientes que comprar.",
        Texto::CambioPendiente => {
            "1 cambio pendiente de enviar: se enviará cuando vuelva la conexión."
        }
        Texto::FaltaCorreo => "Escribe tu correo.",
        Texto::CorreoConEspacios => "El correo no lleva espacios.",
        Texto::CorreoInvalido => "Escribe un correo válido, con arroba y dominio.",
        Texto::FaltaContrasena => "Escribe tu contraseña.",
        Texto::ContrasenaCorta => "La contraseña necesita al menos {} caracteres.",
        Texto::FaltaAlias => "Escribe un alias.",
        Texto::AliasCorto => "El alias necesita {} caracteres o más.",
        Texto::AliasLargo => "El alias no puede pasar de {} caracteres.",
        Texto::AliasConArroba => "El alias no lleva arroba: es el nombre que verán los demás.",
        Texto::CrearCuenta => "Crear cuenta",
        Texto::BajadaCrear => {
            "Tu alias y tu edad quedan en el perfil; la contraseña la guarda Supabase cifrada."
        }
        Texto::BajadaEntrar => "Usa tu correo para volver a tu recetario.",
        Texto::FaltaEdad => "Necesitamos que declares que eres mayor de edad.",
        Texto::MarcaRubro => "Café · Coctelería",
        Texto::MarcaLema => "La proporción se ve en el vaso.",
        Texto::Correo => "Correo",
        Texto::PlaceholderCorreo => "tu@correo.com",
        Texto::Contrasena => "Contraseña",
        Texto::PlaceholderContrasenaCrear => "Al menos 6 caracteres",
        Texto::PlaceholderContrasenaEntrar => "Tu contraseña",
        Texto::OcultarContrasena => "Ocultar",
        Texto::MostrarContrasena => "Ver",
        Texto::Alias => "Alias",
        Texto::PlaceholderAlias => "Como te verán en La Barra",
        Texto::DeclaracionEdad => "Tengo 18 años o más y quiero ver recetas con alcohol.",
        Texto::CorreoEnviado => {
            "Te enviamos un correo con el enlace para cambiar la contraseña. Revisa también el spam."
        }
        Texto::MandandoCorreo => "Mandando el correo…",
        Texto::OlvidasteContrasena => "¿Olvidaste tu contraseña?",
        Texto::YaTienesCuenta => "¿Ya tienes cuenta? ",
        Texto::NoTienesCuenta => "¿No tienes cuenta? ",
        Texto::AvisoLegal => "Contenido con alcohol solo para personas mayores de edad.",
        Texto::PieAcceso => "Ratio · Proporción visible, recetas reproducibles.",
        Texto::VasoConCapas => "Vaso con capas de bebida",
        Texto::TodoFiltro => "Todo",
        Texto::VersionesFiltro => "Versiones",
        Texto::Publicadas => "Publicadas",
        Texto::Borradores => "Borradores",
        Texto::SinRecetasPropias => {
            "Todavía no tienes recetas propias. Empieza por ajustar una de la biblioteca."
        }
        Texto::SinVersionesPropias => "Todavía no has guardado ninguna versión propia.",
        Texto::SinPublicaciones => "Todavía no has publicado una receta.",
        Texto::SinBorradores => "No tienes borradores pendientes.",
        Texto::Borrador => "Borrador",
        Texto::Privada => "Privada",
        Texto::Publicada => "Publicada",
        Texto::Oculta => "Oculta",
        Texto::Recetario => "Recetario",
        Texto::RecetarioSubtitulo => {
            "Tus versiones, tus publicaciones y lo que aún no sale de casa."
        }
        Texto::CargandoRecetario => "Cargando tu recetario…",
        Texto::RecetarioSinSesion => {
            "Inicia sesión para ver tus versiones, tus publicaciones y tus borradores."
        }
        Texto::FiltrarRecetario => "Filtrar mi recetario",
        Texto::CreadaEl => "Creada {}",
        Texto::VuelveAEntrarModerar => "Vuelve a entrar para moderar.",
        Texto::NotaAviso => "Escribe la nota del aviso: es lo que leerá quien publicó.",
        Texto::DeTexto => " de ",
        Texto::ReportadoUnaVez => "Reportado 1 vez",
        Texto::NotaObligatoria => "Nota (obligatoria para avisar)",
        Texto::QueDecidiste => "Qué decidiste y por qué",
        Texto::DecisionSobreContenido => "Decisión sobre este contenido",
        Texto::Avisar => "Avisar",
        Texto::DescartarReportes => "Descartar reportes",
        Texto::ContenidoYaNoExiste => "El contenido ya no existe.",
        Texto::Restaurar => "Restaurar",
        Texto::CargandoColaReportes => "Cargando la cola de reportes…",
        Texto::SinReportesPendientes => "No hay reportes pendientes.",
        Texto::SinDecisiones => "Todavía no hay decisiones registradas.",
        Texto::VolverATuPerfil => "Volver a tu perfil",
        Texto::ModeracionSeccion => "Moderación",
        Texto::ModeracionSinPermiso => {
            "Entra con una cuenta de moderación para revisar los reportes."
        }
        Texto::SeccionesDeModeracion => "Secciones de moderación",
        Texto::Cola => "Cola",
        Texto::HistorialSeccion => "Historial",
        Texto::TuCuenta => "Tu cuenta",
        Texto::AplicarSinConexion => "Sin conexión: se aplicará cuando vuelva la red.",
        Texto::Seguir => "Seguir",
        Texto::MiDiario => "Mi diario",
        Texto::CargandoDiario => "Cargando tu diario…",
        Texto::DiarioVacioAjeno => {
            "Todavía no has preparado recetas de otras personas. Abre una y di si te salió."
        }
        Texto::TuPerfil => "Tu perfil",
        Texto::ResumenPerfil => "{} recetas publicadas · {} seguidores · {} siguiendo",
        Texto::RecetasPublicadas => "Recetas publicadas",
        Texto::RecetasPublicadasVacio => "Todavía no hay recetas publicadas.",
        Texto::CargandoPerfil => "Cargando el perfil…",
        Texto::PerfilSinSesion => "Entra para ver tu perfil, tus recetas y a quién sigues.",
        Texto::PerfilNoExiste => "Este perfil no existe.",
        Texto::SalirDeLaPreparacion => "Salir de la preparación",
        Texto::RecetaSinPasos => "Esta receta todavía no tiene pasos de preparación.",
        Texto::PasoDeTotal => "Paso {} de {}",
        Texto::Tiempo => "¡Tiempo!",
        Texto::Pausar => "Pausar",
        Texto::IniciarTemporizador => "Iniciar temporizador",
        Texto::Anterior => "Anterior",
        Texto::Terminar => "Terminar",
        Texto::SiguientePaso => "Siguiente paso",
        Texto::Listo => "¡Listo!",
        Texto::TeSalioIgual => "¿Te salió igual que en la ficha?",
        Texto::PrepararOtraVez => "Preparar otra vez",
        Texto::RedTiempo => {
            "El servidor no respondió en {} s. Se intentó {} veces: revisa tu conexión y vuelve a probar."
        }
        Texto::RedConexion => {
            "No se pudo abrir la conexión con el servidor ({} s de espera). Revisa tu red o los datos móviles."
        }
        Texto::RedEstado => {
            "El servidor respondió con un error {}. Vuelve a intentarlo en un momento."
        }
        Texto::RedSinConexion => {
            "Sin conexión con el servidor. Revisa tu red o los datos móviles y vuelve a intentarlo."
        }
        Texto::RedServidor => "El servidor no pudo completar la petición. Vuelve a intentarlo.",
        Texto::RedIncompleta => "La respuesta del servidor llegó incompleta. Vuelve a intentarlo.",
        Texto::RedCortada => "La conexión se cortó a media respuesta. Vuelve a intentarlo.",
        Texto::RedOtra => "No se pudo completar la petición: {}",
        Texto::Reintentar => "Reintentar",
        Texto::Cerrar => "Cerrar",
        Texto::ReportarReceta => "Reportar receta",
        Texto::ReportarExplicacion => {
            "Cuéntanos qué pasa con esta receta. Tu reporte es privado y lo revisa el equipo de moderación."
        }
        Texto::DetalleOpcional => "Detalle (opcional)",
        Texto::AgregaContexto => "Agrega contexto para el equipo de moderación.",
        Texto::SoloModeracion => "Solo lo ve moderación",
        Texto::GraciasRevisaremos => "Gracias. Revisaremos el contenido y tomaremos una decisión.",
        Texto::IniciaSesionParaReportar => "Inicia sesión para reportar este contenido.",
        Texto::ReporteEnviado => "Reporte enviado",
        Texto::Enviando => "Enviando…",
        Texto::Comprobando => "Comprobando…",
        Texto::ProbarConexion => "Probar conexión",
        Texto::BackendConectado => "Backend y base de datos: conectados",
        Texto::BotonPruebaConexion => "El botón de arriba repite la prueba cuando quieras.",
        Texto::FotoSinCarpeta => "El teléfono no tiene carpeta privada donde dejar la foto.",
        Texto::FotoSinElegir => "No se eligió ninguna foto a tiempo.",
        Texto::GaleriaNoAbre => "No se pudo abrir la galería en este momento.",
        Texto::SinGaleria => "Este dispositivo no tiene galería de fotos.",
        Texto::CantidadDe => "Cantidad de {}",
        Texto::LectorMililitros => "{} mililitros, {} por ciento",
        Texto::LectorMiligramos => "{} miligramos, suma {} mililitros, {} por ciento",
        Texto::PorcentajeDeLaReceta => "{} % de la receta",
        Texto::PorcentajeDelTotal => "{} % del total",
        Texto::PorcentajeDelAgregado => "{} % del agregado",
        Texto::NombreVersionPropuesto => "{} · versión",
        Texto::VersionGuardadaConAviso => "Versión guardada. {}",
        Texto::NoSeGuardoVersion => "No pudimos guardar la versión: {}",
        Texto::PorAutor => "por {}",
        Texto::NivelDeCinco => "{} de 5",
        Texto::NoCabeSobran => "No cabe: sobran {} ml. Baja una capa o elige un vaso más grande.",
        Texto::DeTotalCorto => "{} de {}",
        Texto::OriginalDe => "· original de {}",
        Texto::MlEnVaso => "{} ml en {} ml de vaso",
        Texto::NoSePudoGuardar => "No se pudo guardar: {}",
        Texto::NoSePudoCambiarPublicacion => "No se pudo cambiar la publicación: {}",
        Texto::NoSePudoGuardarGusto => "No se pudo guardar tu gusto: {}",
        Texto::NoSePudoGuardarValoracion => "No se pudo guardar tu valoración: {}",
        Texto::RecuentoValoracion => "A {} les salió · a {} no les salió",
        Texto::GustoDeCinco => "Gusto {} de 5",
        Texto::NoSePudoComentar => "No se pudo publicar el comentario: {}",
        Texto::ReportarEste => "Reportar este {}",
        Texto::NoSePudoReportar => "No se pudo enviar el reporte: {}",
        Texto::VerPerfilDe => "Ver el perfil de {}",
        Texto::FotoDe => "Foto de {}",
        Texto::SinLeer => "{} sin leer",
        Texto::VerLosComentarios => "Ver los {} comentarios y responder",
        Texto::FotoNumeroDe => "Foto {} de {}",
        Texto::FotosMaximo => "Hasta {} fotos JPG, PNG o WebP de 5 MB como máximo.",
        Texto::GuardadoEl => "Guardado el {}",
        Texto::UltimaVez => "Última vez: {}",
        Texto::QuitarDeGuardados => "Quitar {} de guardados",
        Texto::NoSePudoQuitar => "No se pudo quitar: {}",
        Texto::PorcentajeMeSalio => "{} % me salió ({})",
        Texto::VersionDeOrigen => "Versión de {}",
        Texto::AbrirHiloDe => "Abrir el hilo de {}",
        Texto::ListosDeTotal => "{} de {} listos",
        Texto::CambiosPendientes => {
            "{} cambios pendientes de enviar: se enviarán cuando vuelva la conexión."
        }
        Texto::DeTotalRecetas => "{} de {} recetas",
        Texto::ReportadoVeces => "Reportado {} veces",
        Texto::NoSePudoRestaurar => "No se pudo restaurar: {}",
        Texto::NoSePudoActualizar => "No se pudo actualizar: {}",
        Texto::TiempoDePreparacion => "Tiempo de preparación: {}",
        Texto::NoSePudoGuardarDecision => "No se pudo guardar la decisión: {}",
        Texto::BackendSinBase => "Backend responde, base de datos no: {}",
        Texto::ProbadoEn => "{} (probado en {})",
        Texto::FotoNoTermino => {
            "La foto no terminó de subir en {} s: revisa tu conexión y vuelve a intentarlo."
        }
        Texto::FotoNoAgregada => "La foto se subió, pero no se pudo agregar a la receta: {}",
        Texto::FotoNoLeidaAndroid => "No se pudo leer la foto elegida.",
        Texto::VersionSinAviso => "Sin conexión: la versión se enviará cuando vuelva la red.",
        Texto::HaceHoy => "hoy",
        Texto::HaceAyer => "ayer",
        Texto::HaceDias => "hace {} días",
        Texto::HaceHoras => "hace {} h",
        Texto::HaceMinutos => "hace {} min",
        Texto::HaceMomentos => "hace un momento",
        Texto::PreparacionSeccion => "Preparación",
        Texto::ListaSubtitulo => {
            "Todo lo que necesitas para tus recetas, sumado en una sola lista."
        }
        Texto::UnidadDeLaLista => "Unidad de la lista",
        Texto::RecetasEnLaLista => "Recetas en la lista",
        Texto::SinCoincidenciasCorto => "No hay recetas que coincidan.",
        Texto::VolverALaBiblioteca => "Volver a la biblioteca",
        Texto::BibliotecaTitulo => "Biblioteca",
        Texto::RecetarioSeccion => "Recetario",
        Texto::VolverDescubrirBoton => "‹ Descubrir",
        Texto::VersionesSeccion => "Versiones",
        Texto::Servido => "Servido",
        Texto::ActualizadaEl => "Actualizada {}",
        Texto::MinutosCorto => "{} min",
        Texto::MililitrosCorto => "{} ml",
        Texto::OnzasCorto => "{} oz",
        Texto::PartesCorto => "{} partes",
        Texto::MotivoSpam => "Spam o publicidad",
        Texto::MotivoOfensivo => "Ofensivo",
        Texto::MotivoPeligroso => "Peligroso",
        Texto::MotivoMenores => "Alcohol para menores",
        Texto::MotivoOtro => "Otro motivo",
        Texto::FamiliaDirecto => "Directo",
        Texto::FamiliaLargo => "Largo",
        Texto::FamiliaCorto => "Corto",
        Texto::FamiliaClasico => "Clásico",
        Texto::FamiliaFrio => "Frío",
        Texto::FamiliaCaliente => "Caliente",
        Texto::FamiliaCremoso => "Cremoso",
        Texto::FamiliaCitrico => "Cítrico",
        Texto::FamiliaTropical => "Tropical",
        Texto::FamiliaConHielo => "Con hielo",
        Texto::DuracionSegundos => "{} s",
        Texto::DuracionMinutos => "{} min",
        Texto::DuracionHoras => "{} h {} min",
        Texto::AnclaCapa => "Capa · {}",
        Texto::AnclaPaso => "Paso {} · {}",
        Texto::ObjetivoReceta => "Receta",
        Texto::ObjetivoComentario => "Comentario",
        Texto::AccionRetirado => "Retirado",
        Texto::AccionRestaurado => "Restaurado",
        Texto::AccionDescartado => "Reportes descartados",
        Texto::AccionAvisoEnviado => "Aviso enviado",
        Texto::LeSalio => "Le salió",
        Texto::NoLeSalio => "No le salió",
        Texto::ModeracionOcultarAutor => "Retiramos tu {} de La Barra porque no cumple las normas.",
        Texto::ModeracionRestaurarAutor => "Restauramos tu {}: vuelve a estar visible.",
        Texto::ModeracionAvisoAutor => "Recibimos un reporte sobre tu {}. Revísalo, por favor.",
        Texto::ModeracionOcultarReporte => "Revisamos tu reporte y retiramos el contenido.",
        Texto::ModeracionAvisoReporte => "Revisamos tu reporte y avisamos a quien lo publicó.",
        Texto::ModeracionDescartado => "Revisamos tu reporte y no encontramos nada que retirar.",
        Texto::ModeracionNota => "Nota de moderación: {}",
        Texto::AvisoAlLimite => "La bebida queda al límite del vaso.",
        Texto::ErrorDatos => "Revisa los datos: algo no tiene el formato correcto.",
        Texto::ErrorSesion => "Tu sesión caducó: vuelve a entrar.",
        Texto::ErrorNoEncontrado => "Eso ya no existe o no está disponible para tu cuenta.",
        Texto::ErrorServidor => "El servidor no pudo completar la petición. Vuelve a intentarlo.",
        Texto::SegundosCorto => "{} s",
        Texto::Propone => "Propone: {}",
        Texto::PropuestaOpcional => "Propuesta (opcional)",
        Texto::SegundosPlaceholder => "segundos",
        Texto::CantidadPlaceholder => "cantidad",
        Texto::ReporteConDetalle => "{}: {} (reportó {})",
        Texto::ReporteSinDetalle => "{} (reportó {})",
        Texto::VolverAtras => "Volver a la pantalla anterior",
        Texto::VolverALaPantallaAnterior => "Volver a la pantalla anterior",
        Texto::CrearRecetaSeccion => "Recetario",
        Texto::RecetaNuevaTitulo => "Nueva receta",
        Texto::CrearRecetaNota => {
            "Pon el nombre, elige el vaso y arma la bebida con ingredientes del catálogo. Se guarda como borrador y la publicas cuando quieras."
        }
        Texto::DatosDeLaBebida => "Datos de la bebida",
        Texto::NombreDeLaReceta => "Nombre",
        Texto::PlaceholderNombreReceta => "Como el Cappuccino de la casa",
        Texto::TipoDeBebida => "Café o coctelería",
        Texto::FamiliaDeLaBebida => "Familia",
        Texto::PlaceholderFamilia => "Con leche, directo, tropical…",
        Texto::CargandoVasos => "Cargando los vasos…",
        Texto::CargandoIngredientes => "Cargando los ingredientes…",
        Texto::EligeIngredienteCorto => "Elige un ingrediente",
        Texto::RecetaSinIngredientes => "Añade al menos un ingrediente.",
        Texto::PasosOpcionales => "Pasos (opcional)",
        Texto::TituloDelPaso => "Título del paso",
        Texto::InstruccionDelPaso => "Qué se hace en este paso",
        Texto::AgregarPaso => "Agregar paso",
        Texto::QuitarPaso => "Quitar paso",
        Texto::CrearRecetaBoton => "Crear receta",
        Texto::IniciaSesionCrear => "Inicia sesión para crear una receta.",
        Texto::FaltaNombreReceta => "Escribe el nombre de la receta.",
        Texto::FaltaFamilia => "Escribe la familia de la bebida.",
        Texto::FaltaVaso => "Elige un vaso del catálogo.",
        Texto::PasoSinTitulo => "Paso",
        Texto::NoCabeEnElVaso => {
            "La bebida no cabe en el vaso: quita cantidad o elige uno más grande."
        }
        Texto::AnadirBoton => "Añadir",
        Texto::FotoNoSeAbrioElSelector => "No se pudo abrir el selector de fotos. Prueba otra vez.",
        Texto::AbriendoElSelector => "Abriendo el selector…",
        Texto::EliminarReceta => "Eliminar",
        Texto::ConfirmarEliminar => "¿Eliminar «{0}»? No se puede deshacer.",
        Texto::SiEliminar => "Sí, eliminar",
        Texto::Eliminando => "Eliminando…",
        Texto::NoSePudoEliminar => "No se pudo eliminar: {0}",
        Texto::VuelveAEntrarEliminar => "Vuelve a entrar para eliminar la receta.",
        Texto::RecetaEnUso => {
            "Está publicada en La Barra y tiene comentarios o versiones derivadas: no se puede eliminar ni retirar."
        }
        Texto::GuardarYPublicar => "Guardar y publicar en La Barra",
        Texto::VersionGuardadaSinPublicar => "La versión quedó guardada, pero no se publicó: {0}",
        Texto::VersionGuardadaSinConexionPublicar => {
            "La versión quedó en la cola: publícala desde Mi recetario cuando vuelva la conexión."
        }
        Texto::VersionGuardadaYPublicada => "La versión quedó guardada y publicada en La Barra.",
        Texto::FotoNoQuitada => "No se pudo quitar la foto: {0}",
        Texto::CatalogoDeIngredientes => "Ingredientes",
        Texto::BuscarIngrediente => "Buscar",
        Texto::NuevoIngrediente => "Agregar ingrediente",
        Texto::NombreIngrediente => "Nombre",
        Texto::CategoriaIngrediente => "Categoría",
        Texto::PlaceholderCategoria => "Licor, zumo, especia…",
        Texto::Unidad => "Unidad",
        Texto::Color => "Color",
        Texto::DiluyeLaBebida => "Diluye la bebida",
        Texto::AgregarIngrediente => "Agregar",
        Texto::IngredienteAgregado => "Ingrediente agregado al catálogo.",
        Texto::NoSePudoAgregarIngrediente => "No se pudo agregar: {0}",
        Texto::SinIngredientesQueCoincidan => "No hay ingredientes que coincidan.",
        Texto::IniciaSesionIngrediente => "Inicia sesión para agregar un ingrediente.",
        Texto::RecetaActualizada => "Cambios guardados.",
        Texto::GuardarCambios => "Guardar cambios",
        Texto::Cuenta => "Cuenta",
        Texto::CuentaNota => {
            "El alias es el nombre que se ve en La Barra; la edad habilita las recetas con alcohol."
        }
        Texto::CuentaSinSesion => "Entra para ver y cambiar tu cuenta.",
        Texto::MayorDeEdadCuenta => "Soy mayor de edad",
        Texto::CuentaGuardada => "Cuenta actualizada.",
        Texto::NoSeGuardoCuenta => "No se pudo guardar la cuenta: {0}",
        Texto::IniciaSesionCuenta => "Vuelve a entrar para cambiar tu cuenta.",
    }
}
