//! Entrada del motor: el vaso, sus ingredientes y la receta que los junta.

/// Silueta de un vaso, en unidades del lienzo.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vaso {
    pub capacidad_ml: f64,
    /// Media anchura de la boca.
    pub radio_boca: f64,
    /// Media anchura del fondo.
    pub radio_fondo: f64,
    pub altura: f64,
    pub pared: f64,
}

impl Vaso {
    pub fn nuevo(
        capacidad_ml: f64,
        radio_boca: f64,
        radio_fondo: f64,
        altura: f64,
        pared: f64,
    ) -> Self {
        Self {
            capacidad_ml,
            radio_boca,
            radio_fondo,
            altura,
            pared,
        }
    }
}

/// Un ingrediente con su cantidad. La graduación es la del líquido de origen, en % ABV.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Componente {
    pub nombre: String,
    pub ml: f64,
    pub graduacion: f64,
    /// El hielo y el agua de agitación suman volumen y bajan la graduación.
    pub dilucion: bool,
}

impl Componente {
    pub fn nuevo(nombre: impl Into<String>, ml: f64, graduacion: f64) -> Self {
        Self {
            nombre: nombre.into(),
            ml,
            graduacion,
            dilucion: false,
        }
    }

    pub fn dilucion(nombre: impl Into<String>, ml: f64) -> Self {
        Self {
            nombre: nombre.into(),
            ml,
            graduacion: 0.0,
            dilucion: true,
        }
    }
}

/// Una receta lista para medir: de abajo hacia arriba.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Receta {
    pub vaso: Vaso,
    /// Capacidad efectiva; puede ser menor que la del vaso para dejar aire.
    pub capacidad_ml: f64,
    pub ingredientes: Vec<Componente>,
}

impl Receta {
    pub fn nueva(vaso: Vaso, ingredientes: Vec<Componente>) -> Self {
        let capacidad_ml = vaso.capacidad_ml;
        Self {
            vaso,
            capacidad_ml,
            ingredientes,
        }
    }

    pub fn con_capacidad(mut self, capacidad_ml: f64) -> Self {
        self.capacidad_ml = capacidad_ml;
        self
    }

    /// Multiplica las cantidades. El vaso no cambia: escalar no cambia de recipiente.
    pub fn escalar(&self, factor: f64) -> Self {
        let ingredientes = self
            .ingredientes
            .iter()
            .map(|c| Componente {
                ml: c.ml * factor,
                ..c.clone()
            })
            .collect();
        Self {
            vaso: self.vaso.clone(),
            capacidad_ml: self.capacidad_ml,
            ingredientes,
        }
    }
}
