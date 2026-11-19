//! Muro de La Barra (D2): lo que publica y conversa la comunidad, parte compartida.
//!
//! El muro lista las recetas publicadas con sus contadores (comentarios, «me salió» y
//! versiones) y la conversación lista los últimos comentarios de todas las recetas. Las dos
//! consultas aplican la regla de edad y dejan fuera lo retirado por moderación.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

use crate::biblioteca::RecetaResumen; // Receta con vaso y capas para la miniatura

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta
#[cfg(feature = "server")]
use crate::biblioteca::FilaCapa; // Capa de una receta
#[cfg(feature = "server")]
use crate::biblioteca::FilaReceta; // Cabecera de una receta con su vaso

// CONSTANTES Y VARIABLES GLOBALES

/// Elementos que trae cada página del muro y de la conversación.
pub const POR_PAGINA: u32 = 20;

/// Última página que se puede pedir: acota el desplazamiento de la consulta.
#[cfg(feature = "server")]
const PAGINA_MAXIMA: u32 = 500;

/// Recetas publicadas que muestra un perfil.
#[cfg(feature = "server")]
const RECETAS_POR_PERFIL: u32 = 100;

/// Caracteres del comentario que se muestran en la conversación.
const LARGO_RESUMEN: usize = 140;

// ESTRUCTURAS

/// Pestaña del muro: decide qué recetas salen y en qué orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pestana {
    /// Todas las recetas publicadas, de la más nueva a la más vieja.
    Recientes,
    /// Solo las recetas de las cuentas que sigue quien pregunta.
    Siguiendo,
    /// Las recetas con más preparaciones y comentarios en los últimos 30 días.
    Populares,
}

impl Pestana {
    /// Clave de la pestaña dentro de la consulta.
    pub fn clave(self) -> &'static str {
        match self {
            Pestana::Recientes => "recientes",
            Pestana::Siguiendo => "siguiendo",
            Pestana::Populares => "populares",
        }
    }

    /// Orden de la consulta: siempre termina en fecha e id para que la paginación sea estable.
    pub fn orden_sql(self) -> &'static str {
        match self {
            Pestana::Populares => "c.recientes + p.recientes DESC, r.publicada_en DESC, r.id",
            Pestana::Recientes | Pestana::Siguiendo => "r.publicada_en DESC, r.id",
        }
    }
}

/// Receta publicada tal como se pinta en el muro y en los perfiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TarjetaBarra {
    /// Receta con su vaso y sus capas.
    pub receta: RecetaResumen,
    /// Si deriva de otra receta, aunque esa otra no sea visible para quien pregunta.
    pub derivada: bool,
    /// Nombre de la receta de la que deriva, si es visible.
    pub origen: Option<String>,
    /// Ruta de la primera foto en Storage, si tiene.
    pub foto: Option<String>,
    /// Comentarios visibles del hilo.
    pub comentarios: i64,
    /// Personas a las que les salió igual a la ficha.
    pub me_salio: i64,
    /// Personas que dijeron si les salió.
    pub valoraciones: i64,
    /// Versiones publicadas que derivan de esta receta.
    pub versiones: i64,
    /// Segundos desde que se publicó.
    pub hace_s: i64,
}

impl TarjetaBarra {
    /// Porcentaje de personas a las que les salió; `None` si nadie la ha valorado.
    pub fn porcentaje_me_salio(&self) -> Option<i64> {
        porcentaje(self.me_salio, self.valoraciones)
    }
}

/// Comentario de cualquier receta tal como se pinta en la conversación.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComentarioMuro {
    /// Identificador del comentario (UUID en texto).
    pub id: String,
    /// Alias de quien comenta.
    pub autor: String,
    /// Receta comentada.
    pub receta_id: String,
    /// Nombre de la receta comentada.
    pub receta_nombre: String,
    /// Texto del comentario, recortado a 140 caracteres.
    pub texto: String,
    /// Texto del ancla, por ejemplo «Capa · Leche» o «Paso 2 · Espumar».
    /// Nombre de la capa a la que se ancla, si va anclado a una capa.
    #[serde(default)]
    pub capa: Option<String>,
    /// Posición del paso al que se ancla, si va anclado a un paso.
    #[serde(default)]
    pub paso: Option<i16>,
    /// Título de ese paso.
    #[serde(default)]
    pub paso_titulo: Option<String>,
    /// Si responde a otro comentario.
    pub es_respuesta: bool,
    /// Segundos desde que se escribió.
    pub hace_s: i64,
}

/// Página de una lista: sus elementos y si quedan más por pedir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pagina<T> {
    /// Elementos de esta página, en orden.
    pub elementos: Vec<T>,
    /// Si la página siguiente trae algo.
    pub hay_mas: bool,
}

/// Fila de una tarjeta del muro (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaTarjeta {
    /// Campos compartidos con el catálogo público.
    #[sqlx(flatten)]
    cabecera: FilaReceta,
    /// Identificador de la receta, para pedir sus capas.
    tarjeta_id: String,
    /// Si deriva de otra receta.
    derivada: bool,
    /// Nombre de la receta de la que deriva, si es visible.
    origen_nombre: Option<String>,
    /// Primera foto de la receta.
    foto: Option<String>,
    /// Comentarios visibles.
    comentarios: i64,
    /// Valoraciones positivas.
    me_salio: i64,
    /// Valoraciones totales.
    valoraciones: i64,
    /// Versiones publicadas derivadas.
    versiones: i64,
    /// Segundos desde la publicación.
    hace_s: i64,
}

/// Fila de un comentario de la conversación (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaComentarioMuro {
    /// Identificador del comentario.
    id: String,
    /// Alias del autor.
    autor: String,
    /// Receta comentada.
    receta_id: String,
    /// Nombre de la receta comentada.
    receta_nombre: String,
    /// Texto completo del comentario.
    texto: String,
    /// Nombre de la capa anclada.
    capa: Option<String>,
    /// Posición del paso anclado.
    paso_posicion: Option<i16>,
    /// Título del paso anclado.
    paso_titulo: Option<String>,
    /// Si responde a otro comentario.
    es_respuesta: bool,
    /// Segundos desde que se escribió.
    hace_s: i64,
}

// FUNCIONES

/// Porcentaje entero de `parte` sobre `total`; `None` si el total es cero.
pub fn porcentaje(parte: i64, total: i64) -> Option<i64> {
    (total > 0).then(|| (parte.clamp(0, total) * 100 + total / 2) / total) // Redondeo al entero más cercano
}

/// Recorta un texto a `LARGO_RESUMEN` caracteres y marca el corte con puntos suspensivos.
pub fn resumen(texto: &str) -> String {
    let limpio = texto.split_whitespace().collect::<Vec<_>>().join(" "); // Sin saltos de línea
    if limpio.chars().count() <= LARGO_RESUMEN {
        return limpio; // Cabe completo
    }

    let corte: String = limpio.chars().take(LARGO_RESUMEN - 1).collect();
    format!("{}…", corte.trim_end()) // Texto cortado
}

/// Tiempo transcurrido en palabras: «hace un momento», «hace 5 min», «hace 3 h», «hace 2 d».
pub fn hace(segundos: i64) -> i64 {
    // El cliente decide cómo se escribe: aquí solo viaja el dato
    segundos.max(0)
}

/// Filas que se saltan para llegar a una página.
#[cfg(feature = "server")]
fn desplazamiento(pagina: u32) -> i64 {
    i64::from(pagina.min(PAGINA_MAXIMA)) * i64::from(POR_PAGINA)
}

/// Trae las tarjetas de una pestaña del muro o las recetas publicadas de un alias.
///
/// `alias` limita la lista a un autor (perfil); `limite` y `salto` paginan. Hace dos
/// consultas en total: las cabeceras con sus contadores y las capas de esas recetas.
#[cfg(feature = "server")]
async fn tarjetas(
    pool: &sqlx::PgPool,
    cuenta: &Option<String>,
    adulto: bool,
    pestana: Pestana,
    alias: Option<&str>,
    limite: i64,
    salto: i64,
) -> Result<Vec<TarjetaBarra>, ServerFnError> {
    // Consultar las recetas con sus contadores en una sola pasada
    let consulta = format!(
        r#"
        SELECT r.id::text AS id,
               r.nombre,
               r.tipo,
               r.familia,
               u.alias AS autor,
               COALESCE(r.capacidad_ml, v.capacidad_ml) AS capacidad_ml,
               v.capacidad_ml AS vaso_capacidad_ml,
               v.radio_boca,
               v.radio_fondo,
               v.altura,
               v.pared,
               v.borde,
               v.tallo,
               v.asa,
               v.plato,
               r.minutos,
               r.dificultad,
               r.intensidad,
               r.id::text AS tarjeta_id,
               (r.receta_origen_id IS NOT NULL) AS derivada,
               o.nombre AS origen_nombre,
               r.fotos_rutas[1] AS foto,
               c.total AS comentarios,
               p.me_salio AS me_salio,
               p.total AS valoraciones,
               (
                   SELECT count(*) FROM receta d
                   WHERE d.receta_origen_id = r.id
                     AND d.publicada_en IS NOT NULL
                     AND NOT d.oculta
                     AND ($1 OR {sin_alcohol_d})
               ) AS versiones,
               GREATEST(EXTRACT(EPOCH FROM now() - r.publicada_en), 0)::bigint AS hace_s
        FROM receta r
        JOIN vaso v ON v.id = r.vaso_id
        JOIN usuario u ON u.id = r.autor_id
        LEFT JOIN receta o ON o.id = r.receta_origen_id
          AND o.publicada_en IS NOT NULL
          AND NOT o.oculta
          AND ($1 OR {sin_alcohol_o})
        CROSS JOIN LATERAL (
            SELECT count(*) AS total,
                   count(*) FILTER (WHERE x.creado_en > now() - interval '30 days') AS recientes
            FROM comentario x
            WHERE x.receta_id = r.id AND NOT x.oculto
        ) c
        CROSS JOIN LATERAL (
            SELECT count(*) AS total,
                   count(*) FILTER (WHERE ultima.me_salio) AS me_salio,
                   count(*) FILTER (WHERE ultima.creada_en > now() - interval '30 days') AS recientes
            FROM (
                SELECT DISTINCT ON (usuario_id) me_salio, creada_en
                FROM preparacion
                WHERE receta_id = r.id
                ORDER BY usuario_id, creada_en DESC
            ) ultima
        ) p
        WHERE r.publicada_en IS NOT NULL
          AND NOT r.oculta
          AND ($1 OR {sin_alcohol_r})
          AND ($3 <> 'siguiendo' OR EXISTS (
                SELECT 1 FROM seguimiento s
                WHERE s.seguidor_id = $2::uuid AND s.seguido_id = r.autor_id
              ))
          AND ($4::text IS NULL OR lower(u.alias) = lower($4))
        ORDER BY {orden}
        LIMIT $5 OFFSET $6
        "#,
        sin_alcohol_d = crate::foro::sin_alcohol("d"),
        sin_alcohol_o = crate::foro::sin_alcohol("o"),
        sin_alcohol_r = crate::foro::sin_alcohol("r"),
        orden = pestana.orden_sql(),
    );
    let filas_vec = sqlx::query_as::<_, FilaTarjeta>(&consulta)
        .bind(adulto)
        .bind(cuenta)
        .bind(pestana.clave())
        .bind(alias)
        .bind(limite)
        .bind(salto)
        .fetch_all(pool)
        .con_plazo("el muro de La Barra")
        .await?;

    // Consultar las capas de esas recetas para dibujar cada vaso
    let ids_vec: Vec<String> = filas_vec
        .iter()
        .map(|fila| fila.tarjeta_id.clone())
        .collect();
    let capas_vec = sqlx::query_as::<_, FilaCapa>(
        r#"
        SELECT receta_id::text AS receta_id,
               ingrediente_id::text AS ingrediente_id,
               posicion,
               nombre,
               unidad,
               cantidad,
               graduacion_pct,
               color,
               dilucion
        FROM componente_receta
        WHERE receta_id = ANY($1::text[]::uuid[])
        ORDER BY receta_id, posicion
        "#,
    )
    .bind(&ids_vec[..])
    .fetch_all(pool)
    .con_plazo("las capas del muro")
    .await?;

    // Convertir cada fila en la tarjeta que pinta la app
    let tarjetas_vec = filas_vec
        .into_iter()
        .map(|fila| TarjetaBarra {
            receta: fila.cabecera.a_resumen(&capas_vec),
            derivada: fila.derivada,
            origen: fila.origen_nombre,
            foto: fila.foto,
            comentarios: fila.comentarios,
            me_salio: fila.me_salio,
            valoraciones: fila.valoraciones,
            versiones: fila.versiones,
            hace_s: fila.hace_s,
        })
        .collect();

    Ok(tarjetas_vec)
}

/// Recetas publicadas por un alias que la cuenta puede ver, de la más nueva a la más vieja.
///
/// # Errores
///
/// Devuelve `ServerFnError` si alguna consulta falla.
#[cfg(feature = "server")]
pub(crate) async fn tarjetas_de(
    pool: &sqlx::PgPool,
    cuenta: &Option<String>,
    adulto: bool,
    alias: &str,
) -> Result<Vec<TarjetaBarra>, ServerFnError> {
    tarjetas(
        pool,
        cuenta,
        adulto,
        Pestana::Recientes,
        Some(alias),
        i64::from(RECETAS_POR_PERFIL),
        0,
    )
    .await
}

/// Trae una página del muro de La Barra (D2).
///
/// `Siguiendo` sin sesión devuelve una página vacía. Sin mayoría de edad declarada no salen
/// las recetas con alcohol, y nunca salen las retiradas por moderación.
///
/// # Errores
///
/// Devuelve [`crate::fallos::CODIGO_SESION`] si llega un token vencido, y
/// [`crate::fallos::CODIGO_REINTENTO`] si la base no contesta. `pagina` se recorta al máximo
/// en lugar de rechazarse: el cliente nunca la inventa y el tope protege la consulta.
#[server(endpoint = "muro")]
pub async fn muro(
    token: Option<String>,
    pestana: Pestana,
    pagina: u32,
) -> Result<Pagina<TarjetaBarra>, ServerFnError> {
    let pool = crate::base::pool()?;
    let cuenta = crate::auth::cuenta_estricta(token.as_deref()).await?;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Pedir una fila de más para saber si queda otra página
    let mut elementos = tarjetas(
        pool,
        &cuenta,
        adulto,
        pestana,
        None,
        i64::from(POR_PAGINA) + 1,
        desplazamiento(pagina),
    )
    .await?;
    let hay_mas = elementos.len() > POR_PAGINA as usize;
    elementos.truncate(POR_PAGINA as usize);

    Ok(Pagina { elementos, hay_mas })
}

/// Trae una página con los últimos comentarios de toda la comunidad (D2).
///
/// Solo salen comentarios visibles de recetas publicadas que la cuenta puede ver.
///
/// # Errores
///
/// Devuelve [`crate::fallos::CODIGO_SESION`] si llega un token vencido, y
/// [`crate::fallos::CODIGO_REINTENTO`] si la base no contesta.
#[server(endpoint = "conversacion")]
pub async fn conversacion(
    token: Option<String>,
    pagina: u32,
) -> Result<Pagina<ComentarioMuro>, ServerFnError> {
    let pool = crate::base::pool()?;
    // La conversación es pública, pero un token vencido se avisa en vez de ignorarse
    crate::auth::cuenta_estricta(token.as_deref()).await?;
    let adulto = crate::auth::es_adulto(token.as_deref()).await;

    // Consultar los comentarios más recientes con su receta y su ancla
    let consulta = format!(
        r#"
        SELECT c.id::text AS id,
               COALESCE(u.alias, 'Anónimo') AS autor,
               r.id::text AS receta_id,
               r.nombre AS receta_nombre,
               c.texto,
               capa.nombre AS capa,
               p.posicion AS paso_posicion,
               p.titulo AS paso_titulo,
               (c.respuesta_a IS NOT NULL) AS es_respuesta,
               GREATEST(EXTRACT(EPOCH FROM now() - c.creado_en), 0)::bigint AS hace_s
        FROM comentario c
        JOIN receta r ON r.id = c.receta_id
        LEFT JOIN usuario u ON u.id = c.autor_id
        LEFT JOIN componente_receta capa ON capa.id = c.componente_id
        LEFT JOIN paso p ON p.id = c.paso_id
        WHERE NOT c.oculto
          AND r.publicada_en IS NOT NULL
          AND NOT r.oculta
          AND ($1 OR {sin_alcohol})
        ORDER BY c.creado_en DESC, c.id
        LIMIT $2 OFFSET $3
        "#,
        sin_alcohol = crate::foro::sin_alcohol("r"),
    );
    let mut filas_vec = sqlx::query_as::<_, FilaComentarioMuro>(&consulta)
        .bind(adulto)
        .bind(i64::from(POR_PAGINA) + 1) // Una fila de más para saber si queda otra página
        .bind(desplazamiento(pagina))
        .fetch_all(pool)
        .con_plazo("la conversación de La Barra")
        .await?;
    let hay_mas = filas_vec.len() > POR_PAGINA as usize;
    filas_vec.truncate(POR_PAGINA as usize);

    // Convertir cada fila en el comentario que pinta la app
    let elementos = filas_vec
        .into_iter()
        .map(|fila| ComentarioMuro {
            capa: fila.capa,
            paso: fila.paso_posicion,
            paso_titulo: fila.paso_titulo,
            id: fila.id,
            autor: fila.autor,
            receta_id: fila.receta_id,
            receta_nombre: fila.receta_nombre,
            texto: resumen(&fila.texto),
            es_respuesta: fila.es_respuesta,
            hace_s: fila.hace_s,
        })
        .collect();

    Ok(Pagina { elementos, hay_mas })
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas {
    // LIBRERÍAS

    use super::Pestana;
    use super::hace; // Tiempo transcurrido en palabras
    use super::porcentaje; // Porcentaje de «me salió»
    use super::resumen; // Recorte del comentario // Pestañas del muro

    // FUNCIONES

    /// Comprueba el porcentaje de «me salió»: redondea y no se calcula sin valoraciones.
    #[test]
    fn porcentaje_me_salio() {
        assert_eq!(porcentaje(0, 0), None);
        assert_eq!(porcentaje(3, 4), Some(75));
        assert_eq!(porcentaje(2, 3), Some(67));
        assert_eq!(porcentaje(1, 3), Some(33));
        assert_eq!(porcentaje(5, 5), Some(100));
        assert_eq!(porcentaje(9, 5), Some(100), "never above 100");
    }

    /// Comprueba el orden del muro: Populares va por actividad y todas desempatan igual.
    #[test]
    fn orden_del_muro() {
        assert!(
            Pestana::Populares
                .orden_sql()
                .starts_with("c.recientes + p.recientes DESC")
        );
        assert!(
            Pestana::Recientes
                .orden_sql()
                .starts_with("r.publicada_en DESC")
        );
        assert_eq!(
            Pestana::Siguiendo.orden_sql(),
            Pestana::Recientes.orden_sql()
        );

        for pestana in [Pestana::Recientes, Pestana::Siguiendo, Pestana::Populares] {
            assert!(
                pestana.orden_sql().ends_with("r.publicada_en DESC, r.id"),
                "stable tie-break"
            );
        }
    }

    /// Comprueba que el comentario largo se corta a 140 caracteres y el corto queda igual.
    #[test]
    fn resumen_del_comentario() {
        assert_eq!(resumen("Me salió\n  muy bien"), "Me salió muy bien");

        let largo = "a".repeat(300);
        let corto = resumen(&largo);
        assert_eq!(corto.chars().count(), 140);
        assert!(corto.ends_with('…'));
    }

    /// El servidor solo manda los segundos transcurridos: las palabras las pone el cliente,
    /// que es el único que sabe en qué idioma está la app.
    #[test]
    fn tiempo_transcurrido_en_segundos() {
        assert_eq!(hace(-5), 0);
        assert_eq!(hace(125), 125);
        assert_eq!(hace(3 * 86_400), 3 * 86_400);
    }
}
