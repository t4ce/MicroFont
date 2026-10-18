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

/// Packed rectangles and glyph boundaries in their source table order.
/// Glyph `i` uses `quads[offsets[i]..offsets[i + 1]]`.
pub struct QuadRaster<const N: usize, const O: usize> {
    pub quads: [Quad; N],
    pub offsets: [u16; O],
}

pub type BlockRaster<const N: usize> = QuadRaster<N, 33>;
pub type BoxDrawRaster<const N: usize> = QuadRaster<N, 122>;

/// Number of rectangles in the exact packed cover of all 32 block masks.
pub const BLOCK_QUAD_COUNT: usize = rasterize::<0, 33>(&crate::BLOCK_SYMBOLS).offsets[32] as usize;
/// All blocks use the same compile-time cover and share one packed table.
pub static BLOCK_QUADS: BlockRaster<BLOCK_QUAD_COUNT> = rasterize(&crate::BLOCK_SYMBOLS);

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

/// Packed cover of the 121 straight Box Drawing drafts, with the same algorithm
/// as Block Elements. Kept separate from the active glyph lookup and renderer.
pub const BOX_DRAW_QUAD_COUNT: usize = rasterize::<0, 122>(&crate::BOX_DRAW_SYMBOLS).offsets[121] as usize;
pub static BOX_DRAW_QUADS: BoxDrawRaster<BOX_DRAW_QUAD_COUNT> = rasterize(&crate::BOX_DRAW_SYMBOLS);

/// Draft quad data only; excluded curves/diagonals and non-box characters return `None`.
pub fn boxdraw_quad_slice(character: char) -> Option<&'static [Quad]> {
    let index = crate::BOX_DRAW_SYMBOLS.binary_search_by_key(&character, |&(ch, _)| ch).ok()?;
    Some(&BOX_DRAW_QUADS.quads[
        BOX_DRAW_QUADS.offsets[index] as usize..BOX_DRAW_QUADS.offsets[index + 1] as usize
    ])
}

// Shared exact rectangle cover, evaluated at compile time. A count-only pass
// sizes each packed table; a second pass fills it. No glyph-specific rules.
const fn rasterize<const N: usize, const O: usize>(symbols: &[(char, u128)]) -> QuadRaster<N, O> {
    let empty = Quad { top_left: [0, 0], bottom_right: [0, 0] };
    let mut data = QuadRaster { quads: [empty; N], offsets: [0; O] };
    let mut count = 0;
    let mut i = 0;
    while i < symbols.len() {
        let mut pixels = symbols[i].1;
        while pixels != 0 {
            let mut first = 0;
            while pixels & (1u128 << (65 - first)) == 0 { first += 1; }
            let left = (first % 6) as u8;
            let top = (first / 6) as u8;
            let mut best = Quad { top_left: [left, top], bottom_right: [left + 1, top + 1] };
            let mut best_area = 1;
            let mut bottom = top + 1;
            while bottom <= 11 {
                let mut right = left + 1;
                while right <= 6 {
                    let quad = Quad { top_left: [left, top], bottom_right: [right, bottom] };
                    let covered = quad_pixels(quad);
                    let area = (right - left) * (bottom - top);
                    if covered & pixels == covered && area > best_area {
                        best = quad;
                        best_area = area;
                    }
                    right += 1;
                }
                bottom += 1;
            }
            if N != 0 { data.quads[count] = best; }
            count += 1;
            pixels ^= quad_pixels(best);
        }
        data.offsets[i + 1] = count as u16;
        i += 1;
    }
    data
}

const fn quad_pixels(quad: Quad) -> u128 {
    let mut pixels = 0;
    let mut y = quad.top_left[1];
    while y < quad.bottom_right[1] {
        let mut x = quad.top_left[0];
        while x < quad.bottom_right[0] {
            pixels |= 1u128 << (65 - (y as usize * 6 + x as usize));
            x += 1;
        }
        y += 1;
    }
    pixels
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
