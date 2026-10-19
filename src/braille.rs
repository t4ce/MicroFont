//! all 256 Braille patterns have precomputed raster data: 
//! one 1×1 quad per lit dot, up to eight quads per glyph.
use crate::FWIDTH;

// Unicode dot order: left 1/2/3/7, right 4/5/6/8.
const DOTS: [(u8, u8); 8] = [
    (1, 1), (1, 3), (1, 5), (4, 1), (4, 3), (4, 5), (1, 7), (4, 7),
];

pub(crate) struct BrailleData {
    pub pixels: [u64; 256],
    #[cfg(feature = "raster")]
    pub quads: [crate::raster_quad::BrailleGlyph; 256],
}

pub(crate) const DATA: BrailleData = generate();

const fn generate() -> BrailleData {
    let mut data = BrailleData {
        pixels: [0; 256],
        #[cfg(feature = "raster")]
        quads: [crate::raster_quad::BrailleGlyph::EMPTY; 256],
    };
    let mut pattern = 0;
    while pattern < 256 {
        let mut dot = 0;
        while dot < DOTS.len() {
            let (x, y) = DOTS[dot];
            if pattern & (1 << dot) != 0 {
                data.pixels[pattern] |= 1u64 << (63 - (y as usize * FWIDTH + x as usize));
                #[cfg(feature = "raster")]
                {
                    let glyph = &mut data.quads[pattern];
                    glyph.quads[glyph.len as usize] = crate::raster_quad::Quad {
                        top_left: [x, y], bottom_right: [x + 1, y + 1],
                    };
                    glyph.len += 1;
                }
            }
            dot += 1;
        }
        pattern += 1;
    }
    data
}
