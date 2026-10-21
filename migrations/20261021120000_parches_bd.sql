-- Parches de esquema sobre la base que crearon las dos migraciones anteriores.
-- Idempotente: se puede aplicar sobre una base que ya los tenga puestos a mano.

BEGIN;

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Cuenta: rol para moderar, biografía, preferencias y marca de edición
ALTER TABLE usuario
    ADD COLUMN IF NOT EXISTS bio text,
    ADD COLUMN IF NOT EXISTS rol text NOT NULL DEFAULT 'usuario',
    ADD COLUMN IF NOT EXISTS unidades text NOT NULL DEFAULT 'ml',
    ADD COLUMN IF NOT EXISTS tema text NOT NULL DEFAULT 'sistema',
    ADD COLUMN IF NOT EXISTS idioma text NOT NULL DEFAULT 'es',
    ADD COLUMN IF NOT EXISTS notificaciones_activas boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS actualizado_en timestamp with time zone NOT NULL DEFAULT now();

-- Catálogo: clave estable para la siembra y para el catálogo sin conexión
ALTER TABLE vaso
    ADD COLUMN IF NOT EXISTS clave text,
    ADD COLUMN IF NOT EXISTS forma text;
CREATE UNIQUE INDEX IF NOT EXISTS vaso_clave_unica ON vaso (clave) WHERE clave IS NOT NULL;

ALTER TABLE ingrediente
    ADD COLUMN IF NOT EXISTS clave text,
    ADD COLUMN IF NOT EXISTS tipo text;
CREATE UNIQUE INDEX IF NOT EXISTS ingrediente_clave_unica ON ingrediente (clave) WHERE clave IS NOT NULL;

-- Receta: familia, técnica, temperatura, dificultad, intensidad, tiempo, dosis, notas y etiquetas
ALTER TABLE receta
    ADD COLUMN IF NOT EXISTS familia text,
    ADD COLUMN IF NOT EXISTS tecnica text,
    ADD COLUMN IF NOT EXISTS temperatura text,
    ADD COLUMN IF NOT EXISTS dificultad smallint,
    ADD COLUMN IF NOT EXISTS intensidad smallint,
    ADD COLUMN IF NOT EXISTS minutos smallint,
    ADD COLUMN IF NOT EXISTS dosis_g double precision,
    ADD COLUMN IF NOT EXISTS notas text[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS etiquetas text[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS actualizado_en timestamp with time zone NOT NULL DEFAULT now();
ALTER TABLE receta ALTER COLUMN familia SET NOT NULL;

-- Paso, comentario y preparación
ALTER TABLE paso ADD COLUMN IF NOT EXISTS titulo text;
ALTER TABLE comentario ADD COLUMN IF NOT EXISTS foto_ruta text;
ALTER TABLE preparacion ADD COLUMN IF NOT EXISTS motivo text;

-- Ids que puede generar el servidor (la app sigue mandando los suyos)
ALTER TABLE vaso ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE ingrediente ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE receta ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE componente_receta ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE paso ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE comentario ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE preparacion ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE reporte ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- Restricciones que no admiten IF NOT EXISTS
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'usuario_rol_check' AND conrelid = 'usuario'::regclass) THEN
        ALTER TABLE usuario ADD CONSTRAINT usuario_rol_check CHECK (rol IN ('usuario', 'moderador'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'usuario_unidades_check' AND conrelid = 'usuario'::regclass) THEN
        ALTER TABLE usuario ADD CONSTRAINT usuario_unidades_check CHECK (unidades IN ('ml', 'oz', 'partes', 'cucharadas'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'usuario_tema_check' AND conrelid = 'usuario'::regclass) THEN
        ALTER TABLE usuario ADD CONSTRAINT usuario_tema_check CHECK (tema IN ('claro', 'oscuro', 'sistema'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'usuario_idioma_check' AND conrelid = 'usuario'::regclass) THEN
        ALTER TABLE usuario ADD CONSTRAINT usuario_idioma_check CHECK (idioma IN ('es', 'en'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_temperatura_check' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_temperatura_check CHECK (temperatura IN ('caliente', 'frio', 'mixto'));
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_dificultad_check' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_dificultad_check CHECK (dificultad BETWEEN 1 AND 5);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_intensidad_check' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_intensidad_check CHECK (intensidad BETWEEN 1 AND 5);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_minutos_check' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_minutos_check CHECK (minutos >= 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_dosis_g_check' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_dosis_g_check CHECK (dosis_g > 0);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_nombre_longitud' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_nombre_longitud CHECK (length(nombre) BETWEEN 1 AND 120);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_notas_sin_huecos' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_notas_sin_huecos CHECK (array_position(notas, NULL) IS NULL);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'receta_etiquetas_sin_huecos' AND conrelid = 'receta'::regclass) THEN
        ALTER TABLE receta ADD CONSTRAINT receta_etiquetas_sin_huecos CHECK (array_position(etiquetas, NULL) IS NULL);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'comentario_propuesta_con_ancla' AND conrelid = 'comentario'::regclass) THEN
        ALTER TABLE comentario ADD CONSTRAINT comentario_propuesta_con_ancla
            CHECK (propuesta IS NULL OR componente_id IS NOT NULL OR paso_id IS NOT NULL);
    END IF;
END $$;

-- Actividad, tokens push y registro de moderación
CREATE TABLE IF NOT EXISTS notificacion
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    usuario_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE,
    actor_id uuid REFERENCES usuario (id) ON DELETE SET NULL,
    tipo text NOT NULL CHECK (tipo IN ('respuesta', 'valoracion', 'version_derivada', 'reporte_resuelto')),
    receta_id uuid REFERENCES receta (id) ON DELETE CASCADE,
    comentario_id uuid REFERENCES comentario (id) ON DELETE CASCADE,
    leida_en timestamp with time zone,
    creada_en timestamp with time zone NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS dispositivo
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    usuario_id uuid NOT NULL REFERENCES usuario (id) ON DELETE CASCADE,
    token text NOT NULL UNIQUE,
    plataforma text NOT NULL CHECK (plataforma IN ('android', 'ios')),
    activo boolean NOT NULL DEFAULT true,
    creado_en timestamp with time zone NOT NULL DEFAULT now(),
    visto_en timestamp with time zone NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS moderacion_accion
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    reporte_id uuid REFERENCES reporte (id) ON DELETE SET NULL,
    moderador_id uuid REFERENCES usuario (id) ON DELETE SET NULL,
    accion text NOT NULL CHECK (accion IN ('ocultar', 'restaurar', 'avisar', 'descartar')),
    objetivo_tipo text NOT NULL CHECK (objetivo_tipo IN ('receta', 'comentario')),
    objetivo_id uuid NOT NULL,
    nota text,
    creada_en timestamp with time zone NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS receta_vaso_id_idx ON receta (vaso_id);
CREATE INDEX IF NOT EXISTS receta_familia_publicada_idx ON receta (familia, publicada_en DESC)
    WHERE publicada_en IS NOT NULL AND NOT oculta;
CREATE INDEX IF NOT EXISTS receta_etiquetas_idx ON receta USING gin (etiquetas);
CREATE INDEX IF NOT EXISTS receta_nombre_trgm_idx ON receta USING gin (nombre gin_trgm_ops);
CREATE INDEX IF NOT EXISTS ingrediente_tipo_idx ON ingrediente (tipo);
CREATE INDEX IF NOT EXISTS notificacion_bandeja_idx ON notificacion (usuario_id, creada_en DESC);
CREATE INDEX IF NOT EXISTS notificacion_sin_leer_idx ON notificacion (usuario_id) WHERE leida_en IS NULL;
CREATE INDEX IF NOT EXISTS dispositivo_usuario_idx ON dispositivo (usuario_id);
CREATE INDEX IF NOT EXISTS moderacion_accion_moderador_idx ON moderacion_accion (moderador_id);
CREATE INDEX IF NOT EXISTS moderacion_accion_reporte_idx ON moderacion_accion (reporte_id);

-- Coherencia que la base sostiene por sí sola
CREATE OR REPLACE FUNCTION receta_marca_actualizado() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.actualizado_en := now();
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION usuario_marca_actualizado() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.actualizado_en := now();
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION contenido_toca_receta() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        UPDATE receta SET actualizado_en = now() WHERE id = OLD.receta_id;
        RETURN OLD;
    END IF;
    UPDATE receta SET actualizado_en = now() WHERE id = NEW.receta_id;
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION comentario_limpia_propuesta() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF (NEW.componente_id IS NULL AND OLD.componente_id IS NOT NULL)
       OR (NEW.paso_id IS NULL AND OLD.paso_id IS NOT NULL) THEN
        NEW.propuesta := NULL;
    END IF;
    RETURN NEW;
END $$;

CREATE OR REPLACE FUNCTION receta_sin_ciclo() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    actual uuid := NEW.receta_origen_id;
    saltos integer := 0;
BEGIN
    WHILE actual IS NOT NULL LOOP
        IF actual = NEW.id THEN
            RAISE EXCEPTION 'derivación circular: la receta % descendería de sí misma', NEW.id;
        END IF;
        saltos := saltos + 1;
        IF saltos > 32 THEN
            RAISE EXCEPTION 'cadena de derivaciones demasiado larga para la receta %', NEW.id;
        END IF;
        SELECT receta_origen_id INTO actual FROM receta WHERE id = actual;
    END LOOP;
    RETURN NEW;
END $$;

DROP TRIGGER IF EXISTS receta_actualizado ON receta;
CREATE TRIGGER receta_actualizado BEFORE UPDATE ON receta
    FOR EACH ROW EXECUTE FUNCTION receta_marca_actualizado();

DROP TRIGGER IF EXISTS usuario_actualizado ON usuario;
CREATE TRIGGER usuario_actualizado BEFORE UPDATE ON usuario
    FOR EACH ROW EXECUTE FUNCTION usuario_marca_actualizado();

DROP TRIGGER IF EXISTS componente_toca_receta ON componente_receta;
CREATE TRIGGER componente_toca_receta AFTER INSERT OR UPDATE OR DELETE ON componente_receta
    FOR EACH ROW EXECUTE FUNCTION contenido_toca_receta();

DROP TRIGGER IF EXISTS paso_toca_receta ON paso;
CREATE TRIGGER paso_toca_receta AFTER INSERT OR UPDATE OR DELETE ON paso
    FOR EACH ROW EXECUTE FUNCTION contenido_toca_receta();

DROP TRIGGER IF EXISTS comentario_propuesta ON comentario;
CREATE TRIGGER comentario_propuesta BEFORE UPDATE ON comentario
    FOR EACH ROW EXECUTE FUNCTION comentario_limpia_propuesta();

DROP TRIGGER IF EXISTS receta_derivacion ON receta;
CREATE TRIGGER receta_derivacion BEFORE INSERT OR UPDATE OF receta_origen_id ON receta
    FOR EACH ROW WHEN (NEW.receta_origen_id IS NOT NULL) EXECUTE FUNCTION receta_sin_ciclo();

ALTER TABLE notificacion ENABLE ROW LEVEL SECURITY;
ALTER TABLE dispositivo ENABLE ROW LEVEL SECURITY;
ALTER TABLE moderacion_accion ENABLE ROW LEVEL SECURITY;

COMMIT;
