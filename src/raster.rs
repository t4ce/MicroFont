//! Small glyphs expressed as axis-aligned quads in the 6×11 source cell.
//! Coordinates start at the top left; right and bottom edges are exclusive.

/// A filled rectangle in source-cell units, before renderer scaling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Quad {
    pub top_left: [u8; 2],
    pub bottom_right: [u8; 2],
}

/// Precomputed quads for one glyph. Unused array slots are private.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuadGlyph<const N: usize> {
    pub(crate) quads: [Quad; N],
    pub(crate) len: u8,
}

impl<const N: usize> QuadGlyph<N> {
    pub(crate) const EMPTY: Self = Self {
        quads: [Quad { top_left: [0, 0], bottom_right: [0, 0] }; N],
        len: 0,
    };

    /// Only the filled quads, in Unicode dot order.
    pub fn quads(&self) -> &[Quad] {
        &self.quads[..self.len as usize]
    }
}

/// At most eight single-dot quads per Braille glyph.
pub type BrailleGlyph = QuadGlyph<8>;
/// At most two filled rectangles per solid Block Element.
pub type BlockGlyph = QuadGlyph<2>;

/// All 256 Braille glyphs, populated at compile time alongside their pixel masks.
/// No initialization, warm-up, allocation or geometry generation is needed.
pub static BRAILLE_QUADS: [BrailleGlyph; 256] = crate::braille::DATA.quads;

/// Borrow ready-to-use quads in 6×11 source coordinates.
/// Blank Braille returns an empty slice; other characters return `None`.
pub fn braille_quad_slice(character: char) -> Option<&'static [Quad]> {
    if !('\u{2800}'..='\u{28ff}').contains(&character) {
        return None;
    }
    Some(BRAILLE_QUADS[character as usize - 0x2800].quads())
}

/// Iterate the precomputed quads without allocation or pattern reconstruction.
pub fn braille_quads(character: char) -> Option<impl Iterator<Item = Quad>> {
    Some(braille_quad_slice(character)?.iter().copied())
}

/// Precomputed solid Block Elements. The three shade patterns have no quad data.
pub static BLOCK_QUADS: [Option<BlockGlyph>; 32] = crate::blocks::rasterize();

/// Borrow one or two filled rectangles; shades and non-blocks return `None`.
pub fn block_quad_slice(character: char) -> Option<&'static [Quad]> {
    if !('\u{2580}'..='\u{259f}').contains(&character) {
        return None;
    }
    BLOCK_QUADS[character as usize - 0x2580].as_ref().map(BlockGlyph::quads)
}

/// Iterate precomputed solid block rectangles without allocation.
pub fn block_quads(character: char) -> Option<impl Iterator<Item = Quad>> {
    Some(block_quad_slice(character)?.iter().copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FWIDTH;

    #[test]
    fn solid_blocks_use_at_most_two_exact_nonoverlapping_quads() {
        for (ch, expected) in crate::BLOCK_SYMBOLS {
            let quads = block_quad_slice(ch);
            if ('░'..='▓').contains(&ch) {
                assert!(quads.is_none());
                assert_ne!(crate::glyph_cell_pixels(ch), 0);
                continue;
            }
            let quads = quads.unwrap();
            assert!((1..=2).contains(&quads.len()));
            let mut pixels = 0;
            for quad in quads {
                let [left, top] = quad.top_left;
                let [right, bottom] = quad.bottom_right;
                assert!(left < right && right <= 6 && top < bottom && bottom <= 11);
                for y in top..bottom {
                    for x in left..right {
                        let bit = 1u128 << (65 - (y as usize * FWIDTH + x as usize));
                        assert_eq!(pixels & bit, 0);
                        pixels |= bit;
                    }
                }
            }
            assert_eq!(pixels, expected, "{ch}");
            assert_eq!(pixels, crate::glyph_cell_pixels(ch));
            let count = quads.len();
            if !('▖'..='▟').contains(&ch) { assert_eq!(count, 1); }
        }
        assert!(block_quad_slice('A').is_none());
    }

    #[test]
    fn first_dot_is_one_source_cell_quad() {
        let mut quads = braille_quads('⠁').unwrap();
        assert_eq!(quads.next(), Some(Quad { top_left: [1, 1], bottom_right: [2, 2] }));
        assert_eq!(quads.next(), None);
        assert_eq!(braille_quads('⠀').unwrap().count(), 0);
        assert!(braille_quads('A').is_none());
    }

    #[test]
    fn all_braille_quads_preserve_the_existing_pixels() {
        let dots = [(1, 1), (1, 3), (1, 5), (4, 1), (4, 3), (4, 5), (1, 7), (4, 7)];
        for pattern in 0u32..256 {
            let ch = char::from_u32(0x2800 + pattern).unwrap();
            let mut actual = 0u128;
            let mut count = 0;
            for quad in braille_quads(ch).unwrap() {
                let [x, y] = quad.top_left;
                assert_eq!(quad.bottom_right, [x + 1, y + 1]);
                actual |= 1u128 << (65 - (y as usize * FWIDTH + x as usize));
                count += 1;
            }
            let expected = dots.iter().enumerate().fold(0u128, |bits, (dot, &(x, y))| {
                bits | if pattern & (1 << dot) != 0 { 1 << (65 - (y * FWIDTH + x)) } else { 0 }
            });
            assert_eq!(count, pattern.count_ones());
            assert_eq!(actual, expected);
            assert_eq!(crate::glyph_cell_pixels(ch), expected);
        }
    }
}
