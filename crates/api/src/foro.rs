//! La Barra (D1-D7): publicar, hilo, comentar, valorar, reportar y fotos, parte compartida.
//!
//! Todas las server functions respetan la misma regla de visibilidad que la ficha: una
//! receta oculta, privada ajena o con alcohol para una cuenta sin mayoría de edad declarada
//! no tiene hilo ni valoraciones, y tampoco se puede comentar ni valorar. Las escrituras que
//! le importan a otra cuenta dejan su aviso en la misma transacción (`notificaciones`).

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta
#[cfg(feature = "server")]
use crate::notificaciones::Aviso;
#[cfg(feature = "server")]
use crate::notificaciones::destinatario; // Cuenta que recibe el aviso, si no es uno mismo
#[cfg(feature = "server")]
use crate::notificaciones::notificar; // Aviso dentro de la transacción // Datos del aviso

// CONSTANTES Y VARIABLES GLOBALES

/// Largo máximo de un comentario, igual que el CHECK de la tabla.
#[cfg(feature = "server")]
const LARGO_MAXIMO: usize = 2000;

// ESTRUCTURAS

/// Parte de la receta a la que se ancla un comentario, por su posición (0 = primera).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Ancla {
    /// Capa de la receta, en el orden de la ficha.
    Capa(i16),
    /// Paso de la preparación, en el orden de la ficha.
    Paso(i16),
}

/// Comentario tal como se pinta en el hilo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comentario {
    /// Identificador del comentario (UUID en texto).
    pub id: String,
    /// Alias de quien comenta.
    pub autor: String,
    /// Texto del comentario.
    pub texto: String,
    /// Fecha legible, por ejemplo «12 nov 2026».
    pub fecha: String,
    /// Comentario al que responde, si es una respuesta.
    pub respuesta_a: Option<String>,
    /// Nombre de la capa a la que se ancla, si el comentario va anclado a una capa.
    #[serde(default)]
    pub capa: Option<String>,
    /// Posición del paso al que se ancla, si va anclado a un paso.
    #[serde(default)]
    pub paso: Option<i16>,
    /// Título de ese paso.
    #[serde(default)]
    pub paso_titulo: Option<String>,
    /// Cantidad propuesta para el ancla, en su unidad (ml o segundos).
    #[serde(default)]
    pub propuesta: Option<f64>,
    /// Unidad de esa cantidad: «ml» o «s».
    #[serde(default)]
    pub propuesta_unidad: Option<String>,
    /// Si lo escribió la cuenta que pregunta.
    pub propio: bool,
    /// Ruta de la foto adjunta en Storage, si tiene.
    #[serde(default)]
    pub foto: Option<String>,
}

/// Comentario nuevo que manda la app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComentarioNuevo {
    /// UUID que genera la app: reenviarlo desde la cola no duplica el comentario.
    pub id: Option<String>,
    /// Receta comentada.
    pub receta_id: String,
    /// Texto del comentario (1-2000 caracteres).
    pub texto: String,
    /// Comentario al que responde, si es una respuesta.
    pub respuesta_a: Option<String>,
    /// Capa o paso al que se ancla, si se ancla.
    pub ancla: Option<Ancla>,
    /// Cantidad propuesta para el ancla: en la unidad de la capa, o segundos para un paso.
    pub propuesta: Option<f64>,
    /// Ruta de la foto que la app ya subió a Storage, dentro de la carpeta del comentario.
    #[serde(default)]
    pub foto: Option<String>,
}

/// Recuento de «me salió / no me salió» de una receta.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Valoraciones {
    /// Personas a las que les salió igual a la ficha.
    pub me_salio: i64,
    /// Personas a las que no les salió.
    pub no_me_salio: i64,
    /// Valoración de la cuenta que pregunta, si ya valoró.
    pub mia: Option<bool>,
}

/// Fila de un comentario del hilo (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaComentario {
    /// Identificador del comentario.
    id: String,
    /// Alias del autor.
    autor: String,
    /// Texto del comentario.
    texto: String,
    /// Comentario al que responde.
    respuesta_a: Option<String>,
    /// Nombre de la capa anclada.
    capa: Option<String>,
    /// Unidad de la capa anclada.
    unidad: Option<String>,
    /// Posición del paso anclado.
    paso_posicion: Option<i16>,
    /// Título del paso anclado.
    paso_titulo: Option<String>,
    /// Cantidad propuesta.
    propuesta: Option<f64>,
    /// Si lo escribió la cuenta que pregunta.
    propio: bool,
    /// Ruta de la foto adjunta.
    foto_ruta: Option<String>,
    /// Día de publicación.
    dia: i32,
    /// Mes de publicación (1-12).
    mes: i32,
    /// Año de publicación.
    anio: i32,
}

/// Fila del recuento de valoraciones (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaValoraciones {
    /// Valoraciones positivas.
    me_salio: i64,
    /// Valoraciones negativas.
    no_me_salio: i64,
    /// Valoración de la cuenta, si existe.
    mia: Option<bool>,
}

// FUNCIONES

/// Condición SQL que se cumple cuando la receta del alias `tabla` no lleva alcohol.
#[cfg(feature = "server")]
pub(crate) fn sin_alcohol(tabla: &str) -> String {
    format!(
        "NOT EXISTS (SELECT 1 FROM componente_receta alcohol WHERE alcohol.receta_id = {tabla}.id AND alcohol.graduacion_pct > 0)"
    )
}

/// Fecha en formato neutro (`2026-11-12`).
///
/// El servidor no sabe en qué idioma está la app, así que manda el dato tal cual y el cliente
/// lo escribe con el orden y los meses del idioma elegido (`i18n::fecha_iso`).
#[cfg(feature = "server")]
pub(crate) fn fecha_legible(dia: i32, mes: i32, anio: i32) -> String {
    format!("{anio:04}-{:02}-{:02}", mes.clamp(1, 12), dia.clamp(1, 31))
}

/// Indica si el nombre de archivo de una foto es seguro: letras, números, guion y punto.
pub fn nombre_foto_valido(nombre: &str) -> bool {
    !nombre.is_empty()
        && nombre.len() <= 80
        && nombre
            .chars()
            .all(|letra| letra.is_ascii_alphanumeric() || letra == '-' || letra == '.')
        && !nombre.contains("..")
}

/// Indica si la receta es visible para la cuenta, con la misma regla que la ficha.
#[cfg(feature = "server")]
async fn visible(
    pool: &sqlx::PgPool,
    receta_id: &str,
    cuenta: &Option<String>,
    adulto: bool,
) -> Result<bool, ServerFnError> {
    // Un id roto no puede llegar a `$1::uuid`: la app avisa de que el enlace no sirve
    if !crate::fallos::es_uuid(receta_id) {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde el recetario.",
        ));
    }

    // Consultar si la receta existe, no está oculta, es pública o propia y respeta la edad
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM receta r
            WHERE r.id = $1::uuid
              AND NOT r.oculta
              AND (r.publicada_en IS NOT NULL OR r.autor_id = $2::uuid)
              AND ($3 OR NOT EXISTS (
                    SELECT 1 FROM componente_receta alcohol
                    WHERE alcohol.receta_id = r.id AND alcohol.graduacion_pct > 0
                  ))
        )
        "#,
    )
    .bind(receta_id)
    .bind(cuenta)
    .bind(adulto)
    .fetch_one(pool)
    .con_plazo("la visibilidad de la receta")
    .await
}

/// Cuenta del token o el error que pide iniciar sesión.
#[cfg(feature = "server")]
pub(crate) async fn cuenta_obligatoria(token: &str) -> Result<String, ServerFnError> {
    crate::auth::cuenta(Some(token))
        .await
        .ok_or_else(|| crate::fallos::sesion("Entra con tu cuenta para hacer esto.")) // Token ausente o caducado
}

/// Recuento de valoraciones de una receta, contando la última valoración de cada persona.
#[cfg(feature = "server")]
async fn recuento(
    pool: &sqlx::PgPool,
    receta_id: &str,
    cuenta: &Option<String>,
) -> Result<Valoraciones, ServerFnError> {
    // Contar la valoración más reciente de cada persona
    let fila = sqlx::query_as::<_, FilaValoraciones>(
        r#"
        SELECT count(*) FILTER (WHERE ultima.me_salio) AS me_salio,
               count(*) FILTER (WHERE NOT ultima.me_salio) AS no_me_salio,
               bool_or(ultima.me_salio) FILTER (WHERE ultima.usuario_id = $2::uuid) AS mia
        FROM (
            SELECT DISTINCT ON (usuario_id) usuario_id, me_salio
            FROM preparacion
            WHERE receta_id = $1::uuid
            ORDER BY usuario_id, creada_en DESC
        ) ultima
        "#,
    )
    .bind(receta_id)
    .bind(cuenta)
    .fetch_one(pool)
    .con_plazo("el recuento de valoraciones")
    .await?;

    Ok(Valoraciones {
        me_salio: fila.me_salio,
        no_me_salio: fila.no_me_salio,
        mia: fila.mia,
    })
}

/// Publica una receta propia en La Barra o la retira (D1).
///
/// Devuelve si la receta quedó publicada. Una receta retirada por moderación no cambia.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la receta no es de la cuenta o si la
/// consulta falla.
#[server(endpoint = "publicar")]
pub async fn publicar(
    token: String,
    receta_id: String,
    publica: bool,
) -> Result<bool, ServerFnError> {
    if !crate::fallos::es_uuid(&receta_id) {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde Mi recetario.",
        ));
    }

    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let mut tx = pool.begin().con_plazo("la publicación").await?;

    // Leer el estado actual y a quién pertenece la receta original, si es una versión
    let previa = sqlx::query_as::<_, (bool, Option<String>)>(
        r#"
        SELECT r.publicada_en IS NOT NULL,
               (SELECT o.autor_id::text FROM receta o WHERE o.id = r.receta_origen_id)
        FROM receta r
        WHERE r.id = $1::uuid
          AND r.autor_id = $2::uuid
          AND NOT r.oculta
        FOR UPDATE OF r
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .fetch_optional(&mut *tx)
    .con_plazo("la publicación")
    .await?;
    let Some((estaba_publicada, autor_origen)) = previa else {
        return Err(crate::fallos::no_encontrado(
            "Esa receta no existe o no es tuya.",
        )); // No es suya o está oculta
    };

    // Cambiar la fecha de publicación
    sqlx::query(
        r#"
        UPDATE receta
        SET publicada_en = CASE WHEN $2 THEN COALESCE(publicada_en, now()) ELSE NULL END
        WHERE id = $1::uuid
        "#,
    )
    .bind(&receta_id)
    .bind(publica)
    .execute(&mut *tx)
    .con_plazo("la publicación")
    .await?;

    // Una versión recién publicada avisa a quien creó la receta original
    let mut push_para: Option<String> = None;
    if publica
        && !estaba_publicada
        && let Some(dueno) = autor_origen
            .as_deref()
            .and_then(|dueno| destinatario(dueno, &cuenta))
    {
        notificar(
            &mut tx,
            Aviso {
                para: dueno,
                actor: &cuenta,
                tipo: "version_derivada",
                receta_id: &receta_id,
                comentario_id: None,
            },
        )
        .await?;
        push_para = Some(dueno.to_string());
    }

    tx.commit().con_plazo("la publicación").await?;

    // El push va después del commit: si la escritura se deshace, no se avisa de nada
    if let Some(para) = push_para {
        crate::push::avisar(para, "version_derivada", receta_id.clone());
    }

    Ok(publica) // Estado final de la receta
}

/// Trae el hilo de comentarios de una receta, del más antiguo al más reciente (D2).
///
/// Sin permiso para ver la receta devuelve un hilo vacío. Los comentarios retirados por
/// moderación no se devuelven.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si la consulta falla.
#[server(endpoint = "hilo")]
pub async fn hilo(
    receta_id: String,
    token: Option<String>,
) -> Result<Vec<Comentario>, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(token.as_deref()).await;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Receta no visible: el hilo tampoco lo es
    if !visible(pool, &receta_id, &cuenta, adulto).await? {
        return Ok(Vec::new());
    }

    // Consultar los comentarios visibles con su autor y su ancla
    let filas_vec = sqlx::query_as::<_, FilaComentario>(
        r#"
        SELECT c.id::text AS id,
               COALESCE(u.alias, 'Anónimo') AS autor,
               c.texto,
               c.respuesta_a::text AS respuesta_a,
               capa.nombre AS capa,
               capa.unidad AS unidad,
               p.posicion AS paso_posicion,
               p.titulo AS paso_titulo,
               c.propuesta,
               COALESCE(c.autor_id = $2::uuid, false) AS propio,
               c.foto_ruta,
               EXTRACT(DAY FROM c.creado_en)::int AS dia,
               EXTRACT(MONTH FROM c.creado_en)::int AS mes,
               EXTRACT(YEAR FROM c.creado_en)::int AS anio
        FROM comentario c
        LEFT JOIN usuario u ON u.id = c.autor_id
        LEFT JOIN componente_receta capa ON capa.id = c.componente_id
        LEFT JOIN paso p ON p.id = c.paso_id
        WHERE c.receta_id = $1::uuid
          AND NOT c.oculto
        ORDER BY c.creado_en
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .fetch_all(pool)
    .con_plazo("el hilo")
    .await?;

    // Convertir cada fila en el comentario que pinta la app
    let comentarios_vec = filas_vec
        .into_iter()
        .map(|fila| {
            // El ancla viaja como datos: la app la escribe en su idioma
            let unidad = if fila.capa.is_some() {
                fila.unidad.clone().unwrap_or_default()
            } else {
                "s".to_string()
            };

            Comentario {
                id: fila.id,
                autor: fila.autor,
                texto: fila.texto,
                fecha: fecha_legible(fila.dia, fila.mes, fila.anio),
                respuesta_a: fila.respuesta_a,
                propuesta: fila
                    .propuesta
                    .filter(|_| fila.capa.is_some() || fila.paso_posicion.is_some()),
                propuesta_unidad: Some(unidad.to_string()),
                capa: fila.capa.clone(),
                paso: fila.paso_posicion,
                paso_titulo: fila.paso_titulo.clone(),
                propio: fila.propio,
                foto: fila.foto_ruta,
            }
        })
        .collect();

    Ok(comentarios_vec)
}

/// Publica un comentario en el hilo de una receta, anclado o no (D3).
///
/// Devuelve el identificador del comentario. Si la app reenvía el mismo `id` desde la cola,
/// no se crea otro ni se avisa dos veces. Un comentario nuevo avisa a quien publicó la receta
/// o, si es una respuesta, a quien escribió el comentario respondido.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si el texto está vacío o es demasiado largo,
/// si la receta no es visible para la cuenta, si el ancla, el comentario respondido o la foto
/// no son válidos, o si la escritura falla.
#[server(endpoint = "comentar")]
pub async fn comentar(token: String, nuevo: ComentarioNuevo) -> Result<String, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Validar el texto con el mismo límite que la tabla
    let texto = nuevo.texto.trim();
    if texto.is_empty() || texto.chars().count() > LARGO_MAXIMO {
        return Err(crate::fallos::datos(
            "El comentario necesita entre 1 y 2000 caracteres.",
        ));
    }

    // Validar la propuesta: positiva y solo con ancla
    if let Some(cantidad) = nuevo.propuesta
        && (!cantidad.is_finite() || cantidad <= 0.0 || nuevo.ancla.is_none())
    {
        return Err(crate::fallos::datos(
            "La cantidad propuesta tiene que ser un número mayor que cero.",
        ));
    }

    // Validar los identificadores que llegan de la app antes de mandarlos a la base
    if nuevo
        .id
        .as_deref()
        .is_some_and(|id| !crate::fallos::es_uuid(id))
    {
        return Err(crate::fallos::datos(
            "El comentario no es válido: vuelve a escribirlo.",
        ));
    }
    if nuevo
        .respuesta_a
        .as_deref()
        .is_some_and(|id| !crate::fallos::es_uuid(id))
    {
        return Err(crate::fallos::datos(
            "El comentario al que respondes no es válido: vuelve a abrir la receta.",
        ));
    }

    // Validar la foto: tiene que estar en la carpeta de la cuenta y de este comentario
    if let Some(ruta) = &nuevo.foto {
        let prefijo = format!(
            "{cuenta}/comentarios/{}/",
            nuevo.id.as_deref().unwrap_or_default()
        );
        if nuevo.id.is_none()
            || !nombre_foto_valido(ruta.strip_prefix(&prefijo).unwrap_or_default())
        {
            return Err(crate::fallos::datos(
                "La ruta de la foto no es válida: tiene que estar dentro de tu carpeta.",
            ));
        }
    }

    if !visible(pool, &nuevo.receta_id, &Some(cuenta.clone()), adulto).await? {
        return Err(crate::fallos::no_encontrado(
            "Esa receta no existe o ya no está disponible.",
        ));
    }

    // Comprobar que el comentario respondido sigue visible y es de esta receta
    if let Some(respondido) = &nuevo.respuesta_a {
        let existe = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM comentario WHERE id = $1::uuid AND receta_id = $2::uuid AND NOT oculto)",
        )
        .bind(respondido)
        .bind(&nuevo.receta_id)
        .fetch_one(pool)
        .con_plazo("el comentario respondido")
        .await?;
        if !existe {
            return Err(crate::fallos::datos(
                "Ese comentario ya no está disponible: comenta la receta.",
            ));
        }
    }

    // Resolver el ancla por su posición dentro de la receta
    let (componente_id, paso_id) = match nuevo.ancla {
        Some(Ancla::Capa(posicion)) => {
            let id = sqlx::query_scalar::<_, String>(
                "SELECT id::text FROM componente_receta WHERE receta_id = $1::uuid AND posicion = $2",
            )
            .bind(&nuevo.receta_id)
            .bind(posicion)
            .fetch_optional(pool)
            .con_plazo("la capa del comentario")
            .await?
            .ok_or_else(|| crate::fallos::datos("Esa capa ya no está en la receta: elige otra."))?;
            (Some(id), None) // Anclado a una capa
        }
        Some(Ancla::Paso(posicion)) => {
            let id = sqlx::query_scalar::<_, String>(
                "SELECT id::text FROM paso WHERE receta_id = $1::uuid AND posicion = $2",
            )
            .bind(&nuevo.receta_id)
            .bind(posicion)
            .fetch_optional(pool)
            .con_plazo("el paso del comentario")
            .await?
            .ok_or_else(|| crate::fallos::datos("Ese paso ya no está en la receta: elige otro."))?;
            (None, Some(id)) // Anclado a un paso
        }
        None => (None, None), // Comentario general
    };

    let mut tx = pool.begin().con_plazo("el comentario").await?;

    // Insertar el comentario y leer a quién hay que avisar; un reenvío con el mismo id no hace nada
    let creado = sqlx::query_as::<_, (String, Option<String>)>(
        r#"
        INSERT INTO comentario (id, receta_id, autor_id, respuesta_a, texto, componente_id, paso_id, propuesta, foto_ruta)
        VALUES (COALESCE($1::uuid, gen_random_uuid()), $2::uuid, $3::uuid, $4::uuid, $5, $6::uuid, $7::uuid, $8, $9)
        ON CONFLICT (id) DO NOTHING
        RETURNING id::text,
                  COALESCE(
                      (SELECT padre.autor_id FROM comentario padre WHERE padre.id = comentario.respuesta_a),
                      (SELECT r.autor_id FROM receta r WHERE r.id = comentario.receta_id)
                  )::text
        "#,
    )
    .bind(&nuevo.id)
    .bind(&nuevo.receta_id)
    .bind(&cuenta)
    .bind(&nuevo.respuesta_a)
    .bind(texto)
    .bind(&componente_id)
    .bind(&paso_id)
    .bind(nuevo.propuesta)
    .bind(&nuevo.foto)
    .fetch_optional(&mut *tx)
    .con_plazo("el comentario")
    .await?;

    // Comentario nuevo de otra persona: avisar al dueño de la receta o del comentario respondido
    let mut push_para: Option<String> = None;
    if let Some((id, dueno)) = &creado
        && let Some(para) = dueno
            .as_deref()
            .and_then(|dueno| destinatario(dueno, &cuenta))
    {
        notificar(
            &mut tx,
            Aviso {
                para,
                actor: &cuenta,
                tipo: "respuesta",
                receta_id: &nuevo.receta_id,
                comentario_id: Some(id),
            },
        )
        .await?;
        push_para = Some(para.to_string());
    }

    tx.commit().con_plazo("el comentario").await?;

    // El push va después del commit: si el comentario no se guardó, no se avisa
    if let Some(para) = push_para {
        crate::push::avisar(para, "respuesta", nuevo.receta_id.clone());
    }

    creado.map(|(id, _)| id).or(nuevo.id).ok_or_else(|| {
        crate::fallos::reintento("No se pudo guardar el comentario. Vuelve a intentarlo.")
    }) // Id nuevo o el del reenvío
}

/// Trae el recuento de «me salió / no me salió» de una receta (D4).
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si la consulta falla.
#[server(endpoint = "valoraciones")]
pub async fn valoraciones(
    receta_id: String,
    token: Option<String>,
) -> Result<Valoraciones, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta(token.as_deref()).await;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Receta no visible: sin valoraciones
    if !visible(pool, &receta_id, &cuenta, adulto).await? {
        return Ok(Valoraciones::default());
    }

    recuento(pool, &receta_id, &cuenta).await
}

/// Registra si a la cuenta le salió la receta; una nueva valoración reemplaza la anterior (D4).
///
/// Devuelve el recuento actualizado. Quien publicó la receta no puede valorarla.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la receta no es visible, si es propia o si
/// la consulta falla.
#[server(endpoint = "valorar")]
pub async fn valorar(
    token: String,
    receta_id: String,
    me_salio: bool,
) -> Result<Valoraciones, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    if !visible(pool, &receta_id, &Some(cuenta.clone()), adulto).await? {
        return Err(crate::fallos::no_encontrado(
            "Esa receta no existe o ya no está disponible.",
        ));
    }

    let mut tx = pool.begin().con_plazo("la valoración").await?;

    // Rechazar la valoración de la propia receta
    let autor =
        sqlx::query_scalar::<_, String>("SELECT autor_id::text FROM receta WHERE id = $1::uuid")
            .bind(&receta_id)
            .fetch_one(&mut *tx)
            .con_plazo("la receta valorada")
            .await?;
    let Some(para) = destinatario(&autor, &cuenta) else {
        return Err(crate::fallos::datos("No puedes valorar tu propia receta."));
    };

    // Actualizar la última valoración de la cuenta o crear la primera
    sqlx::query(
        r#"
        WITH previa AS (
            UPDATE preparacion
            SET me_salio = $3, creada_en = now()
            WHERE id = (
                SELECT id FROM preparacion
                WHERE usuario_id = $2::uuid AND receta_id = $1::uuid
                ORDER BY creada_en DESC
                LIMIT 1
            )
            RETURNING id
        )
        INSERT INTO preparacion (usuario_id, receta_id, me_salio)
        SELECT $2::uuid, $1::uuid, $3
        WHERE NOT EXISTS (SELECT 1 FROM previa)
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .bind(me_salio)
    .execute(&mut *tx)
    .con_plazo("tu valoración")
    .await?;

    // Avisar a quien publicó la receta
    notificar(
        &mut tx,
        Aviso {
            para,
            actor: &cuenta,
            tipo: "valoracion",
            receta_id: &receta_id,
            comentario_id: None,
        },
    )
    .await?;

    tx.commit().con_plazo("tu valoración").await?;

    // El push va después del commit: si la valoración se deshace, no se avisa
    crate::push::avisar(para.to_string(), "valoracion", receta_id.clone());

    recuento(pool, &receta_id, &Some(cuenta)).await
}

// REPORTES (D7)

/// Motivo de un reporte, igual que el CHECK de la tabla.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Motivo {
    /// Publicidad o contenido repetido.
    Spam,
    /// Insultos o acoso.
    Ofensivo,
    /// Una preparación que pone en riesgo a alguien.
    Peligroso,
    /// Alcohol dirigido a menores de edad.
    Menores,
    /// Cualquier otra causa, explicada en el detalle.
    Otro,
}

impl Motivo {
    /// Los motivos en el orden en que se ofrecen.
    pub const TODOS: [Motivo; 5] = [
        Motivo::Spam,
        Motivo::Ofensivo,
        Motivo::Peligroso,
        Motivo::Menores,
        Motivo::Otro,
    ];

    /// Clave del motivo en la base.
    pub fn clave(self) -> &'static str {
        match self {
            Motivo::Spam => "spam",
            Motivo::Ofensivo => "ofensivo",
            Motivo::Peligroso => "peligroso",
            Motivo::Menores => "menores",
            Motivo::Otro => "otro",
        }
    }

    /// Motivo que corresponde a una clave de la base, si existe.
    pub fn de_clave(clave: &str) -> Option<Motivo> {
        Motivo::TODOS
            .into_iter()
            .find(|motivo| motivo.clave() == clave)
    }

    /// Texto visible del motivo.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Motivo::Spam => "Spam o publicidad",
            Motivo::Ofensivo => "Ofensivo",
            Motivo::Peligroso => "Peligroso",
            Motivo::Menores => "Alcohol para menores",
            Motivo::Otro => "Otro motivo",
        }
    }
}

/// Reporte de una receta o de un comentario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReporteNuevo {
    /// Receta reportada, o `None` si se reporta un comentario.
    pub receta_id: Option<String>,
    /// Comentario reportado, o `None` si se reporta una receta.
    pub comentario_id: Option<String>,
    /// Causa elegida.
    pub motivo: Motivo,
    /// Explicación libre, opcional.
    pub detalle: Option<String>,
}

/// Reporta una receta o un comentario para la cola de moderación (D7).
///
/// Cada cuenta reporta una sola vez cada contenido: devuelve `true` si el reporte es nuevo
/// y `false` si la cuenta ya lo había reportado, sin crear otro.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si no se indica exactamente un contenido, si
/// ese contenido no es visible para la cuenta o si la consulta falla.
#[server(endpoint = "reportar")]
pub async fn reportar(token: String, reporte: ReporteNuevo) -> Result<bool, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Averiguar la receta del contenido para comprobar que la cuenta puede verlo
    let receta_id = match (&reporte.receta_id, &reporte.comentario_id) {
        (Some(receta_id), None) => receta_id.clone(), // Se reporta una receta
        (None, Some(comentario_id)) if !crate::fallos::es_uuid(comentario_id) => {
            return Err(crate::fallos::datos(
                "Ese comentario no es válido: vuelve a abrir la receta.",
            ));
        }
        (None, Some(comentario_id)) => sqlx::query_scalar::<_, String>(
            "SELECT receta_id::text FROM comentario WHERE id = $1::uuid AND NOT oculto",
        )
        .bind(comentario_id)
        .fetch_optional(pool)
        .con_plazo("el comentario reportado")
        .await?
        .ok_or_else(|| crate::fallos::no_encontrado("Ese comentario ya no está disponible."))?,
        _ => {
            return Err(crate::fallos::datos(
                "Reporta una sola cosa: la receta o un comentario.",
            ));
        }
    };

    if !visible(pool, &receta_id, &Some(cuenta.clone()), adulto).await? {
        return Err(crate::fallos::no_encontrado(
            "Esa receta no existe o ya no está disponible.",
        ));
    }

    let detalle = reporte
        .detalle
        .as_deref()
        .map(str::trim)
        .filter(|texto| !texto.is_empty())
        .map(|texto| texto.chars().take(LARGO_MAXIMO).collect::<String>()); // Detalle limpio y acotado

    // Insertar el reporte salvo que la cuenta ya haya reportado ese contenido
    let creado = sqlx::query(
        r#"
        INSERT INTO reporte (reportante_id, receta_id, comentario_id, motivo, detalle)
        SELECT $1::uuid, $2::uuid, $3::uuid, $4, $5
        WHERE NOT EXISTS (
            SELECT 1 FROM reporte
            WHERE reportante_id = $1::uuid
              AND receta_id IS NOT DISTINCT FROM $2::uuid
              AND comentario_id IS NOT DISTINCT FROM $3::uuid
        )
        "#,
    )
    .bind(&cuenta)
    .bind(&reporte.receta_id)
    .bind(&reporte.comentario_id)
    .bind(reporte.motivo.clave())
    .bind(&detalle)
    .execute(pool)
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo guardar el reporte: {causa}");

        crate::fallos::reintento("No se pudo guardar el reporte. Vuelve a intentarlo.")
    })?;

    Ok(creado.rows_affected() > 0) // `false`: ya lo había reportado
}

// FOTOS (D1)

/// Dirección del proyecto y clave pública de Supabase Storage.
///
/// Las dos son públicas por diseño (la clave anónima solo abre lo que las políticas de
/// Storage permiten), así que la app las pide al servidor en vez de llevarlas compiladas:
/// así las fotos funcionan en cualquier compilación, sin depender de variables de entorno
/// del teléfono.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatosStorage {
    /// Dirección del proyecto, sin barra final.
    pub url: String,
    /// Clave pública del proyecto.
    pub anon: String,
}

/// Datos para subir fotos a Storage, si el servidor tiene Supabase configurado.
///
/// Devuelve `None` cuando el despliegue no configuró `SUPABASE_URL` y `SUPABASE_ANON_KEY`.
#[server(endpoint = "datos_storage")]
pub async fn datos_storage() -> Result<Option<DatosStorage>, ServerFnError> {
    Ok(crate::auth::datos_publicos())
}

/// Fotos que admite una receta, igual que el CHECK de la tabla.
pub const FOTOS_MAXIMAS: usize = 5;

/// Registra en la receta la ruta de una foto que la app ya subió a Storage (D1).
///
/// La foto debe estar en la carpeta `{cuenta}/{receta}/` del bucket, que es la única donde la
/// política de Storage deja subir a la cuenta. Devuelve las rutas de la receta; registrar dos
/// veces la misma ruta no la duplica.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si la ruta no es de la cuenta y la receta, si la
/// receta no es suya, si ya tiene cinco fotos o si la consulta falla.
#[server(endpoint = "agregar_foto")]
pub async fn agregar_foto(
    token: String,
    receta_id: String,
    ruta: String,
) -> Result<Vec<String>, ServerFnError> {
    if !crate::fallos::es_uuid(&receta_id) {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde Mi recetario.",
        ));
    }

    let pool = crate::base::pool()?;
    let cuenta = cuenta_obligatoria(&token).await?;

    // Aceptar solo una ruta dentro de la carpeta de la cuenta y la receta
    let prefijo = format!("{cuenta}/{receta_id}/");
    if !nombre_foto_valido(ruta.strip_prefix(&prefijo).unwrap_or_default()) {
        return Err(crate::fallos::datos(
            "La ruta de la foto no es válida: tiene que estar dentro de tu carpeta.",
        ));
    }

    // Agregar la ruta si la receta es de la cuenta, no la tiene ya y no llegó al máximo
    let rutas = sqlx::query_scalar::<_, Vec<String>>(
        r#"
        UPDATE receta
        SET fotos_rutas = CASE
                WHEN $3 = ANY(fotos_rutas) THEN fotos_rutas
                ELSE array_append(fotos_rutas, $3)
            END
        WHERE id = $1::uuid
          AND autor_id = $2::uuid
          AND ($3 = ANY(fotos_rutas) OR cardinality(fotos_rutas) < $4)
        RETURNING fotos_rutas
        "#,
    )
    .bind(&receta_id)
    .bind(&cuenta)
    .bind(&ruta)
    .bind(FOTOS_MAXIMAS as i32)
    .fetch_optional(pool)
    .await
    .map_err(|causa| {
        dioxus::logger::tracing::error!("no se pudo registrar la foto: {causa}");

        crate::fallos::reintento("No se pudo registrar la foto en la receta. Vuelve a intentarlo.")
    })?;

    rutas.ok_or_else(|| crate::fallos::datos("La receta no es tuya o ya tiene cinco fotos.")) // No es suya o ya tiene cinco
}
