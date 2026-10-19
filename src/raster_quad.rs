//! Raster seeds and small-glyph quads in the 6×11 source cell.
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

impl BoxDrawQuad {
    const fn rectangle(quad: Quad) -> Self {
        let [left, top] = quad.top_left;
        let [right, bottom] = quad.bottom_right;
        Self { vertices: [
            [left as f32, top as f32], [right as f32, top as f32],
            [right as f32, bottom as f32], [left as f32, bottom as f32],
        ] }
    }
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
pub struct QuadRaster<const N: usize, const O: usize, Q = Quad> {
    pub quads: [Q; N],
    pub offsets: [u16; O],
}

pub type BlockRaster<const N: usize> = QuadRaster<N, 33>;
pub type BoxDrawRaster<const N: usize> = QuadRaster<N, 129, BoxDrawQuad>;

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

const BOX_DRAW_RECTANGLE_COUNT: usize = rasterize::<0, 129>(&crate::boxdraw::RASTER_SYMBOLS).offsets[128] as usize;
const BOX_DRAW_RECTANGLES: QuadRaster<BOX_DRAW_RECTANGLE_COUNT, 129> = rasterize(&crate::boxdraw::RASTER_SYMBOLS);

/// Exact rectangles for 121 straight glyphs plus five continuous diagonal
/// quads. Kept separate from the active glyph lookup and renderer.
pub const BOX_DRAW_QUAD_COUNT: usize = BOX_DRAW_RECTANGLE_COUNT + 5;
pub static BOX_DRAW_QUADS: BoxDrawRaster<BOX_DRAW_QUAD_COUNT> = boxdraw_rasterize();

/// Draft quad data only; rounded corners and non-box characters return `None`.
/// Diagonals are continuous ribbons rather than staircase pixel rectangles.
pub fn boxdraw_quad_slice(character: char) -> Option<&'static [BoxDrawQuad]> {
    let index = crate::BOX_DRAW_SYMBOLS.binary_search_by_key(&character, |&(ch, _)| ch).ok()?;
    let start = BOX_DRAW_QUADS.offsets[index] as usize;
    let end = BOX_DRAW_QUADS.offsets[index + 1] as usize;
    (start != end).then_some(&BOX_DRAW_QUADS.quads[start..end])
}

const fn boxdraw_rasterize() -> BoxDrawRaster<BOX_DRAW_QUAD_COUNT> {
    let empty = BoxDrawQuad { vertices: [[0.0; 2]; 4] };
    let mut data = BoxDrawRaster { quads: [empty; BOX_DRAW_QUAD_COUNT], offsets: [0; 129] };
    let mut count = 0;
    let mut i = 0;
    while i < crate::BOX_DRAW_SYMBOLS.len() {
        let manual = crate::boxdraw::diagonal_quads(crate::BOX_DRAW_SYMBOLS[i].0);
        let mut j = 0;
        while j < manual.len() {
            data.quads[count] = manual[j];
            count += 1;
            j += 1;
        }
        let mut j = BOX_DRAW_RECTANGLES.offsets[i] as usize;
        while j < BOX_DRAW_RECTANGLES.offsets[i + 1] as usize {
            data.quads[count] = BoxDrawQuad::rectangle(BOX_DRAW_RECTANGLES.quads[j]);
            count += 1;
            j += 1;
        }
        data.offsets[i + 1] = count as u16;
        i += 1;
    }
    data
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
