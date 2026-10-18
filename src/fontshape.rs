//! Font shape selection, independent of raster or stamp support.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontShape {
    Meso,
    Lean,
}

impl FontShape {
    /// Dista influence width in source-cell units, shared by Meso and Lean.
    pub const INFLUENCE_WIDTH: f64 = 0.20;

    /// Dista uses 90° and 45° angles with softly rounded corners.
    pub const fn body_threshold(self) -> f64 {
        match self {
            Self::Meso => 0.22,
            Self::Lean => 0.63,
        }
    }
}
