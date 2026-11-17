//! Diario de preparaciones (D4): gusto y duración de cada preparación, parte compartida.
//!
//! «Me salió / no me salió» dice si la receta se pudo reproducir; el gusto (1-5) y el tiempo
//! que tomó se guardan aparte, en la misma fila de `preparacion`, y no cambian el porcentaje
//! de «me salió» de la receta. El diario lista las preparaciones de la cuenta.

// LIBRERÍAS

use dioxus::prelude::*; // Macro server y ServerFnError
use serde::Deserialize; // Lectura de los tipos que viajan entre app y servidor
use serde::Serialize; // Escritura de los tipos que viajan entre app y servidor

#[cfg(feature = "server")]
use crate::base::ConPlazo; // Plazo de cada consulta

// CONSTANTES Y VARIABLES GLOBALES

/// Duración máxima que se acepta para una preparación: un día, en segundos.
pub const DURACION_MAXIMA_S: i32 = 86_400;

/// Preparaciones que trae el diario.
#[cfg(feature = "server")]
const ENTRADAS_MAXIMAS: i64 = 100;

// ESTRUCTURAS

/// Preparación de la cuenta tal como se pinta en «Mi diario».
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntradaDiario
{
    /// Receta preparada; `None` si la cuenta ya no puede abrirla.
    pub receta_id: Option<String>,
    /// Nombre de la receta preparada.
    pub receta_nombre: String,
    /// Si salió igual a la ficha.
    pub me_salio: bool,
    /// Cuánto gustó, de 1 a 5, si se contestó.
    pub gusto: Option<i16>,
    /// Cuánto tomó la preparación, en segundos, si se midió.
    pub duracion_s: Option<i32>,
    /// Fecha legible, por ejemplo «12 nov 2026».
    pub fecha: String,
}

/// Fila de una preparación del diario (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaEntrada
{
    /// Receta preparada.
    receta_id: String,
    /// Nombre de la receta.
    receta_nombre: String,
    /// Si la cuenta todavía puede abrir la receta.
    receta_visible: bool,
    /// Si salió igual a la ficha.
    me_salio: bool,
    /// Gusto de 1 a 5.
    gusto: Option<i16>,
    /// Duración en segundos.
    duracion_s: Option<i32>,
    /// Día de la preparación.
    dia: i32,
    /// Mes de la preparación (1-12).
    mes: i32,
    /// Año de la preparación.
    anio: i32,
}

// FUNCIONES

/// Duración en palabras: «45 s», «8 min» o «1 h 5 min».
pub fn duracion_legible(segundos: i32) -> String
{
    let segundos = segundos.max(0); // Una duración nunca es negativa
    match segundos
    {
        0..60 => format!("{segundos} s"),
        60..3_600 => format!("{} min", segundos / 60),
        _ => format!("{} h {} min", segundos / 3_600, segundos % 3_600 / 60),
    }
}

/// Guarda el gusto y la duración en la última preparación de la cuenta para una receta (D4).
///
/// Solo se cambia lo que llega: un valor `None` conserva lo que ya estaba guardado.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión, si el gusto no está entre 1 y 5, si la duración
/// no es válida, si la cuenta todavía no dijo si le salió la receta o si la escritura falla.
#[server(endpoint = "detallar_preparacion")]
pub async fn detallar_preparacion(
    token: String,
    receta_id: String,
    gusto: Option<i16>,
    duracion_s: Option<i32>,
) -> Result<(), ServerFnError>
{
    if !crate::fallos::es_uuid(&receta_id)
    {
        return Err(crate::fallos::datos(
            "La receta pedida no es válida: vuelve a abrirla desde el recetario.",
        ));
    }
    if gusto.is_some_and(|valor| !(1..=5).contains(&valor))
    {
        return Err(crate::fallos::datos("El gusto va de 1 a 5."));
    }
    if duracion_s.is_some_and(|valor| !(0..=DURACION_MAXIMA_S).contains(&valor))
    {
        return Err(crate::fallos::datos("La duración de la preparación no es válida."));
    }

    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;

    // Completar la preparación más reciente de la cuenta para esa receta
    let actualizada = sqlx::query_scalar::<_, String>(
        r#"
        UPDATE preparacion
        SET gusto = COALESCE($3, gusto),
            duracion_s = COALESCE($4, duracion_s)
        WHERE id = (
            SELECT id FROM preparacion
            WHERE usuario_id = $1::uuid AND receta_id = $2::uuid
            ORDER BY creada_en DESC
            LIMIT 1
        )
        RETURNING id::text
        "#,
    )
    .bind(&cuenta)
    .bind(&receta_id)
    .bind(gusto)
    .bind(duracion_s)
    .fetch_optional(pool)
    .con_plazo("tu preparación")
    .await?;

    actualizada
        .map(|_| ())
        .ok_or_else(|| crate::fallos::datos("Primero di si te salió la receta.")) // Sin preparación que completar
}

/// Trae las últimas 100 preparaciones de la cuenta, de la más reciente a la más antigua (D4).
///
/// Una receta con alcohol no aparece si la cuenta no declaró mayoría de edad.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesión o si la consulta falla.
#[server(endpoint = "diario")]
pub async fn diario(token: String) -> Result<Vec<EntradaDiario>, ServerFnError>
{
    let pool = crate::base::pool()?;
    let cuenta = crate::foro::cuenta_obligatoria(&token).await?;
    let adulto = crate::auth::es_adulto(Some(&token)).await;

    // Consultar las preparaciones con su receta y si esa receta sigue visible
    let consulta = format!(
        r#"
        SELECT r.id::text AS receta_id,
               r.nombre AS receta_nombre,
               (NOT r.oculta AND (r.publicada_en IS NOT NULL OR r.autor_id = $1::uuid)) AS receta_visible,
               p.me_salio,
               p.gusto,
               p.duracion_s,
               EXTRACT(DAY FROM p.creada_en)::int AS dia,
               EXTRACT(MONTH FROM p.creada_en)::int AS mes,
               EXTRACT(YEAR FROM p.creada_en)::int AS anio
        FROM preparacion p
        JOIN receta r ON r.id = p.receta_id
        WHERE p.usuario_id = $1::uuid
          AND ($2 OR {sin_alcohol})
        ORDER BY p.creada_en DESC, p.id
        LIMIT $3
        "#,
        sin_alcohol = crate::foro::sin_alcohol("r"),
    );
    let filas_vec = sqlx::query_as::<_, FilaEntrada>(&consulta)
        .bind(&cuenta)
        .bind(adulto)
        .bind(ENTRADAS_MAXIMAS)
        .fetch_all(pool)
        .con_plazo("tu diario")
        .await?;

    // Convertir cada fila en la entrada que pinta la app
    let entradas_vec = filas_vec
        .into_iter()
        .map(|fila| EntradaDiario
        {
            receta_id: fila.receta_visible.then_some(fila.receta_id),
            receta_nombre: fila.receta_nombre,
            me_salio: fila.me_salio,
            gusto: fila.gusto,
            duracion_s: fila.duracion_s,
            fecha: crate::foro::fecha_legible(fila.dia, fila.mes, fila.anio),
        })
        .collect();

    Ok(entradas_vec)
}

// PUNTO DE PARTIDA

#[cfg(test)]
mod pruebas
{
    // LIBRERÍAS

    use super::duracion_legible; // Duración en palabras

    // FUNCIONES

    /// Comprueba la duración en segundos, minutos y horas.
    #[test]
    fn duracion_en_palabras()
    {
        assert_eq!(duracion_legible(45), "45 s");
        assert_eq!(duracion_legible(492), "8 min");
        assert_eq!(duracion_legible(3_900), "1 h 5 min");
    }
}
