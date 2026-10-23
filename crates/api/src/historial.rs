//! Historial de versiones de una bebida (B6).
//!
//! Recorre el árbol de versiones que cuelga de `receta.receta_origen_id`: sube desde la
//! receta pedida hasta la original y baja por todas sus derivadas, y compara cada versión con
//! su origen ingrediente a ingrediente.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db;

/// Cambio de cantidad de un ingrediente respecto de la versión de la que deriva.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CambioHistorial {
    /// Nombre visible del ingrediente.
    pub ingrediente: String,
    /// Cantidad en la versión de origen (0 si no lo llevaba).
    pub antes: f64,
    /// Cantidad en esta versión (0 si se quitó).
    pub despues: f64,
    /// Unidad de las cantidades, por ejemplo `ml` o `mg`.
    pub unidad: String,
}

/// Una versión de la bebida tal como se pinta en el historial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionHistorial {
    /// Identificador de la receta (UUID en texto).
    pub id: String,
    /// Nombre de esta versión.
    pub nombre: String,
    /// Alias de quien la creó.
    pub autor: String,
    /// Fecha legible, por ejemplo «12 oct 2026».
    pub fecha: String,
    /// Si cualquiera puede verla; las privadas solo las ve su autor.
    pub publica: bool,
    /// Nombre de la versión de la que deriva; `None` en la original.
    pub origen: Option<String>,
    /// Volumen total de la bebida en mililitros.
    pub total_ml: f64,
    /// Proporción entre el total y el ingrediente base, por ejemplo «1:4,5».
    pub ratio: String,
    /// Cambios frente a la versión de origen; vacío en la original.
    pub cambios: Vec<CambioHistorial>,
}

/// Fila de la consulta de versiones (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaVersion {
    /// Identificador de la receta (UUID en texto).
    id: String,
    /// Nombre de la versión.
    nombre: String,
    /// Identificador de la receta de la que deriva.
    origen_id: Option<String>,
    /// Alias del autor, o «Anónimo» si la cuenta ya no existe.
    autor: String,
    /// Si la versión está publicada y visible.
    publica: bool,
    /// Día de creación.
    dia: i32,
    /// Mes de creación (1-12).
    mes: i32,
    /// Año de creación.
    anio: i32,
}

/// Fila de un componente de cualquiera de las versiones (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaComponente {
    /// Receta a la que pertenece el componente.
    receta_id: String,
    /// Nombre visible del ingrediente.
    nombre: String,
    /// Cantidad en su unidad.
    cantidad: f64,
    /// Unidad de la cantidad.
    unidad: String,
    /// Alcohol por volumen, para elegir el ingrediente base.
    graduacion_pct: f64,
}

/// Árbol de versiones de una receta: sube a la original y baja por sus derivadas.
#[cfg(feature = "server")]
const ARBOL: &str = r#"
WITH RECURSIVE arriba AS (
    SELECT r.id, r.receta_origen_id
    FROM receta r
    WHERE r.id = $1::uuid
    UNION ALL
    SELECT p.id, p.receta_origen_id
    FROM receta p
    JOIN arriba a ON p.id = a.receta_origen_id
), arbol AS (
    SELECT id FROM arriba WHERE receta_origen_id IS NULL
    UNION ALL
    SELECT r.id FROM receta r JOIN arbol t ON r.receta_origen_id = t.id
)
"#;

/// Nombres de los meses en español, para la fecha legible.
#[cfg(feature = "server")]
const MESES: [&str; 12] = [
    "ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic",
];

/// Proporción entre el total y el ingrediente base, con coma decimal («1:4,5»).
#[cfg(feature = "server")]
fn ratio_de(total: f64, base: f64) -> String {
    if total <= 0.0 || base <= 0.0 {
        return "—".to_string();
    }

    let valor = (total / base * 10.0).round() / 10.0;

    if valor.fract().abs() < 0.05 {
        format!("1:{valor:.0}")
    } else {
        format!("1:{valor:.1}").replace('.', ",")
    }
}

/// Suma de las cantidades en mililitros de una lista de componentes.
#[cfg(feature = "server")]
fn total_ml_de(componentes: &[FilaComponente]) -> f64 {
    componentes
        .iter()
        .filter(|componente| componente.unidad == "ml")
        .map(|componente| componente.cantidad)
        .sum()
}

/// Cantidad del ingrediente base: el de mayor graduación y, si ninguno tiene alcohol, el primero.
#[cfg(feature = "server")]
fn base_ml_de(componentes: &[FilaComponente]) -> f64 {
    let con_alcohol = componentes
        .iter()
        .filter(|componente| componente.graduacion_pct > 0.0)
        .max_by(|a, b| a.graduacion_pct.total_cmp(&b.graduacion_pct));

    match con_alcohol.or_else(|| componentes.first()) {
        Some(componente) if componente.unidad == "ml" => componente.cantidad,
        _ => 0.0,
    }
}

/// Cambios de esta versión frente a su origen, ordenados por nombre de ingrediente.
#[cfg(feature = "server")]
fn cambios_de(version: &[FilaComponente], origen: &[FilaComponente]) -> Vec<CambioHistorial> {
    let mut nombres: Vec<&str> = version
        .iter()
        .map(|componente| componente.nombre.as_str())
        .chain(origen.iter().map(|componente| componente.nombre.as_str()))
        .collect();
    nombres.sort_unstable();
    nombres.dedup();

    let mut cambios = Vec::new();

    for nombre in nombres {
        let actual = version
            .iter()
            .find(|componente| componente.nombre == nombre)
            .map(|componente| (componente.cantidad, componente.unidad.clone()));
        let previo = origen
            .iter()
            .find(|componente| componente.nombre == nombre)
            .map(|componente| componente.cantidad);

        let despues = actual.as_ref().map(|(cantidad, _)| *cantidad).unwrap_or(0.0);
        let antes = previo.unwrap_or(0.0);

        if (despues - antes).abs() < 0.05 {
            continue;
        }

        cambios.push(CambioHistorial {
            ingrediente: nombre.to_string(),
            antes,
            despues,
            unidad: actual
                .map(|(_, unidad)| unidad)
                .or_else(|| {
                    origen
                        .iter()
                        .find(|componente| componente.nombre == nombre)
                        .map(|componente| componente.unidad.clone())
                })
                .unwrap_or_else(|| "ml".to_string()),
        });
    }

    cambios
}

/// Trae las versiones publicadas de una bebida y, si hay sesión, también las privadas de su autor.
///
/// Sin token solo se devuelven las versiones públicas; las privadas quedan para quien las creó.
///
/// # Errores
///
/// Devuelve `ServerFnError` si el servidor no tiene pool o si alguna consulta falla.
#[server(endpoint = "historial")]
pub async fn versiones(
    id: String,
    token: Option<String>,
) -> Result<Vec<VersionHistorial>, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cuenta = crate::auth::cuenta(token.as_deref()).await;

    let filas = sqlx::query_as::<_, FilaVersion>(&format!(
        r#"{ARBOL}
        SELECT r.id::text AS id,
               r.nombre,
               r.receta_origen_id::text AS origen_id,
               COALESCE(u.alias, 'Anónimo') AS autor,
               (r.publicada_en IS NOT NULL AND NOT r.oculta) AS publica,
               EXTRACT(DAY FROM r.creada_en)::int AS dia,
               EXTRACT(MONTH FROM r.creada_en)::int AS mes,
               EXTRACT(YEAR FROM r.creada_en)::int AS anio
        FROM receta r
        LEFT JOIN usuario u ON u.id = r.autor_id
        WHERE r.id IN (SELECT id FROM arbol)
          AND ((r.publicada_en IS NOT NULL AND NOT r.oculta) OR r.autor_id = $2::uuid)
        ORDER BY r.creada_en, r.nombre
        "#
    ))
    .bind(&id)
    .bind(&cuenta)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    if filas.is_empty() {
        return Ok(Vec::new());
    }

    let componentes = sqlx::query_as::<_, FilaComponente>(&format!(
        r#"{ARBOL}
        SELECT c.receta_id::text AS receta_id, c.nombre, c.cantidad, c.unidad, c.graduacion_pct
        FROM componente_receta c
        WHERE c.receta_id IN (SELECT id FROM arbol)
        ORDER BY c.receta_id, c.posicion
        "#
    ))
    .bind(&id)
    .fetch_all(pool)
    .await
    .map_err(|_| ServerFnError::new("Query error"))?;

    let de_receta = |receta: &str| -> Vec<FilaComponente> {
        componentes
            .iter()
            .filter(|componente| componente.receta_id == receta)
            .map(|componente| FilaComponente {
                receta_id: componente.receta_id.clone(),
                nombre: componente.nombre.clone(),
                cantidad: componente.cantidad,
                unidad: componente.unidad.clone(),
                graduacion_pct: componente.graduacion_pct,
            })
            .collect()
    };

    let versiones = filas
        .iter()
        .map(|fila| {
            let propias = de_receta(&fila.id);
            let origen = fila.origen_id.as_deref().unwrap_or_default();
            let del_origen = if fila.origen_id.is_some() {
                de_receta(origen)
            } else {
                Vec::new()
            };
            let total_ml = total_ml_de(&propias);

            VersionHistorial {
                id: fila.id.clone(),
                nombre: fila.nombre.clone(),
                autor: fila.autor.clone(),
                fecha: format!(
                    "{} {} {}",
                    fila.dia,
                    MESES[(fila.mes.clamp(1, 12) - 1) as usize],
                    fila.anio
                ),
                publica: fila.publica,
                origen: fila.origen_id.as_ref().and_then(|origen_id| {
                    filas
                        .iter()
                        .find(|otra| &otra.id == origen_id)
                        .map(|otra| otra.nombre.clone())
                }),
                total_ml,
                ratio: ratio_de(total_ml, base_ml_de(&propias)),
                cambios: if fila.origen_id.is_some() {
                    cambios_de(&propias, &del_origen)
                } else {
                    Vec::new()
                },
            }
        })
        .collect();

    Ok(versiones)
}
