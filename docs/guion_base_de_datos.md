# Guion: la base de datos de Ratio

## Introducción

La base de datos de Ratio vive en PostgreSQL, administrado por Supabase, y se compone de once tablas definidas en una sola migración versionada que se aplica con `sqlx migrate run`. Antes de recorrerlas conviene aclarar una decisión que atraviesa todo el esquema: casi todos los identificadores son UUID generados en el propio teléfono, no por el servidor. La razón es práctica. Ratio debe permitir crear borradores, comentar o registrar una preparación sin conexión, y esos cambios esperan en una cola hasta que vuelve la red; si el identificador dependiera del servidor, la app no podría relacionar entre sí los registros que creó mientras estaba desconectada.

La segunda decisión general tiene que ver con la seguridad. Supabase publica automáticamente una API sobre cada tabla del esquema público, y nosotros no la usamos: la app habla solo con nuestras server functions. Por eso todas las tablas tienen activada la seguridad a nivel de fila (RLS) sin ninguna política, lo que equivale a cerrar esa puerta. El backend se conecta como dueño de las tablas y no se ve afectado.

Hay un tercer detalle, más técnico, que conviene mencionar desde el inicio. Todas las cantidades (mililitros, miligramos, medidas del vaso, propuestas) se guardan como `double precision`, el mismo tipo `f64` que usa el motor de proporciones en Rust, de modo que SQLx las entrega sin conversiones.

## Usuarios

La tabla **usuario** guarda el perfil de cada persona. Resulta interesante observar lo que *no* contiene: ni correo ni contraseña. Esa información la administra Supabase Auth en su propia tabla `auth.users`, y nuestro `usuario.id` es exactamente el mismo identificador, enlazado con una llave foránea. Si alguien elimina su cuenta, el borrado en cascada se lleva su perfil y, con él, sus recetas, comentarios y preparaciones.

El perfil no lo crea el backend, sino la propia base de datos. Al registrarse, la app envía en los metadatos de Supabase Auth el alias y la declaración de mayoría de edad, y un trigger sobre `auth.users` inserta la fila correspondiente en el mismo instante. El alias es obligatorio, no puede quedar en blanco y es único sin distinguir mayúsculas, así que «ana» y «ANA» no pueden convivir; si el alias ya está ocupado, el registro completo se cancela, por lo que la app debe comprobar su disponibilidad antes de enviarlo. Quien entra con un proveedor social, como Google, no pasa por ese formulario, y para no bloquearlo el trigger le asigna un alias provisional que después puede cambiar.

La columna `mayor_de_edad` merece una aclaración. Ratio no guarda la fecha de nacimiento ni la edad, solo un sí o un no, y ese dato es privado: decide si la persona ve recetas con alcohol, pero nunca aparece en su perfil. Si la app no lo declara, se asume que es menor de edad. Junto a ella está la ruta del avatar en Supabase Storage; la imagen misma nunca entra en la base de datos. Los campos `modo_uso`, `nivel`, `intereses` y `equipo` corresponden a la pantalla «¿Cómo preparas?» del registro. Todos admiten quedar vacíos porque esa pantalla se puede saltar, y se usan después para ordenar la biblioteca y sugerir recetas que el usuario puede preparar con lo que ya tiene; `intereses` solo acepta café y coctelería.

## Vasos e ingredientes

**vaso** describe la silueta de cada recipiente con las mismas medidas que usa el motor de proporciones: radio de la boca y del fondo, altura, grosor de la pared, labio, tallo, asa y plato. Esta coincidencia no es casual. El servidor puede leer una fila y entregarla directamente al código que dibuja la taza, sin conversiones intermedias. Los usuarios pueden crear sus propios vasos, y por eso la tabla registra a su autor; los del catálogo oficial no tienen autor. Si alguien borra su cuenta, sus vasos permanecen sin autor, porque otras personas podrían estar sirviendo sus recetas en ellos.

**ingrediente**, en cambio, es un catálogo cerrado que carga el equipo y que los usuarios no pueden ampliar. La restricción tiene un motivo concreto: el generador de la imagen del vaso necesita colores y graduaciones confiables para dibujar cada capa. Cada ingrediente indica además su unidad, mililitros para los líquidos y miligramos para los sólidos, y un sólido como el azúcar se dibuja también como una capa más dentro del vaso. El color se valida con una expresión regular idéntica a la que aplica el generador de SVG, de modo que un valor inválido se rechaza en la base de datos antes de llegar a la pantalla.

## Recetas

**receta** es la tabla central. Cada receta pertenece a un autor, se sirve en un vaso y es de tipo café o coctelería. Dos columnas merecen atención. La primera es `receta_origen_id`, que apunta a otra receta y convierte a esta en una versión derivada; así se construye la cadena «Negroni 1:1:1 → Esta · 2:1:1 → 4 derivadas» que aparece en el hilo, y una restricción impide que una receta derive de sí misma. La segunda es `publicada_en`: mientras vale NULL la receta es un borrador privado, y en cuanto se publica en La Barra registra la fecha. Las fotos, hasta cinco, se guardan como rutas de Storage en un arreglo.

Podría parecer que falta la columna «ratio» que menciona la planeación. Se omitió a propósito, ya que la proporción se calcula a partir de los componentes; si se guardara aparte, tarde o temprano dejaría de coincidir con ellos.

**componente_receta** almacena cada capa de la bebida, ordenada desde el fondo (posición 0) hacia arriba, con su cantidad y su unidad. Aquí aparece una decisión de diseño menos evidente. Cada capa apunta a su ingrediente del catálogo, pero además copia el nombre, el color, la graduación y la unidad. Si bien es cierto que esto duplica algunos datos, también garantiza que una receta publicada se vea igual para siempre, aunque mañana el equipo corrija el color del Campari. Por la misma razón, un ingrediente no puede borrarse del catálogo mientras alguna receta lo use.

**paso** guarda las instrucciones en orden, cada una con un temporizador opcional en segundos, como el «refrescar 30 s» de un cóctel.

Las recetas se pueden editar incluso después de publicarse, y eso explica por qué cada capa y cada paso tienen su propio identificador además de su posición. La posición solo indica el orden; el identificador es lo que permanece. Reordenar las capas, por ejemplo subir el vermut por encima del gin, no rompe nada, porque la restricción que impide dos capas en la misma posición se verifica al terminar la transacción y no fila por fila.

## La Barra

**comentario** es la tabla más elaborada del foro, porque refleja la pantalla del hilo anclado. Un comentario puede responder a otro y formar así un hilo, y puede anclarse a una capa («Anclado a Vermut rojo 20 ml») o a un paso, nunca a ambos a la vez. El ancla apunta al identificador de la capa o del paso, no a su posición, de modo que el comentario sigue al vermut aunque el autor reordene la receta. La base de datos verifica además que la capa, el paso o el comentario al que se responde pertenezcan a la misma receta. Cuando está anclado, el comentario puede llevar una propuesta numérica: una cantidad en la unidad de la capa, o segundos si se ancla a un paso. Si la capa o el paso desaparecen en una edición, el ancla se libera automáticamente pero el comentario permanece, y la propuesta deja de mostrarse porque ya no tiene sobre qué aplicarse.

**preparacion** registra el resultado de la pantalla «¿Te salió?». Esta tabla separa dos preguntas que suelen mezclarse. `me_salio` indica si la persona logró reproducir la ficha, y con esa columna se calcula el «76 % me salió» que aparece en cada tarjeta. `gusto`, del 1 al 5 y opcional, mide otra cosa. Mantenerlas en columnas distintas evita que una receta difícil pero deliciosa quede castigada, y viceversa. También se guarda la duración de la comanda.

**guardado** y **seguimiento** son tablas de relación sencillas. La primera alimenta la pantalla de recetas guardadas y la segunda la pestaña «Siguiendo» de La Barra; una restricción impide que alguien se siga a sí mismo.

## Moderación

**reporte** alimenta la cola del moderador. Cada reporte apunta exactamente a una receta o a un comentario, condición que la base de datos verifica, e incluye un motivo cerrado, un detalle libre y un estado que pasa de pendiente a aceptado o descartado; un reporte resuelto siempre lleva su fecha de decisión, y una misma persona no puede reportar dos veces lo mismo. Cuando un moderador acepta un reporte, la receta o el comentario se marca con `oculta` u `oculto` en lugar de borrarse, lo que deja un rastro revisable de cada decisión. Quién es moderador no se guarda en estas tablas: viaja en el JWT de Supabase y el backend lo lee al verificar la sesión.

## Índices

Los índices responden a las pantallas concretas. Uno sirve para listar las recetas de un autor y otro para contar las derivadas de una receta. Hay un índice parcial que ordena solo las recetas publicadas y visibles para la pestaña «Nuevas», y un índice de texto completo en español para la búsqueda por nombre. Los comentarios se recuperan por receta en orden cronológico, y las preparaciones se agrupan por receta, para el porcentaje, y por usuario, para la pantalla de actividad. Otros índices sirven a la lista de seguidores, a los vasos de cada usuario y a la cola de reportes pendientes.

Resulta menos obvio que varios índices existen para los borrados y no para las pantallas. Cuando se elimina una cuenta, una receta o una capa, PostgreSQL debe localizar todas las filas que dependen de ella, ya sea para borrarlas en cascada o para liberar un ancla; sin un índice en esas columnas tendría que recorrer la tabla completa. Por último, dos índices únicos garantizan que el alias no se repita sin importar mayúsculas y que nadie reporte dos veces la misma receta o el mismo comentario.
