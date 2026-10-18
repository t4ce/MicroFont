//! Font shape selection, independent of raster or stamp support.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontShape {
    Meso,
    Lean,
}

impl FontShape {
    /// Shared influence width in source-cell units, for both font families.
    pub const INFLUENCE_WIDTH: f64 = 0.20;

    pub const fn body_threshold(self) -> f64 {
        match self {
            Self::Meso => 0.24,
            Self::Lean => 0.70,
        }
    }
}
