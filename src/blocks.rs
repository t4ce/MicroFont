//! Block Elements in Unicode order; full 6×11 masks, top-left at bit 65.
//! Solid blocks use at most two quads; shades use an exact rectangle cover.

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

// The same greedy, non-overlapping cover for every block, evaluated at compile
// time. A count-only pass sizes the packed table; the second pass fills it.
#[cfg(feature = "raster")]
pub(crate) const fn rasterize<const N: usize>() -> crate::raster::BlockRaster<N> {
    use crate::raster::{BlockRaster, Quad};
    let empty = Quad { top_left: [0, 0], bottom_right: [0, 0] };
    let mut data = BlockRaster { quads: [empty; N], offsets: [0; 33] };
    let mut count = 0;
    let mut i = 0;
    while i < BLOCK_SYMBOLS.len() {
        let mut pixels = BLOCK_SYMBOLS[i].1;
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
