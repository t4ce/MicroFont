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

    /// Only the filled quads, in generation order.
    pub fn quads(&self) -> &[Quad] {
        &self.quads[..self.len as usize]
    }
}

/// At most eight single-dot quads per Braille glyph.
pub type BrailleGlyph = QuadGlyph<8>;
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

/// Packed rectangles and glyph boundaries in Block Elements Unicode order.
/// Glyph `i` uses `quads[offsets[i]..offsets[i + 1]]`.
pub struct BlockRaster<const N: usize> {
    pub quads: [Quad; N],
    pub offsets: [u16; 33],
}

/// Number of rectangles in the exact packed cover of all 32 block masks.
pub const BLOCK_QUAD_COUNT: usize = crate::blocks::rasterize::<0>().offsets[32] as usize;
/// All blocks use the same compile-time cover and share one packed table.
pub static BLOCK_QUADS: BlockRaster<BLOCK_QUAD_COUNT> = crate::blocks::rasterize();

/// Borrow exact filled rectangles for any Block Element; non-blocks return `None`.
pub fn block_quad_slice(character: char) -> Option<&'static [Quad]> {
    if !('\u{2580}'..='\u{259f}').contains(&character) {
        return None;
    }
    let index = character as usize - 0x2580;
    Some(&BLOCK_QUADS.quads[
        BLOCK_QUADS.offsets[index] as usize..BLOCK_QUADS.offsets[index + 1] as usize
    ])
}

/// Iterate precomputed block rectangles, including shades, without allocation.
pub fn block_quads(character: char) -> Option<impl Iterator<Item = Quad>> {
    Some(block_quad_slice(character)?.iter().copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FWIDTH;

    #[test]
    fn all_blocks_have_exact_nonoverlapping_quads() {
        for (ch, expected) in crate::BLOCK_SYMBOLS {
            let quads = block_quad_slice(ch).unwrap();
            assert!((1..=66).contains(&quads.len()));
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
