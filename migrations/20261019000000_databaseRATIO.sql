-- Perfil de cada cuenta de Supabase Auth (el correo y la contraseña viven en auth.users)
CREATE TABLE usuario
(
    id uuid PRIMARY KEY REFERENCES auth.users (id) ON DELETE CASCADE, -- Mismo id que la cuenta de Supabase Auth
    alias text NOT NULL CHECK (btrim(alias) <> ''), -- Nombre visible en La Barra (sin @, obligatorio y no vacío)
    mayor_de_edad boolean NOT NULL DEFAULT false, -- Declaración de mayoría de edad del registro que habilita las recetas con alcohol (privada, no se muestra en el perfil)
    avatar_ruta text, -- Ruta del avatar en Supabase Storage (NULL = sin avatar)
    modo_uso text CHECK (modo_uso IN ('casa', 'comparto', 'oficio')), -- Respuesta a «¿Cómo preparas?» (NULL = omitido)
    nivel text CHECK (nivel IN ('principiante', 'intermedio', 'avanzado')), -- Experiencia declarada (NULL = omitido)
    intereses text[] NOT NULL DEFAULT '{}' CHECK (intereses <@ ARRAY['cafe', 'cocteleria']), -- Tipos de bebida que le interesan (solo 'cafe' y 'cocteleria')
    equipo text[] NOT NULL DEFAULT '{}', -- Utensilios que tiene (báscula, V60, jigger...)
    creado_en timestamptz NOT NULL DEFAULT now() -- Fecha de registro
);

-- Silueta de un vaso que dibuja crates/proporciones (medidas en unidades del lienzo); los usuarios pueden crear los suyos
CREATE TABLE vaso
(
    id uuid PRIMARY KEY, -- Generado en la app para funcionar sin conexión
    autor_id uuid REFERENCES usuario (id) ON DELETE SET NULL, -- Usuario que lo creó (NULL = catálogo oficial o autor eliminado, para no borrar recetas ajenas que lo usan)
    nombre text NOT NULL, -- Nombre visible (Taza 300, Rocks, Copa)
    capacidad_ml double precision NOT NULL CHECK (capacidad_ml > 0), -- Volumen útil hasta el borde interior (ml)
    radio_boca double precision NOT NULL CHECK (radio_boca > 0), -- Media anchura de la boca
    radio_fondo double precision NOT NULL CHECK (radio_fondo >= 0), -- Media anchura del fondo (0 = cono)
    altura double precision NOT NULL CHECK (altura > 0), -- Altura interior del vaso
    pared double precision NOT NULL CHECK (pared >= 0), -- Grosor de la pared
    borde double precision NOT NULL DEFAULT 0 CHECK (borde >= 0), -- Labio sobre el nivel máximo del líquido
    tallo double precision NOT NULL DEFAULT 0 CHECK (tallo >= 0), -- Altura del tallo en copas (0 = sin tallo)
    asa text CHECK (asa IN ('chica', 'grande')), -- Tamaño del asa (NULL = sin asa)
    plato boolean NOT NULL DEFAULT false, -- Dibuja plato bajo la taza
    creado_en timestamptz NOT NULL DEFAULT now() -- Fecha de alta
);

-- Catálogo cerrado de ingredientes (lo carga el equipo, no los usuarios) con nombre, color, graduación y unidad
CREATE TABLE ingrediente
(
    id uuid PRIMARY KEY, -- Generado al cargar el catálogo
    nombre text NOT NULL, -- Nombre visible (Espresso, Gin, Campari)
    unidad text NOT NULL DEFAULT 'ml' CHECK (unidad IN ('ml', 'mg')), -- Unidad de medida (ml = líquido, mg = sólido)
    graduacion_pct double precision NOT NULL DEFAULT 0 CHECK (graduacion_pct BETWEEN 0 AND 100), -- Alcohol por volumen (0-100)
    color text CHECK (color ~ '^#[0-9A-Fa-f]{6}$'), -- Color de la capa (mismo formato que valida svg.rs)
    es_dilucion boolean NOT NULL DEFAULT false, -- Agua o hielo derretido que diluye la bebida
    creado_en timestamptz NOT NULL DEFAULT now() -- Fecha de alta en el catálogo
);

-- Receta de una bebida; si tiene receta de origen es una versión derivada
CREATE TABLE receta
(
    id uuid PRIMARY KEY, -- Generado en la app para funcionar sin conexión
    autor_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Usuario que la creó
    receta_origen_id uuid REFERENCES receta (id) ON DELETE SET NULL, -- Receta de la que deriva (NULL = original)
    nombre text NOT NULL, -- Nombre visible (Negroni de la casa)
    tipo text NOT NULL CHECK (tipo IN ('cafe', 'cocteleria')), -- Pestaña Café o Coctelería
    vaso_id uuid NOT NULL REFERENCES vaso (id), -- Vaso donde se sirve (no se puede borrar mientras se use)
    capacidad_ml double precision CHECK (capacidad_ml > 0), -- Volumen servido (ml, NULL = la del vaso)
    fotos_rutas text[] NOT NULL DEFAULT '{}' CHECK (cardinality(fotos_rutas) <= 5), -- Rutas en Supabase Storage (máximo 5)
    equipo text[] NOT NULL DEFAULT '{}', -- Utensilios necesarios (V60, báscula...)
    publicada_en timestamptz, -- Fecha de publicación en La Barra (NULL = borrador)
    oculta boolean NOT NULL DEFAULT false, -- Retirada por un moderador
    creada_en timestamptz NOT NULL DEFAULT now(), -- Fecha de creación del borrador
    CHECK (receta_origen_id <> id) -- Una receta no puede derivar de sí misma
);

-- Capa de una receta: copia nombre, color, graduación y unidad para que editar el catálogo no altere recetas publicadas
CREATE TABLE componente_receta
(
    id uuid PRIMARY KEY, -- Generado en la app; los comentarios anclados siguen a la capa aunque cambie de posición
    receta_id uuid NOT NULL REFERENCES receta (id) ON DELETE CASCADE, -- Receta a la que pertenece la capa
    posicion smallint NOT NULL CHECK (posicion >= 0), -- Orden de la capa (0 = fondo)
    ingrediente_id uuid NOT NULL REFERENCES ingrediente (id), -- Ingrediente del catálogo (no se puede borrar mientras se use)
    nombre text NOT NULL, -- Nombre del ingrediente en esta receta
    graduacion_pct double precision NOT NULL DEFAULT 0 CHECK (graduacion_pct BETWEEN 0 AND 100), -- Alcohol por volumen (0-100)
    color text CHECK (color ~ '^#[0-9A-Fa-f]{6}$'), -- Color de la capa (NULL = color por defecto)
    cantidad double precision NOT NULL CHECK (cantidad > 0), -- Cantidad del ingrediente en su unidad
    unidad text NOT NULL CHECK (unidad IN ('ml', 'mg')), -- Unidad copiada del ingrediente (ml = líquido, mg = sólido)
    dilucion boolean NOT NULL DEFAULT false, -- Capa de agua o hielo derretido
    UNIQUE (receta_id, posicion) DEFERRABLE INITIALLY DEFERRED, -- Una capa por posición (se verifica al confirmar para poder reordenar)
    UNIQUE (receta_id, id) -- Permite exigir que el ancla de un comentario sea de la misma receta
);

-- Instrucción de preparación con temporizador opcional
CREATE TABLE paso
(
    id uuid PRIMARY KEY, -- Generado en la app; los comentarios anclados siguen al paso aunque cambie de posición
    receta_id uuid NOT NULL REFERENCES receta (id) ON DELETE CASCADE, -- Receta a la que pertenece el paso
    posicion smallint NOT NULL CHECK (posicion >= 0), -- Orden del paso (0 = primero)
    instruccion text NOT NULL, -- Texto del paso (Refrescar 30 s con hielo)
    temporizador_s integer CHECK (temporizador_s > 0), -- Duración del temporizador (s, NULL = sin temporizador)
    UNIQUE (receta_id, posicion) DEFERRABLE INITIALLY DEFERRED, -- Un paso por posición (se verifica al confirmar para poder reordenar)
    UNIQUE (receta_id, id) -- Permite exigir que el ancla de un comentario sea de la misma receta
);

-- Comentario de La Barra; puede responder a otro y anclarse a una capa o a un paso con una propuesta de cambio
CREATE TABLE comentario
(
    id uuid PRIMARY KEY, -- Generado en la app para funcionar sin conexión
    receta_id uuid NOT NULL REFERENCES receta (id) ON DELETE CASCADE, -- Receta comentada
    autor_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Usuario que comenta
    respuesta_a uuid, -- Comentario de la misma receta al que responde (NULL = hilo nuevo)
    texto text NOT NULL CHECK (length(texto) BETWEEN 1 AND 2000), -- Contenido (1-2000 caracteres)
    componente_id uuid, -- Capa anclada (NULL = sin anclar a capa)
    paso_id uuid, -- Paso anclado (NULL = sin anclar a paso)
    propuesta double precision CHECK (propuesta > 0), -- Valor propuesto: cantidad en la unidad de la capa si ancla capa, s si ancla paso (sin ancla se ignora)
    oculto boolean NOT NULL DEFAULT false, -- Retirado por un moderador
    creado_en timestamptz NOT NULL DEFAULT now(), -- Fecha de publicación
    UNIQUE (receta_id, id), -- Permite exigir que una respuesta sea de la misma receta
    CHECK (componente_id IS NULL OR paso_id IS NULL), -- Se ancla a una capa o a un paso, no a ambos
    FOREIGN KEY (receta_id, respuesta_a) REFERENCES comentario (receta_id, id) ON DELETE CASCADE, -- Borrar el comentario borra sus respuestas
    FOREIGN KEY (receta_id, componente_id) REFERENCES componente_receta (receta_id, id) ON DELETE SET NULL (componente_id), -- Borrar la capa libera el ancla
    FOREIGN KEY (receta_id, paso_id) REFERENCES paso (receta_id, id) ON DELETE SET NULL (paso_id) -- Borrar el paso libera el ancla
);

-- Resultado de preparar una receta (pantalla «¿Te salió?»); el gusto se guarda aparte del resultado
CREATE TABLE preparacion
(
    id uuid PRIMARY KEY, -- Generado en la app para funcionar sin conexión
    usuario_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Quien preparó la bebida
    receta_id uuid NOT NULL REFERENCES receta (id) ON DELETE CASCADE, -- Receta preparada
    me_salio boolean NOT NULL, -- Pudo reproducirla igual a la ficha
    gusto smallint CHECK (gusto BETWEEN 1 AND 5), -- Valoración opcional (1-5, NULL = omitida)
    duracion_s integer CHECK (duracion_s >= 0), -- Tiempo de la comanda (s)
    creada_en timestamptz NOT NULL DEFAULT now() -- Fecha de la preparación
);

-- Receta guardada por un usuario (pantalla Ver guardados)
CREATE TABLE guardado
(
    usuario_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Usuario que guarda
    receta_id uuid NOT NULL REFERENCES receta (id) ON DELETE CASCADE, -- Receta guardada
    creado_en timestamptz NOT NULL DEFAULT now(), -- Fecha en que se guardó
    PRIMARY KEY (usuario_id, receta_id)
);

-- Relación de seguimiento entre usuarios (pestaña Siguiendo de La Barra)
CREATE TABLE seguimiento
(
    seguidor_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Usuario que sigue
    seguido_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Usuario seguido
    creado_en timestamptz NOT NULL DEFAULT now(), -- Fecha en que empezó a seguir
    PRIMARY KEY (seguidor_id, seguido_id),
    CHECK (seguidor_id <> seguido_id) -- Nadie se sigue a sí mismo
);

-- Reporte de una receta o un comentario para la cola de moderación
CREATE TABLE reporte
(
    id uuid PRIMARY KEY, -- Generado en la app para funcionar sin conexión
    reportante_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE, -- Usuario que reporta
    receta_id uuid REFERENCES receta (id) ON DELETE CASCADE, -- Receta reportada (NULL = se reporta un comentario)
    comentario_id uuid REFERENCES comentario (id) ON DELETE CASCADE, -- Comentario reportado (NULL = se reporta una receta)
    motivo text NOT NULL CHECK (motivo IN ('spam', 'ofensivo', 'peligroso', 'menores', 'otro')), -- Causa elegida
    detalle text, -- Explicación libre del reportante
    estado text NOT NULL DEFAULT 'pendiente' CHECK (estado IN ('pendiente', 'aceptado', 'descartado')), -- Decisión del moderador
    moderador_id uuid REFERENCES usuario (id) ON DELETE SET NULL, -- Moderador que resolvió (NULL = sin resolver o moderador eliminado)
    creado_en timestamptz NOT NULL DEFAULT now(), -- Fecha del reporte
    resuelto_en timestamptz, -- Fecha de la decisión (NULL = pendiente)
    CHECK ((receta_id IS NULL) <> (comentario_id IS NULL)), -- Reporta exactamente una receta o un comentario
    CHECK ((estado = 'pendiente') = (resuelto_en IS NULL)) -- Solo los reportes resueltos tienen fecha de decisión
);

-- Alias único sin distinguir mayúsculas (ana y ANA son el mismo)
CREATE UNIQUE INDEX ON usuario (lower(alias));

-- Vasos creados por un usuario (pantalla Mis vasos)
CREATE INDEX ON vaso (autor_id);

-- Recetas de un autor (perfil y pestaña Siguiendo)
CREATE INDEX ON receta (autor_id);

-- Versiones derivadas de una receta (contador «4 versiones»)
CREATE INDEX ON receta (receta_origen_id);

-- Recetas publicadas de la más nueva a la más vieja (pestaña Nuevas)
CREATE INDEX ON receta (publicada_en DESC) WHERE publicada_en IS NOT NULL AND NOT oculta;

-- Búsqueda de recetas por nombre con texto completo en español
CREATE INDEX ON receta USING gin (to_tsvector('spanish', nombre));

-- Capas que usan un ingrediente (verificación al borrarlo del catálogo)
CREATE INDEX ON componente_receta (ingrediente_id);

-- Comentarios de una receta en orden cronológico
CREATE INDEX ON comentario (receta_id, creado_en);

-- Comentarios de un autor (borrado en cascada al eliminar la cuenta)
CREATE INDEX ON comentario (autor_id);

-- Respuestas de un comentario (hilo y borrado en cascada)
CREATE INDEX ON comentario (respuesta_a) WHERE respuesta_a IS NOT NULL;

-- Comentarios anclados a una capa (liberar el ancla al borrar la capa)
CREATE INDEX ON comentario (componente_id) WHERE componente_id IS NOT NULL;

-- Comentarios anclados a un paso (liberar el ancla al borrar el paso)
CREATE INDEX ON comentario (paso_id) WHERE paso_id IS NOT NULL;

-- Preparaciones de una receta (porcentaje «me salió»)
CREATE INDEX ON preparacion (receta_id);

-- Historial de preparaciones de un usuario (pantalla Actividad)
CREATE INDEX ON preparacion (usuario_id, creada_en DESC);

-- Usuarios que guardaron una receta (borrado en cascada al eliminar la receta)
CREATE INDEX ON guardado (receta_id);

-- Seguidores de un usuario (la llave primaria solo sirve para buscar por seguidor)
CREATE INDEX ON seguimiento (seguido_id);

-- Un usuario reporta una misma receta una sola vez
CREATE UNIQUE INDEX ON reporte (reportante_id, receta_id);

-- Un usuario reporta un mismo comentario una sola vez
CREATE UNIQUE INDEX ON reporte (reportante_id, comentario_id);

-- Reportes de una receta (borrado en cascada al eliminar la receta)
CREATE INDEX ON reporte (receta_id) WHERE receta_id IS NOT NULL;

-- Reportes de un comentario (borrado en cascada al eliminar el comentario)
CREATE INDEX ON reporte (comentario_id) WHERE comentario_id IS NOT NULL;

-- Cola de reportes sin resolver para el moderador
CREATE INDEX ON reporte (creado_en) WHERE estado = 'pendiente';

-- Crea el perfil al registrarse una cuenta en Supabase Auth con el alias y la declaración de edad que envía la app en los metadatos
CREATE FUNCTION crear_perfil()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER -- Se ejecuta como dueño para escribir en public.usuario desde el alta de auth.users
SET search_path = '' -- Evita resolver objetos de otros esquemas (recomendación de Supabase)
AS $$
BEGIN
    -- Insertar el perfil: sin alias (proveedor social) se asigna uno provisional a partir del id y sin declaración se toma como menor de edad
    INSERT INTO public.usuario (id, alias, mayor_de_edad)
    VALUES (NEW.id,
        coalesce(NULLIF(btrim(NEW.raw_user_meta_data ->> 'alias'), ''), 'usuario_' || left(NEW.id::text, 8)),
        coalesce(NEW.raw_user_meta_data ->> 'mayor_de_edad' = 'true', false) -- Solo el texto 'true' declara mayoría de edad
    );

    RETURN NEW; -- Continuar con el alta de la cuenta
END;
$$;

-- Impedir que la API pública llame a la función (solo la usa el trigger)
REVOKE EXECUTE ON FUNCTION crear_perfil() FROM PUBLIC, anon, authenticated;

-- Perfil creado junto con cada cuenta nueva (un alias repetido cancela el registro)
CREATE TRIGGER crear_perfil_al_registrar
AFTER INSERT ON auth.users
FOR EACH ROW
EXECUTE FUNCTION crear_perfil();

-- Bloquear el acceso directo por la API pública de Supabase (el backend entra como dueño y no le aplica RLS)
ALTER TABLE usuario ENABLE ROW LEVEL SECURITY;
ALTER TABLE vaso ENABLE ROW LEVEL SECURITY;
ALTER TABLE ingrediente ENABLE ROW LEVEL SECURITY;
ALTER TABLE receta ENABLE ROW LEVEL SECURITY;
ALTER TABLE componente_receta ENABLE ROW LEVEL SECURITY;
ALTER TABLE paso ENABLE ROW LEVEL SECURITY;
ALTER TABLE comentario ENABLE ROW LEVEL SECURITY;
ALTER TABLE preparacion ENABLE ROW LEVEL SECURITY;
ALTER TABLE guardado ENABLE ROW LEVEL SECURITY;
ALTER TABLE seguimiento ENABLE ROW LEVEL SECURITY;
ALTER TABLE reporte ENABLE ROW LEVEL SECURITY;
