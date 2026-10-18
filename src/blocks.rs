//! Block Elements in Unicode order; full 6×11 masks, top-left at bit 65.
//! The three shade patterns remain pixel-only.

pub const BLOCK_SYMBOLS: [(char, u128); 32] = [
    ('▀', 0x3fffffff000000000),
    ('▁', 0x0000000000000003f),
    ('▂', 0x0000000000003ffff),
    ('▃', 0x00000000000ffffff),
    ('▄', 0x00000000fffffffff),
    ('▅', 0x0000003ffffffffff),
    ('▆', 0x00000ffffffffffff),
    ('▇', 0x00fffffffffffffff),
    ('█', 0x3ffffffffffffffff),
    ('▉', 0x3efbefbefbefbefbe),
    ('▊', 0x3efbefbefbefbefbe),
    ('▋', 0x3cf3cf3cf3cf3cf3c),
    ('▌', 0x38e38e38e38e38e38),
    ('▍', 0x30c30c30c30c30c30),
    ('▎', 0x30c30c30c30c30c30),
    ('▏', 0x20820820820820820),
    ('▐', 0x071c71c71c71c71c7),
    ('░', 0x22222222222222222),
    ('▒', 0x2a56a56a56a56a56a),
    ('▓', 0x1dddddddddddddddd),
    ('▔', 0x3f000000000000000),
    ('▕', 0x01041041041041041),
    ('▖', 0x00000000e38e38e38),
    ('▗', 0x000000001c71c71c7),
    ('▘', 0x38e38e38000000000),
    ('▙', 0x38e38e38fffffffff),
    ('▚', 0x38e38e381c71c71c7),
    ('▛', 0x3fffffffe38e38e38),
    ('▜', 0x3fffffff1c71c71c7),
    ('▝', 0x071c71c7000000000),
    ('▞', 0x071c71c7e38e38e38),
    ('▟', 0x071c71c7fffffffff),
];

// Extract exact rectangles from the embedded masks once, at compile time.
// First try one bounding rectangle; otherwise try a rectangle containing the
// first set pixel, requiring the remaining pixels to form one more rectangle.
#[cfg(feature = "raster")]
pub(crate) const fn rasterize() -> [Option<crate::raster::BlockGlyph>; 32] {
    use crate::raster::{BlockGlyph, Quad};
    let mut glyphs = [None; 32];
    let mut i = 0;
    while i < BLOCK_SYMBOLS.len() {
        if i >= 17 && i <= 19 {
            i += 1;
            continue;
        }
        let pixels = BLOCK_SYMBOLS[i].1;
        let bounds = bounding_quad(pixels);
        let mut glyph = BlockGlyph::EMPTY;
        if quad_pixels(bounds) == pixels {
            glyph.quads[0] = bounds;
            glyph.len = 1;
        } else {
            let mut first = 0;
            while pixels & (1u128 << (65 - first)) == 0 { first += 1; }
            let left = (first % 6) as u8;
            let top = (first / 6) as u8;
            let mut bottom = top + 1;
            while bottom <= 11 && glyph.len == 0 {
                let mut right = left + 1;
                while right <= 6 && glyph.len == 0 {
                    let quad = Quad { top_left: [left, top], bottom_right: [right, bottom] };
                    let covered = quad_pixels(quad);
                    if covered & pixels == covered {
                        let remainder = pixels ^ covered;
                        let second = bounding_quad(remainder);
                        if quad_pixels(second) == remainder {
                            glyph.quads = [quad, second];
                            glyph.len = 2;
                        }
                    }
                    right += 1;
                }
                bottom += 1;
            }
            assert!(glyph.len == 2, "solid Block Element needs more than two quads");
        }
        glyphs[i] = Some(glyph);
        i += 1;
    }
    glyphs
}

#[cfg(feature = "raster")]
const fn bounding_quad(pixels: u128) -> crate::raster::Quad {
    let mut quad = crate::raster::Quad { top_left: [6, 11], bottom_right: [0, 0] };
    let mut bit = 0;
    while bit < 66 {
        if pixels & (1u128 << (65 - bit)) != 0 {
            let x = (bit % 6) as u8;
            let y = (bit / 6) as u8;
            if x < quad.top_left[0] { quad.top_left[0] = x; }
            if y < quad.top_left[1] { quad.top_left[1] = y; }
            if x + 1 > quad.bottom_right[0] { quad.bottom_right[0] = x + 1; }
            if y + 1 > quad.bottom_right[1] { quad.bottom_right[1] = y + 1; }
        }
        bit += 1;
    }
    quad
}

#[cfg(feature = "raster")]
const fn quad_pixels(quad: crate::raster::Quad) -> u128 {
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
