//! Font selection, independent of its Meso/Lean shape.
//! Dista uses 90° and 45° angles with softly rounded corners.
//! Convo uses curvier, bending lines throughout.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Font {
    Dista,
    Convo,
}
