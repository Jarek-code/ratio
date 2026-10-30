//! C2: guardar una version privada de una receta ajustada.
//!
//! La version es una fila nueva de `receta` con `receta_origen_id`; nunca modifica ni
//! despublica la receta original. Las capas copian la metadata del origen y solo cambian
//! las cantidades de los liquidos que el usuario ajusto en C1.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::db;

/// Capa que C1 ajusto y quiere conservar en la version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponenteNuevo {
    /// Ingrediente del catalogo al que apunta la capa.
    pub ingrediente_id: String,
    /// Posicion original de la capa; permite conservar solidos y orden.
    pub posicion: i16,
    /// Cantidad nueva para una capa liquida, en mililitros.
    pub cantidad: f64,
}

/// Datos minimos para guardar una version privada desde C1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecetaNueva {
    /// Receta publicada o propia de la que se deriva.
    pub origen_id: String,
    /// Nombre visible de la version.
    pub nombre: String,
    /// Vaso elegido en C1.
    pub vaso_id: String,
    /// Capacidad efectiva elegida en C1.
    pub capacidad_ml: f64,
    /// Capas liquidas ajustadas; los solidos se copian del origen.
    pub componentes: Vec<ComponenteNuevo>,
}

/// Resultado de guardar una version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionGuardada {
    /// Identificador de la receta nueva.
    pub receta_id: String,
    /// Avisos no bloqueantes, como quedar por encima del 94 % del vaso.
    pub advertencias: Vec<String>,
}

/// Fila de la receta origen que se copia en la version (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaOrigen {
    /// Pestaña de la bebida.
    tipo: String,
    /// Familia de la bebida.
    familia: String,
    /// Fotos de la receta original.
    fotos_rutas: Vec<String>,
    /// Equipo de la receta original.
    equipo: Vec<String>,
    /// Tecnica de la receta original.
    tecnica: Option<String>,
    /// Temperatura de servicio.
    temperatura: Option<String>,
    /// Dificultad declarada.
    dificultad: Option<i16>,
    /// Intensidad declarada.
    intensidad: Option<i16>,
    /// Tiempo de preparacion.
    minutos: Option<i16>,
    /// Dosis de cafe, si aplica.
    dosis_g: Option<f64>,
    /// Notas de sabor.
    notas: Vec<String>,
    /// Etiquetas de la receta.
    etiquetas: Vec<String>,
}

/// Fila de una capa del origen (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaComponenteOrigen {
    /// Ingrediente del catalogo.
    ingrediente_id: String,
    /// Posicion original.
    posicion: i16,
    /// Nombre copiado en la receta.
    nombre: String,
    /// Unidad de la cantidad.
    unidad: String,
    /// Cantidad original.
    cantidad: f64,
    /// Alcohol por volumen.
    graduacion_pct: f64,
    /// Color de la capa.
    color: Option<String>,
    /// Si es agua o hielo derretido.
    dilucion: bool,
}

/// Fila de un paso del origen (solo servidor).
#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct FilaPasoOrigen {
    /// Posicion original del paso.
    posicion: i16,
    /// Titulo opcional.
    titulo: Option<String>,
    /// Instruccion completa.
    instruccion: String,
    /// Temporizador opcional.
    temporizador_s: Option<i32>,
}

/// Guarda una version privada de una receta visible para la cuenta.
///
/// El autor se toma del token, nunca del cliente. La receta original no se modifica:
/// la version queda con `publicada_en = NULL` y `receta_origen_id` apuntando al origen.
///
/// # Errores
///
/// Devuelve `ServerFnError` si no hay sesion, si el origen no es visible, si falta una
/// capa, si la version no cabe en el vaso o si la transaccion falla.
#[server(endpoint = "guardar_receta")]
pub async fn guardar_receta(
    token: String,
    datos: RecetaNueva,
) -> Result<VersionGuardada, ServerFnError> {
    let Some(pool) = db::pool() else {
        return Err(ServerFnError::new("Database unavailable"));
    };

    let cuenta = crate::auth::cuenta(Some(&token))
        .await
        .ok_or_else(|| ServerFnError::new("Inicia sesion para guardar una version"))?;

    let nombre = datos.nombre.trim();
    if nombre.is_empty() || nombre.chars().count() > 120 {
        return Err(ServerFnError::new(
            "El nombre debe tener entre 1 y 120 caracteres",
        ));
    }

    if !datos.capacidad_ml.is_finite() || datos.capacidad_ml <= 0.0 {
        return Err(ServerFnError::new("La capacidad del vaso no es valida"));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| ServerFnError::new("No se pudo iniciar la transaccion"))?;

    let origen = sqlx::query_as::<_, FilaOrigen>(
        r#"
        SELECT r.tipo,
               r.familia,
               r.fotos_rutas,
               r.equipo,
               r.tecnica,
               r.temperatura,
               r.dificultad,
               r.intensidad,
               r.minutos,
               r.dosis_g,
               r.notas,
               r.etiquetas
        FROM receta r
        WHERE r.id = $1::uuid
          AND NOT r.oculta
          AND (r.publicada_en IS NOT NULL OR r.autor_id = $2::uuid)
        "#,
    )
    .bind(&datos.origen_id)
    .bind(&cuenta)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| ServerFnError::new("No se pudo leer la receta original"))?
    .ok_or_else(|| ServerFnError::new("No puedes versionar esa receta"))?;

    let vaso_capacidad =
        sqlx::query_scalar::<_, f64>("SELECT capacidad_ml FROM vaso WHERE id = $1::uuid")
            .bind(&datos.vaso_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| ServerFnError::new("No se pudo leer el vaso"))?
            .ok_or_else(|| ServerFnError::new("El vaso elegido no existe"))?;

    if datos.capacidad_ml > vaso_capacidad + 0.01 {
        return Err(ServerFnError::new(
            "La capacidad no puede superar la del vaso",
        ));
    }

    let capas = sqlx::query_as::<_, FilaComponenteOrigen>(
        r#"
        SELECT ingrediente_id::text AS ingrediente_id,
               posicion,
               nombre,
               unidad,
               cantidad,
               graduacion_pct,
               color,
               dilucion
        FROM componente_receta
        WHERE receta_id = $1::uuid
        ORDER BY posicion
        "#,
    )
    .bind(&datos.origen_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| ServerFnError::new("No se pudieron leer los ingredientes"))?;

    let mut usados = vec![false; datos.componentes.len()];
    let mut capas_nuevas = Vec::with_capacity(capas.len());
    let mut total_ml = 0.0;
    let mut tiene_alcohol = false;

    for capa in capas {
        // Un factor cero en C1 significa quitar la capa, no guardar una cantidad invalida.
        let cantidad = if capa.unidad == "ml" {
            let Some(indice) = datos.componentes.iter().position(|enviado| {
                enviado.posicion == capa.posicion && enviado.ingrediente_id == capa.ingrediente_id
            }) else {
                continue;
            };

            if usados[indice] {
                return Err(ServerFnError::new(
                    "Hay un ingrediente repetido en la version",
                ));
            }

            usados[indice] = true;
            let cantidad = datos.componentes[indice].cantidad;
            if !cantidad.is_finite() || cantidad <= 0.0 {
                return Err(ServerFnError::new(
                    "Las cantidades deben ser mayores que cero",
                ));
            }
            total_ml += cantidad;
            cantidad
        } else {
            capa.cantidad
        };

        tiene_alcohol |= capa.graduacion_pct > 0.0;
        capas_nuevas.push((capa, cantidad));
    }

    if usados.iter().any(|usado| !usado) {
        return Err(ServerFnError::new("Hay un ingrediente ajeno a la receta"));
    }

    if total_ml > datos.capacidad_ml {
        return Err(ServerFnError::new("La bebida no cabe en el vaso"));
    }

    if tiene_alcohol && !crate::auth::es_adulto(Some(&token)).await {
        return Err(ServerFnError::new(
            "Solo una cuenta de mayor de edad puede guardar bebidas con alcohol",
        ));
    }

    let receta_id = sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO receta (
            autor_id, receta_origen_id, nombre, tipo, familia, vaso_id, capacidad_ml,
            fotos_rutas, equipo, tecnica, temperatura, dificultad, intensidad,
            minutos, dosis_g, notas, etiquetas
        )
        VALUES (
            $1::uuid, $2::uuid, $3, $4, $5, $6::uuid, $7,
            $8, $9, $10, $11, $12, $13,
            $14, $15, $16, $17
        )
        RETURNING id::text
        "#,
    )
    .bind(&cuenta)
    .bind(&datos.origen_id)
    .bind(nombre)
    .bind(&origen.tipo)
    .bind(&origen.familia)
    .bind(&datos.vaso_id)
    .bind(datos.capacidad_ml)
    .bind(&origen.fotos_rutas)
    .bind(&origen.equipo)
    .bind(&origen.tecnica)
    .bind(&origen.temperatura)
    .bind(origen.dificultad)
    .bind(origen.intensidad)
    .bind(origen.minutos)
    .bind(origen.dosis_g)
    .bind(&origen.notas)
    .bind(&origen.etiquetas)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| ServerFnError::new("No se pudo crear la version"))?;

    for (capa, cantidad) in capas_nuevas {
        sqlx::query(
            r#"
            INSERT INTO componente_receta (
                receta_id, posicion, ingrediente_id, nombre,
                graduacion_pct, color, cantidad, unidad, dilucion
            )
            VALUES ($1::uuid, $2, $3::uuid, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&receta_id)
        .bind(capa.posicion)
        .bind(&capa.ingrediente_id)
        .bind(&capa.nombre)
        .bind(capa.graduacion_pct)
        .bind(&capa.color)
        .bind(cantidad)
        .bind(&capa.unidad)
        .bind(capa.dilucion)
        .execute(&mut *tx)
        .await
        .map_err(|_| ServerFnError::new("No se pudieron copiar los ingredientes"))?;
    }

    let pasos = sqlx::query_as::<_, FilaPasoOrigen>(
        r#"
        SELECT posicion, titulo, instruccion, temporizador_s
        FROM paso
        WHERE receta_id = $1::uuid
        ORDER BY posicion
        "#,
    )
    .bind(&datos.origen_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| ServerFnError::new("No se pudieron leer los pasos"))?;

    for paso in pasos {
        sqlx::query(
            r#"
            INSERT INTO paso (receta_id, posicion, titulo, instruccion, temporizador_s)
            VALUES ($1::uuid, $2, $3, $4, $5)
            "#,
        )
        .bind(&receta_id)
        .bind(paso.posicion)
        .bind(&paso.titulo)
        .bind(&paso.instruccion)
        .bind(paso.temporizador_s)
        .execute(&mut *tx)
        .await
        .map_err(|_| ServerFnError::new("No se pudieron copiar los pasos"))?;
    }

    tx.commit()
        .await
        .map_err(|_| ServerFnError::new("No se pudo confirmar la version"))?;

    let mut advertencias = Vec::new();
    if total_ml > datos.capacidad_ml * proporciones::LLENADO_MAX {
        advertencias.push("La bebida queda al limite del vaso.".to_string());
    }

    Ok(VersionGuardada {
        receta_id,
        advertencias,
    })
}
