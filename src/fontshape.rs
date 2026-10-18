//! Font shape selection, independent of raster or stamp support.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontShape {
    Meso,
    Lean,
}

