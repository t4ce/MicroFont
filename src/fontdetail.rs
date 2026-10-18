//! Dista triangle tessellation detail, independent of its Meso/Lean shape.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum FontDetail {
    #[default]
    Fine = 0,
    High = 1,
}

impl FontDetail {
    /// Select detail from a stored 0/1 setting.
    pub const fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(Self::Fine),
            1 => Some(Self::High),
            _ => None,
        }
    }

    /// Geometry subdivisions per source-cell unit, matching the HTML presets.
    pub const fn subdivisions_per_cell(self) -> usize {
        match self {
            Self::Fine => 12,
            Self::High => 24,
        }
    }

    /// Direct circle-fan budget for each lit Braille dot.
    pub const fn braille_triangles_per_dot(self) -> usize {
        match self {
            Self::Fine => 36,
            Self::High => 72,
        }
    }
}
