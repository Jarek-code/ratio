//! Entrada del motor: el vaso, sus ingredientes y la receta que los junta.

/// Asa de una taza o una jarra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Asa {
    Chica,
    Grande,
}

impl Asa {
    /// Cuánto sobresale el asa, en unidades del lienzo.
    pub fn vuelo(self) -> f64 {
        match self {
            Asa::Chica => 20.0,
            Asa::Grande => 26.0,
        }
    }
}

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
    /// Labio por encima del borde interior: separa la boca del nivel del líquido.
    pub borde: f64,
    /// Altura del tallo. Las copas de cóctel separan el bowl del pie.
    pub tallo: f64,
    pub asa: Option<Asa>,
    pub plato: bool,
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
            borde: 0.0,
            tallo: 0.0,
            asa: None,
            plato: false,
        }
    }

    pub fn con_borde(mut self, borde: f64) -> Self {
        self.borde = borde;
        self
    }

    pub fn con_tallo(mut self, tallo: f64) -> Self {
        self.tallo = tallo;
        self
    }

    pub fn con_asa(mut self, asa: Asa) -> Self {
        self.asa = Some(asa);
        self
    }

    pub fn con_plato(mut self) -> Self {
        self.plato = true;
        self
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
    /// Color del líquido en hex (`#RRGGBB`). Si falta o no es válido, se usa la paleta.
    pub color: Option<String>,
}

impl Componente {
    pub fn nuevo(nombre: impl Into<String>, ml: f64, graduacion: f64) -> Self {
        Self {
            nombre: nombre.into(),
            ml,
            graduacion,
            dilucion: false,
            color: None,
        }
    }

    pub fn dilucion(nombre: impl Into<String>, ml: f64) -> Self {
        Self {
            nombre: nombre.into(),
            ml,
            graduacion: 0.0,
            dilucion: true,
            color: None,
        }
    }

    pub fn con_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Una receta lista para medir: de abajo hacia arriba.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Receta {
    pub nombre: String,
    pub vaso: Vaso,
    /// Capacidad efectiva; puede ser menor que la del vaso para dejar aire.
    pub capacidad_ml: f64,
    pub ingredientes: Vec<Componente>,
}

impl Receta {
    pub fn nueva(nombre: impl Into<String>, vaso: Vaso, ingredientes: Vec<Componente>) -> Self {
        let capacidad_ml = vaso.capacidad_ml;
        Self {
            nombre: nombre.into(),
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
            nombre: self.nombre.clone(),
            vaso: self.vaso.clone(),
            capacidad_ml: self.capacidad_ml,
            ingredientes,
        }
    }
}
