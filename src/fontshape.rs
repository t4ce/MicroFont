//! Named native-line weight presets for the current glyph experiments.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontShape {
    Meso,
    Lean,
}

impl FontShape {
    /// Native screen-pixel width at the demo's current glyph size.
    pub const fn line_width_px(self) -> f32 {
        match self {
            Self::Meso => 9.0,
            Self::Lean => 7.0,
        }
    }
}
