//! Catálogo de textos de la interfaz: una variante por cada texto visible.
//!
//! Los nombres dicen qué se muestra, no dónde: así el mismo texto se reutiliza en varias
//! pantallas y solo hay que traducirlo una vez.

/// Un texto de la interfaz.
///
/// Algunos textos solo se usan en una plataforma (los selectores de fotos de iOS y Android),
/// así que en las demás el compilador los ve sin usar: es esperado.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Texto {
    /// comentó tu receta
    ComentoTuReceta,
    /// respondió tu comentario en
    RespondioTuComentario,
    /// preparó tu receta
    PreparoTuReceta,
    /// publicó una versión:
    PublicoUnaVersion,
    /// revisó un reporte sobre
    RevisoUnReporte,
    /// Nueva
    Nueva,
    /// Cargando tu actividad…
    CargandoActividad,
    /// Entra para ver quién comenta y prepara tus recetas.
    ActividadSinSesion,
    /// Entrar
    Entrar,
    /// Todavía no hay novedades. Publica una receta en La Barra para empezar.
    ActividadVacia,
    /// Más fuerte
    MasFuerte,
    /// Más débil
    MasDebil,
    /// Atajos de proporción
    AtajosDeProporcion,
    /// Se aplican a todas las capas
    SeAplicanATodas,
    /// Escala
    Escala,
    /// Volumen de la bebida en mililitros
    VolumenDeLaBebida,
    /// La escala termina en 500 ml y mueve todas las capas.
    EscalaNota,
    /// Nuevos ingredientes
    NuevosIngredientes,
    /// Líquidos o sólidos · 1 g = 1 ml
    LiquidosOSolidos,
    /// Ingrediente del catálogo
    IngredienteDelCatalogo,
    /// Elige un ingrediente (ml o mg)
    EligeIngrediente,
    /// No hay más ingredientes disponibles en el catálogo.
    SinMasIngredientes,
    /// Quitar
    Quitar,
    /// Nombre de la versión
    NombreDeLaVersion,
    /// Restablecer
    Restablecer,
    /// Inicia sesión para guardar una versión.
    IniciaSesionGuardarVersion,
    /// Versión guardada. Ya aparece en Mi recetario.
    VersionGuardada,
    /// Sin conexión: la versión se enviará cuando vuelva la red y aparecerá en Mi recetario.
    VersionSinConexion,
    /// Guardando…
    Guardando,
    /// Guardar versión privada
    GuardarVersionPrivada,
    /// Abrir versión
    AbrirVersion,
    /// Cargando las proporciones…
    CargandoProporciones,
    /// Esta receta no existe o ya no está publicada.
    RecetaNoPublicada,
    /// No hay vasos disponibles para ajustar la receta.
    NoHayVasos,
    /// Ajustar proporciones
    AjustarProporciones,
    /// Total
    Total,
    /// Del vaso
    DelVaso,
    /// Sin alcohol
    SinAlcohol,
    /// Intensidad base
    IntensidadBase,
    /// El vaso queda al límite: el dibujo se corta al 94 % de la capacidad.
    VasoAlLimite,
    /// Todas las capas están a cero. Sube alguna para ver la bebida.
    TodasLasCapasACero,
    /// Vaso
    Vaso,
    /// Elegir el vaso
    ElegirElVaso,
    /// La proporción se ve en el vaso
    BienvenidaTitulo1,
    /// Cada receta se dibuja como capas dentro de su vaso real, con su porcentaje y sus mililitros.
    BienvenidaTexto1,
    /// Ajusta y haz tu versión
    BienvenidaTitulo2,
    /// Cambia cantidades o vaso; guarda tu versión y publícala en La Barra con crédito al original.
    BienvenidaTexto2,
    /// Contenido con alcohol
    BienvenidaTitulo3,
    /// Los cócteles solo se muestran si al crear tu cuenta declaras que eres mayor de edad. Bebe con responsabilidad.
    BienvenidaTexto3,
    /// Bienvenida · {} de {}
    BienvenidaPaso,
    /// Ratio no vende alcohol ni verifica tu edad: confía en tu declaración.
    BienvenidaAvisoEdad,
    /// Atrás
    Atras,
    /// Empezar
    Empezar,
    /// Siguiente
    Siguiente,
    /// Volver al perfil
    VolverPerfil,
    /// Perfil
    DestinoPerfil,
    /// Ajustes
    Ajustes,
    /// Unidades
    Unidades,
    /// Cómo se muestran las cantidades en la ficha y en las equivalencias.
    UnidadesNota,
    /// Tema
    Tema,
    /// El tema oscuro cambia el fondo y el texto de las pantallas del recetario.
    TemaNota,
    /// Idioma
    IdiomaSeccion,
    /// Cambia el idioma de toda la aplicación.
    IdiomaNota,
    /// Bienvenida
    BienvenidaSeccion,
    /// Volver a ver la bienvenida al salir
    VolverBienvenida,
    /// Mililitros (ml)
    Mililitros,
    /// Onzas (oz)
    Onzas,
    /// Partes
    Partes,
    /// Claro
    Claro,
    /// Oscuro
    Oscuro,
    /// Descubrir
    DestinoDescubrir,
    /// Biblioteca
    DestinoBiblioteca,
    /// Recetario
    DestinoRecetario,
    /// La Barra
    DestinoLaBarra,
    /// Navegación principal
    NavAria,
    /// {} novedades sin leer
    NovedadesSinLeer,
    /// Nombre
    OrdenNombre,
    /// Intensidad
    OrdenIntensidad,
    /// Dificultad
    OrdenDificultad,
    /// Tiempo
    OrdenTiempo,
    /// Todas las recetas publicadas, con su proporción a la vista.
    ProporcionEnLaFicha,
    /// Lista de compra
    ListaDeCompra,
    /// Cargando el catálogo…
    CargandoCatalogo,
    /// No hay recetas que coincidan. Prueba con otra palabra o quita el filtro.
    SinCoincidencias,
    /// {} recetas
    CuentaRecetas,
    /// Buscar por nombre
    BuscarPorNombreCorto,
    /// Filtrar por familia
    FiltrarPorFamilia,
    /// Todas
    TodasLasFamilias,
    /// Cócteles
    Cocteles,
    /// Ordenar la lista
    OrdenarLaLista,
    /// Ordenar
    OrdenarTitulo,
    /// Dificultad {}
    DificultadNivel,
    /// Con leche
    ConLeche,
    /// Sin alcohol
    SinAlcoholTexto,
    /// — igual
    Igual,
    /// Volumen
    Volumen,
    /// Graduación
    Graduacion,
    /// Capas
    Capas,
    /// Volver a la ficha
    VolverALaFicha,
    /// Pon dos recetas frente a frente: el vaso, el volumen y la graduación.
    CompararSubtitulo,
    /// Cargando la receta A…
    CargandoRecetaA,
    /// Esta receta no existe o ya no está disponible.
    RecetaNoDisponible,
    /// Cargando la receta B…
    CargandoRecetaB,
    /// Elige la receta B para ver las dos tazas y sus diferencias.
    EligeRecetaB,
    /// Elige una receta…
    EligeUnaReceta,
    /// Sin el catálogo no se pueden cambiar las recetas.
    SinCatalogoComparar,
    /// Partes
    PartesUnidad,
    /// Diferencias · A frente a B
    DiferenciasAB,
    /// Medidas del motor de proporciones, con las cantidades en {}.
    MedidasDelMotor,
    /// Medida
    Medida,
    /// Ingredientes en otras unidades
    IngredientesOtrasUnidades,
    /// Volver a la receta
    VolverALaReceta,
    /// Sin alcohol
    SinAlcoholCorto,
    /// Ingredientes y equivalencias
    IngredientesYEquivalencias,
    /// Cargando la receta…
    CargandoReceta,
    /// Unidad preferida: {}. Las partes se cuentan contra el ingrediente más pequeño que no es hielo ni agua; 1 cucharada = 15 ml.
    NotaEquivalencias,
    /// Ingrediente
    Ingrediente,
    /// Cdas
    Cucharadas,
    /// Cambiar la unidad preferida
    CambiarUnidadPreferida,
    /// Perfil de {}
    PerfilDe,
    /// Mi perfil
    MiPerfil,
    /// Mi recetario
    MiRecetario,
    /// Guardados
    Guardados,
    /// Actividad
    Actividad,
    /// Salir
    Salir,
    /// Entrar a tu cuenta
    EntrarACuenta,
    /// Buscar recetas
    BuscarRecetas,
    /// Buscar por nombre de bebida
    BuscarPorNombre,
    /// Buscar
    Buscar,
    /// Familias de bebida
    FamiliasDeBebida,
    /// Espresso
    Espresso,
    /// Ver receta
    VerReceta,
    /// Sugerencia del día
    SugerenciaDelDia,
    /// Buscando la sugerencia de hoy…
    BuscandoSugerencia,
    /// Todavía no hay recetas publicadas. Vuelve pronto.
    SinRecetasPublicadas,
    /// Sugerencia anterior
    SugerenciaAnterior,
    /// Sugerencia siguiente
    SugerenciaSiguiente,
    /// Sigue donde lo dejaste
    SigueDondeLoDejaste,
    /// Aún no has abierto ninguna receta. Empieza por la sugerencia de hoy.
    AunNoHayVistas,
    /// ¿Qué vas a preparar hoy?
    QuePreparasHoy,
    /// Esta receta no existe o ya no está publicada.
    RecetaNoPublicadaFicha,
    /// Versión de
    VersionDe,
    /// Capacidad
    Capacidad,
    /// Proporciones en el vaso
    ProporcionesEnElVaso,
    /// Acciones
    Acciones,
    /// Preparar
    Preparar,
    /// Ajustar las proporciones o derivar una versión propia
    AjustarAria,
    /// Ajustar o derivar
    AjustarODerivar,
    /// Comparar
    CompararBoton,
    /// Ingredientes
    Ingredientes,
    /// Reportar
    Reportar,
    /// Inicia sesión para guardar la receta.
    IniciaSesionGuardar,
    /// Sin conexión: se guardará cuando vuelva la red.
    SinConexionSeGuardara,
    /// Guardado
    Guardado,
    /// Guardar
    Guardar,
    /// Preparación
    Preparacion,
    /// Notas de sabor
    NotasDeSabor,
    /// Cada versión deriva de otra y conserva el crédito a su original.
    CadaVersionDeriva,
    /// Ver el historial de versiones
    VerHistorialDeVersiones,
    /// Vuelve a entrar para publicar.
    VuelveAEntrarPublicar,
    /// Sin conexión: se aplicará cuando vuelva la red.
    SinConexionPublicacion,
    /// Un momento…
    UnMomento,
    /// Retirar de La Barra
    RetirarDeLaBarra,
    /// Publicar en La Barra
    PublicarEnLaBarra,
    /// Sin conexión: tu valoración se enviará cuando vuelva la red.
    SinConexionValoracion,
    /// ¿Te salió esta receta?
    TeSalioEstaReceta,
    /// Me salió
    MeSalio,
    /// No me salió
    NoMeSalio,
    /// Aparte, ¿cuánto te gustó? De 1, nada, a 5, mucho
    CuantoTeGusto,
    /// Aparte: ¿te gustó? (opcional)
    TeGustoOpcional,
    /// Tu gusto se guarda en tu diario y no cambia el porcentaje de «me salió».
    GustoNoCambia,
    /// Es tu receta: quienes la preparen dirán si les salió.
    EsTuReceta,
    /// Vuelve a entrar para adjuntar una foto.
    VuelveAEntrarFoto,
    /// No se pudo preparar la foto en este dispositivo.
    FotoNoPreparada,
    /// Escribe tu comentario antes de publicarlo.
    EscribeTuComentario,
    /// Vuelve a entrar para comentar.
    VuelveAEntrarComentar,
    /// La propuesta debe ser un número mayor que cero y necesita una capa o un paso.
    PropuestaInvalida,
    /// Sin conexión: tu comentario se publicará cuando vuelva la red.
    SinConexionComentario,
    /// Cancelar
    Cancelar,
    /// Tu comentario
    TuComentario,
    /// ¿Qué cambiarías o qué te funcionó?
    QueCambiarias,
    /// Sobre
    Sobre,
    /// Toda la receta
    TodaLaReceta,
    /// Foto que vas a adjuntar
    FotoQueVasAAjuntar,
    /// Quitar foto
    QuitarFoto,
    /// Adjuntar foto
    AdjuntarFoto,
    /// Publicando…
    Publicando,
    /// Responder
    Responder,
    /// Comentar
    Comentar,
    /// Vuelve a entrar para reportar.
    VuelveAEntrarReportar,
    /// Gracias: el equipo de moderación lo revisará.
    GraciasModeracion,
    /// Ya habías reportado esto: el equipo de moderación lo está revisando.
    YaHabiasReportado,
    /// Sin conexión: el reporte se enviará cuando vuelva la red.
    SinConexionReporte,
    /// Motivo
    Motivo,
    /// Detalle
    Detalle,
    /// Cuéntanos qué pasa (opcional)
    CuentanosQuePasaOpcional,
    /// Enviar reporte
    EnviarReporte,
    /// La Barra
    LaBarra,
    /// Cargando comentarios…
    CargandoComentarios,
    /// Nadie ha comentado todavía. ¿Cómo te salió?
    SinComentariosAun,
    /// Ver el comentario y responder
    VerComentarioYResponder,
    /// Comentarios pendientes de enviar
    ComentariosPendientes,
    ///  · Pendiente de enviar
    PendienteDeEnviar,
    /// Entra para comentar y valorar.
    EntraParaComentar,
    /// Esta compilación no tiene Storage configurado: no se pueden subir fotos.
    FotoSinStorage,
    /// Esta versión de la app no tiene configuradas las fotos.
    FotoAppSinConfigurar,
    /// Elige una foto JPG, PNG o WebP.
    FotoTipoInvalido,
    /// La foto pesa más de 5 MB.
    FotoPesaDemasiado,
    /// No se pudo nombrar la foto.
    FotoSinNombre,
    /// No se pudo preparar la subida de la foto.
    FotoSinPreparar,
    /// Sin conexión: intenta subir la foto cuando vuelva la red.
    FotoSinConexion,
    /// Storage rechazó la foto.
    FotoRechazada,
    /// No se pudo leer la foto.
    FotoNoLeida,
    /// Subiendo…
    SubiendoFoto,
    /// Vuelve a entrar para subir fotos.
    FotoSinSesion,
    /// Fotos
    Fotos,
    /// Agregar foto
    AgregarFoto,
    /// Tu lista
    TuLista,
    /// Las recetas que quieres volver a preparar.
    GuardadosSub,
    /// Cargando tus guardados…
    CargandoGuardados,
    /// Volver a Descubrir
    VolverADescubrir,
    /// Inicia sesión para ver las recetas que guardaste.
    GuardadosSinSesion,
    /// Todavía no has guardado recetas. Abre una ficha y usa «Guardar».
    GuardadosVacio,
    /// Ir a la biblioteca
    IrALaBiblioteca,
    /// 1 receta guardada
    NRecetasGuardadas,
    /// La sesión caducó; vuelve a entrar.
    SesionCaducada,
    /// Sin conexión: se quitará cuando vuelva la red.
    SinConexionSeQuitara,
    /// Quitando…
    Quitando,
    /// Todas
    TodasFiltro,
    /// Públicas
    Publicas,
    /// Privadas
    Privadas,
    /// Pública
    Publica,
    /// Privada
    PrivadaFiltro,
    /// Volver a la ficha de la receta
    VolverALaFichaReceta,
    /// Historial de versiones
    HistorialDeVersiones,
    /// Cargando el historial…
    CargandoHistorial,
    /// Esta bebida todavía no tiene versiones publicadas.
    SinVersionesBebida,
    /// Receta original
    RecetaOriginal,
    /// Versión
    VersionUna,
    /// Versiones
    Versiones,
    /// Filtrar versiones
    FiltrarVersiones,
    /// Línea de tiempo de versiones
    LineaDeTiempoVersiones,
    /// Toca «Abrir» para ver la ficha de esa versión.
    TocaAbrir,
    /// Datos de la base de Ratio
    DatosDeLaBase,
    /// Derivada
    Derivada,
    /// Deriva de una versión que no está disponible
    DerivaNoDisponible,
    /// Sin cambios frente a su origen
    SinCambiosFrenteAlOrigen,
    /// Receta original · sin cambios
    RecetaOriginalSinCambios,
    /// Abrir →
    Abrir,
    /// Recientes
    Recientes,
    /// Siguiendo
    Siguiendo,
    /// Populares
    Populares,
    /// Conversación
    Conversacion,
    /// Coctelería
    CocteleriaSeccion,
    /// Café
    CafeSeccion,
    /// Cargando más…
    CargandoMas,
    /// Cargar más
    CargarMas,
    /// Sin valoraciones
    SinValoraciones,
    /// Original
    Original,
    /// Comentario
    ComentarioUno,
    /// Comentarios
    ComentarioVarios,
    /// Versiones
    VersionVarias,
    /// Entra para ver lo que publican las cuentas que sigues.
    BarraSinSesion,
    /// Sigue a alguien desde su perfil para ver aquí lo que publica.
    SigueAAlguien,
    /// Todavía no hay recetas con actividad reciente.
    SinActividadReciente,
    /// Todavía no hay recetas publicadas. Publica la tuya desde Mi recetario.
    BarraSinRecetas,
    /// Cargando La Barra…
    CargandoLaBarra,
    /// Nadie ha comentado todavía. Abre una receta y cuenta cómo te salió.
    BarraSinComentarios,
    /// respondió ·
    RespondioEn,
    /// comentó ·
    ComentoEn,
    /// Cargando la conversación…
    CargandoConversacion,
    /// Comunidad
    Comunidad,
    /// Secciones de La Barra
    SeccionesDeLaBarra,
    /// Volver a La Barra
    VolverALaBarra,
    /// ‹ La Barra
    VolverALaBarraBoton,
    /// Ver la receta
    VerLaReceta,
    /// Cargando el hilo…
    CargandoHilo,
    /// Esta receta no existe o ya no está disponible.
    RecetaNoDisponibleBarra,
    /// Hilo
    Hilo,
    /// por
    PorTexto,
    /// Café y especias
    CafeYEspecias,
    /// Lácteos
    Lacteos,
    /// café
    IngredienteCafe,
    /// té
    IngredienteTe,
    /// coñac
    IngredienteConac,
    /// azúcar
    IngredienteAzucar,
    /// Cargando el recetario…
    CargandoRecetarioCorto,
    /// Desmarcar todo
    DesmarcarTodo,
    /// Comensales
    Comensales,
    /// Quitar un comensal
    QuitarComensal,
    /// Agregar un comensal
    AgregarComensal,
    /// Vaciar
    Vaciar,
    /// Elige una o más recetas para empezar.
    EligeRecetasParaLista,
    /// Agregar receta
    AgregarReceta,
    /// Buscar recetas para la lista
    BuscarRecetasParaLista,
    /// La lista aparecerá aquí en cuanto elijas una receta.
    ListaApareceraAqui,
    /// Estas recetas no tienen ingredientes que comprar.
    RecetasSinIngredientes,
    /// 1 cambio pendiente de enviar: se enviará cuando vuelva la conexión.
    CambioPendiente,
    /// Escribe tu correo.
    FaltaCorreo,
    /// El correo no lleva espacios.
    CorreoConEspacios,
    /// Escribe un correo válido, con arroba y dominio.
    CorreoInvalido,
    /// Escribe tu contraseña.
    FaltaContrasena,
    /// La contraseña necesita al menos {} caracteres.
    ContrasenaCorta,
    /// Escribe un alias.
    FaltaAlias,
    /// El alias necesita {} caracteres o más.
    AliasCorto,
    /// El alias no puede pasar de {} caracteres.
    AliasLargo,
    /// El alias no lleva arroba: es el nombre que verán los demás.
    AliasConArroba,
    /// Crear cuenta
    CrearCuenta,
    /// Tu alias y tu edad quedan en el perfil; la contraseña la guarda Supabase cifrada.
    BajadaCrear,
    /// Usa tu correo para volver a tu recetario.
    BajadaEntrar,
    /// Necesitamos que declares que eres mayor de edad.
    FaltaEdad,
    /// Café · Coctelería
    MarcaRubro,
    /// La proporción se ve en el vaso.
    MarcaLema,
    /// Correo
    Correo,
    /// tu@correo.com
    PlaceholderCorreo,
    /// Contraseña
    Contrasena,
    /// Al menos 6 caracteres
    PlaceholderContrasenaCrear,
    /// Tu contraseña
    PlaceholderContrasenaEntrar,
    /// Ocultar
    OcultarContrasena,
    /// Ver
    MostrarContrasena,
    /// Como te verán en La Barra
    PlaceholderAlias,
    /// Tengo 18 años o más y quiero ver recetas con alcohol.
    DeclaracionEdad,
    /// Te enviamos un correo con el enlace para cambiar la contraseña. Revisa también el spam.
    CorreoEnviado,
    /// Mandando el correo…
    MandandoCorreo,
    /// ¿Olvidaste tu contraseña?
    OlvidasteContrasena,
    /// ¿Ya tienes cuenta?
    YaTienesCuenta,
    /// ¿No tienes cuenta?
    NoTienesCuenta,
    /// Contenido con alcohol solo para personas mayores de edad.
    AvisoLegal,
    /// Ratio · Proporción visible, recetas reproducibles.
    PieAcceso,
    /// Vaso con capas de bebida
    VasoConCapas,
    /// Todo
    TodoFiltro,
    /// Versiones
    VersionesFiltro,
    /// Publicadas
    Publicadas,
    /// Borradores
    Borradores,
    /// Todavía no tienes recetas propias. Empieza por ajustar una de la biblioteca.
    SinRecetasPropias,
    /// Todavía no has guardado ninguna versión propia.
    SinVersionesPropias,
    /// Todavía no has publicado una receta.
    SinPublicaciones,
    /// No tienes borradores pendientes.
    SinBorradores,
    /// Borrador
    Borrador,
    /// Privada
    Privada,
    /// Publicada
    Publicada,
    /// Oculta
    Oculta,
    /// Recetario
    Recetario,
    /// Tus versiones, tus publicaciones y lo que aún no sale de casa.
    RecetarioSubtitulo,
    /// Cargando tu recetario…
    CargandoRecetario,
    /// Inicia sesión para ver tus versiones, tus publicaciones y tus borradores.
    RecetarioSinSesion,
    /// Filtrar mi recetario
    FiltrarRecetario,
    /// Creada {}
    CreadaEl,
    /// Vuelve a entrar para moderar.
    VuelveAEntrarModerar,
    /// Escribe la nota del aviso: es lo que leerá quien publicó.
    NotaAviso,
    ///  de
    DeTexto,
    /// Reportado 1 vez
    ReportadoUnaVez,
    /// Nota (obligatoria para avisar)
    NotaObligatoria,
    /// Qué decidiste y por qué
    QueDecidiste,
    /// Decisión sobre este contenido
    DecisionSobreContenido,
    /// Avisar
    Avisar,
    /// Descartar reportes
    DescartarReportes,
    /// El contenido ya no existe.
    ContenidoYaNoExiste,
    /// Restaurar
    Restaurar,
    /// Cargando la cola de reportes…
    CargandoColaReportes,
    /// No hay reportes pendientes.
    SinReportesPendientes,
    /// Todavía no hay decisiones registradas.
    SinDecisiones,
    /// Volver a tu perfil
    VolverATuPerfil,
    /// Moderación
    ModeracionSeccion,
    /// Entra con una cuenta de moderación para revisar los reportes.
    ModeracionSinPermiso,
    /// Secciones de moderación
    SeccionesDeModeracion,
    /// Cola
    Cola,
    /// Historial
    HistorialSeccion,
    /// Tu cuenta
    TuCuenta,
    /// Sin conexión: se aplicará cuando vuelva la red.
    AplicarSinConexion,
    /// Seguir
    Seguir,
    /// Mi diario
    MiDiario,
    /// Cargando tu diario…
    CargandoDiario,
    /// Todavía no has preparado recetas de otras personas. Abre una y di si te salió.
    DiarioVacioAjeno,
    /// Tu perfil
    TuPerfil,
    /// {} recetas publicadas · {} seguidores · {} siguiendo
    ResumenPerfil,
    /// Recetas publicadas
    RecetasPublicadas,
    /// Todavía no hay recetas publicadas.
    RecetasPublicadasVacio,
    /// Cargando el perfil…
    CargandoPerfil,
    /// Entra para ver tu perfil, tus recetas y a quién sigues.
    PerfilSinSesion,
    /// Este perfil no existe.
    PerfilNoExiste,
    /// Salir de la preparación
    SalirDeLaPreparacion,
    /// Esta receta todavía no tiene pasos de preparación.
    RecetaSinPasos,
    /// Paso {} de {}
    PasoDeTotal,
    /// ¡Tiempo!
    Tiempo,
    /// Pausar
    Pausar,
    /// Iniciar temporizador
    IniciarTemporizador,
    /// Anterior
    Anterior,
    /// Terminar
    Terminar,
    /// Siguiente paso
    SiguientePaso,
    /// ¡Listo!
    Listo,
    /// ¿Te salió igual que en la ficha?
    TeSalioIgual,
    /// Preparar otra vez
    PrepararOtraVez,
    /// El servidor no respondió en {} s. Se intentó {} veces: revisa tu conexión y vuelve a probar.
    RedTiempo,
    /// No se pudo abrir la conexión con el servidor ({} s de espera). Revisa tu red o los datos móviles.
    RedConexion,
    /// El servidor respondió con un error {}. Vuelve a intentarlo en un momento.
    RedEstado,
    /// Sin conexión con el servidor. Revisa tu red o los datos móviles y vuelve a intentarlo.
    RedSinConexion,
    /// El servidor no pudo completar la petición. Vuelve a intentarlo.
    RedServidor,
    /// La respuesta del servidor llegó incompleta. Vuelve a intentarlo.
    RedIncompleta,
    /// La conexión se cortó a media respuesta. Vuelve a intentarlo.
    RedCortada,
    /// No se pudo completar la petición: {}
    RedOtra,
    /// Reintentar
    Reintentar,
    /// Cerrar
    Cerrar,
    /// Reportar receta
    ReportarReceta,
    /// Cuéntanos qué pasa con esta receta. Tu reporte es privado y lo revisa el equipo de moderación.
    ReportarExplicacion,
    /// Detalle (opcional)
    DetalleOpcional,
    /// Agrega contexto para el equipo de moderación.
    AgregaContexto,
    /// Solo lo ve moderación
    SoloModeracion,
    /// Gracias. Revisaremos el contenido y tomaremos una decisión.
    GraciasRevisaremos,
    /// Inicia sesión para reportar este contenido.
    IniciaSesionParaReportar,
    /// Reporte enviado
    ReporteEnviado,
    /// Enviando…
    Enviando,
    /// Comprobando…
    Comprobando,
    /// Probar conexión
    ProbarConexion,
    /// Backend y base de datos: conectados
    BackendConectado,
    /// El botón de arriba repite la prueba cuando quieras.
    BotonPruebaConexion,
    /// El teléfono no tiene carpeta privada donde dejar la foto.
    FotoSinCarpeta,
    /// No se eligió ninguna foto a tiempo.
    FotoSinElegir,
    /// No se pudo abrir la galería en este momento.
    GaleriaNoAbre,
    /// Este dispositivo no tiene galería de fotos.
    SinGaleria,
    /// Cantidad de {}
    CantidadDe,
    /// {} mililitros, {} por ciento
    LectorMililitros,
    /// {} miligramos, suma {} mililitros, {} por ciento
    LectorMiligramos,
    /// {} % de la receta
    PorcentajeDeLaReceta,
    /// {} % del total
    PorcentajeDelTotal,
    /// {} % del agregado
    PorcentajeDelAgregado,
    /// {} · versión
    NombreVersionPropuesto,
    /// Versión guardada. {}
    VersionGuardadaConAviso,
    /// No pudimos guardar la versión: {}
    NoSeGuardoVersion,
    /// por {}
    PorAutor,
    /// {} de 5
    NivelDeCinco,
    /// No cabe: sobran {} ml. Baja una capa o elige un vaso más grande.
    NoCabeSobran,
    /// {} de {}
    DeTotalCorto,
    /// · original de {}
    OriginalDe,
    /// {} ml en {} ml de vaso
    MlEnVaso,
    /// No se pudo guardar: {}
    NoSePudoGuardar,
    /// No se pudo cambiar la publicación: {}
    NoSePudoCambiarPublicacion,
    /// No se pudo guardar tu gusto: {}
    NoSePudoGuardarGusto,
    /// No se pudo guardar tu valoración: {}
    NoSePudoGuardarValoracion,
    /// A {} les salió · a {} no les salió
    RecuentoValoracion,
    /// Gusto {} de 5
    GustoDeCinco,
    /// No se pudo publicar el comentario: {}
    NoSePudoComentar,
    /// Reportar este {}
    ReportarEste,
    /// No se pudo enviar el reporte: {}
    NoSePudoReportar,
    /// Ver el perfil de {}
    VerPerfilDe,
    /// Foto de {}
    FotoDe,
    /// {} sin leer
    SinLeer,
    /// Ver los {} comentarios y responder
    VerLosComentarios,
    /// Foto {} de {}
    FotoNumeroDe,
    /// Hasta {} fotos JPG, PNG o WebP de 5 MB como máximo.
    FotosMaximo,
    /// Guardado el {}
    GuardadoEl,
    /// Última vez: {}
    UltimaVez,
    /// Quitar {} de guardados
    QuitarDeGuardados,
    /// No se pudo quitar: {}
    NoSePudoQuitar,
    /// {} % me salió ({})
    PorcentajeMeSalio,
    /// Versión de {}
    VersionDeOrigen,
    /// Abrir el hilo de {}
    AbrirHiloDe,
    /// {} de {} listos
    ListosDeTotal,
    /// {} cambios pendientes de enviar: se enviarán cuando vuelva la conexión.
    CambiosPendientes,
    /// {} de {} recetas
    DeTotalRecetas,
    /// Reportado {} veces
    ReportadoVeces,
    /// No se pudo restaurar: {}
    NoSePudoRestaurar,
    /// No se pudo actualizar: {}
    NoSePudoActualizar,
    /// Tiempo de preparación: {}
    TiempoDePreparacion,
    /// No se pudo guardar la decisión: {}
    NoSePudoGuardarDecision,
    /// Backend responde, base de datos no: {}
    BackendSinBase,
    /// {} (probado en {})
    ProbadoEn,
    /// La foto no terminó de subir en {} s: revisa tu conexión y vuelve a intentarlo.
    FotoNoTermino,
    /// La foto se subió, pero no se pudo agregar a la receta: {}
    FotoNoAgregada,
    /// No se pudo leer la foto elegida.
    FotoNoLeidaAndroid,
    /// Sin conexión: la versión se enviará cuando vuelva la red.
    VersionSinAviso,
    /// hoy
    HaceHoy,
    /// ayer
    HaceAyer,
    /// hace {} días
    HaceDias,
    /// hace {} h
    HaceHoras,
    /// hace {} min
    HaceMinutos,
    /// hace un momento
    HaceMomentos,
    /// Preparación
    PreparacionSeccion,
    /// Todo lo que necesitas para tus recetas, sumado en una sola lista.
    ListaSubtitulo,
    /// Unidad de la lista
    UnidadDeLaLista,
    /// Recetas en la lista
    RecetasEnLaLista,
    /// No hay recetas que coincidan.
    SinCoincidenciasCorto,
    /// Volver a la biblioteca
    VolverALaBiblioteca,
    /// Biblioteca
    BibliotecaTitulo,
    /// Recetario
    RecetarioSeccion,
    /// ‹ Descubrir
    VolverDescubrirBoton,
    /// Versiones
    VersionesSeccion,
    /// Servido
    Servido,
    /// Actualizada {}
    ActualizadaEl,
    /// {} min
    MinutosCorto,
    /// {} ml
    MililitrosCorto,
    /// {} oz
    OnzasCorto,
    /// {} partes
    PartesCorto,
    /// Spam o publicidad
    MotivoSpam,
    /// Ofensivo
    MotivoOfensivo,
    /// Peligroso
    MotivoPeligroso,
    /// Alcohol para menores
    MotivoMenores,
    /// Otro motivo
    MotivoOtro,
    /// Directo
    FamiliaDirecto,
    /// Largo
    FamiliaLargo,
    /// Corto
    FamiliaCorto,
    /// Clásico
    FamiliaClasico,
    /// Frío
    FamiliaFrio,
    /// Caliente
    FamiliaCaliente,
    /// Cremoso
    FamiliaCremoso,
    /// Cítrico
    FamiliaCitrico,
    /// Tropical
    FamiliaTropical,
    /// Con hielo
    FamiliaConHielo,
    /// {} s
    DuracionSegundos,
    /// {} min
    DuracionMinutos,
    /// {} h {} min
    DuracionHoras,
    /// Capa · {}
    AnclaCapa,
    /// Paso {} · {}
    AnclaPaso,
    /// Receta
    ObjetivoReceta,
    /// Comentario
    ObjetivoComentario,
    /// Retirado
    AccionRetirado,
    /// Restaurado
    AccionRestaurado,
    /// Reportes descartados
    AccionDescartado,
    /// Aviso enviado
    AccionAvisoEnviado,
    /// Le salió
    LeSalio,
    /// No le salió
    NoLeSalio,
    /// Retiramos tu {} de La Barra porque no cumple las normas.
    ModeracionOcultarAutor,
    /// Restauramos tu {}: vuelve a estar visible.
    ModeracionRestaurarAutor,
    /// Recibimos un reporte sobre tu {}. Revísalo, por favor.
    ModeracionAvisoAutor,
    /// Revisamos tu reporte y retiramos el contenido.
    ModeracionOcultarReporte,
    /// Revisamos tu reporte y avisamos a quien lo publicó.
    ModeracionAvisoReporte,
    /// Revisamos tu reporte y no encontramos nada que retirar.
    ModeracionDescartado,
    /// Nota de moderación: {}
    ModeracionNota,
    /// La bebida queda al límite del vaso.
    AvisoAlLimite,
    /// Revisa los datos: algo no tiene el formato correcto.
    ErrorDatos,
    /// Tu sesión caducó: vuelve a entrar.
    ErrorSesion,
    /// Eso ya no existe o no está disponible para tu cuenta.
    ErrorNoEncontrado,
    /// El servidor no pudo completar la petición. Vuelve a intentarlo.
    ErrorServidor,
    /// {} s
    SegundosCorto,
    /// Propone: {}
    Propone,
    /// Propuesta (opcional)
    PropuestaOpcional,
    /// segundos
    SegundosPlaceholder,
    /// cantidad
    CantidadPlaceholder,
    /// {}: {} (reportó {})
    ReporteConDetalle,
    /// {} (reportó {})
    ReporteSinDetalle,
    /// Volver a la pantalla anterior
    VolverAtras,
    /// Volver a la pantalla anterior
    VolverALaPantallaAnterior,
    /// Recetario
    CrearRecetaSeccion,
    /// Nueva receta
    RecetaNuevaTitulo,
    /// Pon el nombre, elige el vaso y arma la bebida con ingredientes del catálogo. Se guarda como borrador y la publicas cuando quieras.
    CrearRecetaNota,
    /// Datos de la bebida
    DatosDeLaBebida,
    /// Nombre
    NombreDeLaReceta,
    /// Como el Cappuccino de la casa
    PlaceholderNombreReceta,
    /// Café o coctelería
    TipoDeBebida,
    /// Familia
    FamiliaDeLaBebida,
    /// Con leche, directo, tropical…
    PlaceholderFamilia,
    /// Cargando los vasos…
    CargandoVasos,
    /// Cargando los ingredientes…
    CargandoIngredientes,
    /// Elige un ingrediente
    EligeIngredienteCorto,
    /// Añade al menos un ingrediente.
    RecetaSinIngredientes,
    /// Pasos (opcional)
    PasosOpcionales,
    /// Título del paso
    TituloDelPaso,
    /// Qué se hace en este paso
    InstruccionDelPaso,
    /// Agregar paso
    AgregarPaso,
    /// Quitar paso
    QuitarPaso,
    /// Crear receta
    CrearRecetaBoton,
    /// Inicia sesión para crear una receta.
    IniciaSesionCrear,
    /// Escribe el nombre de la receta.
    FaltaNombreReceta,
    /// Escribe la familia de la bebida.
    FaltaFamilia,
    /// Elige un vaso del catálogo.
    FaltaVaso,
    /// Paso
    PasoSinTitulo,
    /// La bebida no cabe en el vaso: quita cantidad o elige uno más grande.
    NoCabeEnElVaso,
    /// Añadir
    AnadirBoton,
    /// No se pudo abrir el selector de fotos. Prueba otra vez.
    FotoNoSeAbrioElSelector,
    /// Abriendo el selector…
    AbriendoElSelector,
    /// Eliminar
    EliminarReceta,
    /// ¿Eliminar «{0}»? No se puede deshacer.
    ConfirmarEliminar,
    /// Sí, eliminar
    SiEliminar,
    /// Eliminando…
    Eliminando,
    /// No se pudo eliminar: {0}
    NoSePudoEliminar,
    /// Vuelve a entrar para eliminar la receta.
    VuelveAEntrarEliminar,
    /// Está publicada en La Barra y tiene comentarios o versiones derivadas: no se puede eliminar ni retirar.
    RecetaEnUso,
    /// Guardar y publicar en La Barra
    GuardarYPublicar,
    /// La versión quedó guardada, pero no se publicó: {0}
    VersionGuardadaSinPublicar,
    /// La versión quedó en la cola: publícala desde Mi recetario cuando vuelva la conexión.
    VersionGuardadaSinConexionPublicar,
    /// La versión quedó guardada y publicada en La Barra.
    VersionGuardadaYPublicada,
    /// No se pudo quitar la foto: {0}
    FotoNoQuitada,
    /// Ingredientes
    CatalogoDeIngredientes,
    /// Buscar
    BuscarIngrediente,
    /// Agregar ingrediente
    NuevoIngrediente,
    /// Nombre
    NombreIngrediente,
    /// Categoría
    CategoriaIngrediente,
    /// Licor, zumo, especia…
    PlaceholderCategoria,
    /// Unidad
    Unidad,
    /// Color
    Color,
    /// Diluye la bebida
    DiluyeLaBebida,
    /// Agregar
    AgregarIngrediente,
    /// Ingrediente agregado al catálogo.
    IngredienteAgregado,
    /// No se pudo agregar: {0}
    NoSePudoAgregarIngrediente,
    /// No hay ingredientes que coincidan.
    SinIngredientesQueCoincidan,
    /// Inicia sesión para agregar un ingrediente.
    IniciaSesionIngrediente,
    /// Cambios guardados.
    RecetaActualizada,
    /// Guardar cambios
    GuardarCambios,
    /// Cuenta
    Cuenta,
    /// El alias es el nombre que se ve en La Barra; la edad habilita las recetas con alcohol.
    CuentaNota,
    /// Entra para ver y cambiar tu cuenta.
    CuentaSinSesion,
    /// Alias
    Alias,
    /// Soy mayor de edad
    MayorDeEdadCuenta,
    /// Cuenta actualizada.
    CuentaGuardada,
    /// No se pudo guardar la cuenta: {0}
    NoSeGuardoCuenta,
    /// Vuelve a entrar para cambiar tu cuenta.
    IniciaSesionCuenta,
}
