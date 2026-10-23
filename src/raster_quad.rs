//! Borrowed line/quad raster records in the 6×11 source cell.
//! Coordinates start at the top left; right and bottom edges are exclusive.

/// A filled rectangle in source-cell units, before renderer scaling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Quad {
    pub top_left: [u8; 2],
    pub bottom_right: [u8; 2],
}

/// Four corners in perimeter order, in source-cell units. A renderer can split
/// these convex quads into triangles (0, 1, 2) and (0, 2, 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxDrawQuad {
    pub vertices: [[f32; 2]; 4],
}

/// Borrowed packed float coordinates, decoded one quad at a time. This view
/// needs neither aligned storage, native pointers in the file, nor allocation.
#[derive(Clone, Copy, Debug)]
pub struct BoxDrawQuads {
    bytes: &'static [u8],
}
impl BoxDrawQuads {
    fn new(bytes: &'static [u8]) -> Option<Self> {
        if bytes.len() % 32 != 0 { return None; }
        for value in bytes.chunks_exact(4) {
            if !f32::from_le_bytes(value.try_into().ok()?).is_finite() { return None; }
        }
        Some(Self { bytes })
    }
    pub fn len(self) -> usize { self.bytes.len() / 32 }
    pub fn is_empty(self) -> bool { self.bytes.is_empty() }
    pub fn get(self, index: usize) -> Option<BoxDrawQuad> {
        let start = index.checked_mul(32)?;
        let bytes = self.bytes.get(start..start.checked_add(32)?)?;
        let mut vertices = [[0.0; 2]; 4];
        for (point, values) in vertices.iter_mut().zip(bytes.chunks_exact(8)) {
            point[0] = f32::from_le_bytes(values[..4].try_into().ok()?);
            point[1] = f32::from_le_bytes(values[4..].try_into().ok()?);
        }
        Some(BoxDrawQuad { vertices })
    }
    pub fn iter(self) -> impl ExactSizeIterator<Item = BoxDrawQuad> {
        (0..self.len()).map(move |index| self.get(index).unwrap())
    }
}
impl IntoIterator for BoxDrawQuads {
    type Item = BoxDrawQuad;
    type IntoIter = BoxDrawQuadIter;
    fn into_iter(self) -> Self::IntoIter { BoxDrawQuadIter { source: self, index: 0 } }
}
#[derive(Clone, Debug)]
pub struct BoxDrawQuadIter { source: BoxDrawQuads, index: usize }
impl Iterator for BoxDrawQuadIter {
    type Item = BoxDrawQuad;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.source.get(self.index)?;
        self.index += 1;
        Some(result)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.source.len() - self.index;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for BoxDrawQuadIter {}

/// Integer rectangle view used by Block Elements and Braille.
#[derive(Clone, Copy, Debug)]
pub struct Quads { source: BoxDrawQuads }
impl Quads {
    fn new(source: BoxDrawQuads) -> Option<Self> {
        for quad in source {
            let [[left, top], [right, t], [r, bottom], [l, b]] = quad.vertices;
            if left != l || top != t || right != r || bottom != b
                || left < 0.0 || top < 0.0 || right > 6.0 || bottom > 11.0
                || left >= right || top >= bottom
                || [left, top, right, bottom].iter().any(|&v| v as u8 as f32 != v)
            { return None; }
        }
        Some(Self { source })
    }
    pub fn len(self) -> usize { self.source.len() }
    pub fn is_empty(self) -> bool { self.source.is_empty() }
    pub fn get(self, index: usize) -> Option<Quad> {
        let quad = self.source.get(index)?;
        Some(Quad { top_left: quad.vertices[0].map(|v| v as u8),
            bottom_right: quad.vertices[2].map(|v| v as u8) })
    }
    pub fn iter(self) -> impl ExactSizeIterator<Item = Quad> {
        (0..self.len()).map(move |index| self.get(index).unwrap())
    }
}
impl IntoIterator for Quads {
    type Item = Quad;
    type IntoIter = QuadIter;
    fn into_iter(self) -> Self::IntoIter { QuadIter { source: self, index: 0 } }
}
#[derive(Clone, Debug)]
pub struct QuadIter { source: Quads, index: usize }
impl Iterator for QuadIter {
    type Item = Quad;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.source.get(self.index)?;
        self.index += 1;
        Some(result)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.source.len() - self.index;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for QuadIter {}

/// Borrow any ready-to-use quad glyph from the installed binary bundle.
pub fn glyph_quads(character: char) -> Option<BoxDrawQuads> {
    let record = crate::bundle::record(character as u32)?;
    if record.kind() != 2 { return None; }
    BoxDrawQuads::new(record.raster())
}

/// Borrow exact filled rectangles for any Block Element; non-blocks return `None`.
pub fn block_quad_slice(character: char) -> Option<Quads> {
    if !('\u{2580}'..='\u{259f}').contains(&character) { return None; }
    Quads::new(glyph_quads(character)?)
}
/// Iterate stored block rectangles, including shades, without allocation.
pub fn block_quads(character: char) -> Option<impl ExactSizeIterator<Item = Quad>> {
    Some(block_quad_slice(character)?.iter())
}
/// Blank Braille returns an empty view; other characters return `None`.
pub fn braille_quad_slice(character: char) -> Option<Quads> {
    if !('\u{2800}'..='\u{28ff}').contains(&character) { return None; }
    Quads::new(glyph_quads(character)?)
}
/// Iterate stored single-dot quads without allocation or pattern reconstruction.
pub fn braille_quads(character: char) -> Option<impl ExactSizeIterator<Item = Quad>> {
    Some(braille_quad_slice(character)?.iter())
}
/// Rounded corners and non-box characters return `None`. Diagonals remain
/// continuous ribbons rather than staircase pixel rectangles.
pub fn boxdraw_quad_slice(character: char) -> Option<BoxDrawQuads> {
    if !('\u{2500}'..='\u{257f}').contains(&character) { return None; }
    let quads = glyph_quads(character)?;
    (!quads.is_empty()).then_some(quads)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_quad_payloads_are_rejected() {
        assert!(BoxDrawQuads::new(&[0; 31]).is_none());
        assert!(BoxDrawQuads::new(&[0; 32]).is_some());
        assert!(Quads::new(BoxDrawQuads::new(&[0; 32]).unwrap()).is_none());
        assert!(BoxDrawQuads::new(&[]).unwrap().is_empty());
    }
    #[test]
    fn stored_quads_cover_each_braille_dot_and_blank() {
        for pattern in 0u32..256 {
            let ch = char::from_u32(0x2800 + pattern).unwrap();
            let quads = braille_quad_slice(ch).unwrap();
            assert_eq!(quads.len(), pattern.count_ones() as usize);
            assert!(quads.get(quads.len()).is_none());
            for quad in quads {
                assert_eq!(quad.bottom_right, quad.top_left.map(|v| v + 1));
            }
        }
    }
}
