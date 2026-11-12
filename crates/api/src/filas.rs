//! Filas de la base que comparten varias consultas, parte del servidor.
//!
//! El vaso de una receta y cada una de sus capas se leen igual en Biblioteca, Ficha,
//! Guardados y Mi recetario; aquí se definen una sola vez, junto con su conversión al
//! formato del motor de proporciones.

// LIBRERÍAS

use crate::biblioteca::ComponenteFicha; // Capa con ingrediente y unidad que viaja a la app

// ESTRUCTURAS

/// Medidas del vaso de una receta, tal como salen de la tabla `vaso`.
///
/// Se incrusta con `#[sqlx(flatten)]` en las filas de receta, así que la consulta debe
/// traer estas columnas con estos mismos nombres.
#[derive(sqlx::FromRow)]
pub(crate) struct MedidasVaso
{
    /// Volumen útil del vaso (ml).
    vaso_capacidad_ml: f64,
    /// Media anchura de la boca.
    radio_boca: f64,
    /// Media anchura del fondo.
    radio_fondo: f64,
    /// Altura interior del vaso.
    altura: f64,
    /// Grosor de la pared.
    pared: f64,
    /// Labio sobre el nivel máximo del líquido.
    borde: f64,
    /// Altura del tallo.
    tallo: f64,
    /// Tamaño del asa (`chica`, `grande` o ninguna).
    asa: Option<String>,
    /// Si el vaso lleva plato.
    plato: bool,
}

/// Capa de una receta tal como vive en `componente_receta`.
#[derive(sqlx::FromRow)]
pub(crate) struct FilaCapa
{
    /// Receta a la que pertenece la capa (UUID en texto).
    pub(crate) receta_id: String,
    /// Ingrediente del catálogo al que apunta la capa.
    ingrediente_id: String,
    /// Orden de la capa dentro de la receta.
    posicion: i16,
    /// Nombre del ingrediente en esta receta.
    nombre: String,
    /// Unidad de la cantidad: `ml` o `mg`.
    unidad: String,
    /// Cantidad en la unidad indicada.
    cantidad: f64,
    /// Alcohol por volumen (0-100).
    graduacion_pct: f64,
    /// Color de la capa en hex, si lo tiene.
    color: Option<String>,
    /// Si la capa es agua o hielo derretido.
    dilucion: bool,
}

// FUNCIONES

impl MedidasVaso
{
    /// Convierte las medidas en el vaso que dibuja el motor de proporciones.
    pub(crate) fn a_vaso(self) -> proporciones::Vaso
    {
        // Elegir el asa según el texto guardado en la base
        let asa_val = match self.asa.as_deref()
        {
            Some("chica") => Some(proporciones::Asa::Chica), // Asa chica
            Some("grande") => Some(proporciones::Asa::Grande), // Asa grande
            _ => None, // Vaso sin asa
        };

        proporciones::Vaso
        {
            capacidad_ml: self.vaso_capacidad_ml,
            radio_boca: self.radio_boca,
            radio_fondo: self.radio_fondo,
            altura: self.altura,
            pared: self.pared,
            borde: self.borde,
            tallo: self.tallo,
            asa: asa_val,
            plato: self.plato,
        }
    }
}

impl FilaCapa
{
    /// Convierte la capa en un componente del motor.
    ///
    /// De momento 1 g = 1 ml: el motor recibe todas las capas como volumen. El detalle con
    /// unidad se conserva en `a_ficha` para guardar versiones y armar la lista de compra.
    pub(crate) fn a_componente(&self) -> proporciones::Componente
    {
        proporciones::Componente
        {
            nombre: self.nombre.clone(),
            ml: crate::ml_equivalentes(self.cantidad, &self.unidad),
            graduacion: self.graduacion_pct,
            dilucion: self.dilucion,
            color: self.color.clone(),
            superficie: None,
        }
    }

    /// Convierte la capa en el componente con ingrediente y unidad que viaja a la app.
    pub(crate) fn a_ficha(&self) -> ComponenteFicha
    {
        ComponenteFicha
        {
            ingrediente_id: self.ingrediente_id.clone(),
            posicion: self.posicion,
            nombre: self.nombre.clone(),
            unidad: self.unidad.clone(),
            cantidad: self.cantidad,
            graduacion_pct: self.graduacion_pct,
            color: self.color.clone(),
            dilucion: self.dilucion,
        }
    }
}
