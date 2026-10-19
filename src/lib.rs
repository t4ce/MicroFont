#![no_std]

#[cfg(feature = "raster")]
extern crate alloc;

#[cfg(test)]
extern crate std;

mod basic;
pub use basic::{BASIC_SYMBOLS, CONTROL_SYMBOLS, control_label_cells};
mod cp850;
pub use cp850::{CP850_SYMBOLS, FPIXELS, font_pixels, glyph_byte, glyph_character};
mod fontshape;
pub use fontshape::FontShape;
#[cfg(feature = "han")]
mod gui_han;
mod braille;
mod blocks;
mod boxdraw;
pub use boxdraw::BOX_DRAW_SYMBOLS;
pub use blocks::BLOCK_SYMBOLS;
#[cfg(feature = "stamp")]
mod stamp;
#[cfg(feature = "raster")]
pub mod raster_quad;
#[cfg(feature = "raster")]
pub mod raster_line;
#[cfg(feature = "stamp")]
pub use stamp::{
    stamp_argb, stamp_argb_stride, stamp_bytes, stamp_bytes_with_stride,
    stamp_text, stamp_text_with_stride,
};
#[cfg(feature = "han")]
pub use gui_han::{GUI_HAN_COUNT, GUI_HAN_TABLE_BYTES, characters as gui_han_characters,
    glyph_cell_pixels as gui_han_cell_pixels};

pub const FHEIGHT: usize = 11;
pub const FWIDTH: usize = 6;
pub const ERRINV: &str = "invalid dimensions";
pub const ERRBUFF: &str = "buffer too small";
pub const ULINE: u8 = 1;
pub const STRIKE: u8 = 2;
pub const FLIPY: u8 = 4;
const fn z128(bits: u128, shift: u32) -> u128 {
    bits << shift
}

/// Icons composed of consecutive 6x11 cell masks (top-left at bit 65).
pub const SPECIAL: [(char, &'static [u128]); 2] = [
    ('\u{2328}', &[0x007E0AB287B81F000, 0x00F81BC3AF507E000]),
    ('\u{1F5FA}', &[0x1F82FFAE8E186189F, 0x1F826BC70C78E289F, 0x1E87DF7DE51BED85E]),
];

/// Multi-cell expansion: private cells for keyboard/map, existing Basic Latin
/// letters for control names, including `SP` for spaces.
pub fn special_cells(character: char) -> Option<&'static [char]> {
    match character {
        '\u{2328}' => Some(&['\u{e000}', '\u{e001}']),
        '\u{1f5fa}' => Some(&['\u{e002}', '\u{e003}', '\u{e004}']),
        _ => basic::control_characters(character),
    }
}

// Additional 6×11 UI symbols; top-left at bit 65.
const UI_SYMBOLS: [(char, u128); 36] = [
    ('ᗰ', 0x00033cedb61861000),
    ('→', 0x00000102fc2100000),
    ('⇝', 0x00000112b42100000),
    ('⎛', 0x02108208208208208),
    ('⎜', 0x08208208208208208),
    ('⎝', 0x08208208208208102),
    ('␛', 0x38c3800c218006206),
    ('␡', 0x30a3001c61c00820e),
    ('◇', 0x000085228a2508000),
    ('★', 0x0000823e71c522000),
    ('☩', 0x00708abea88700000),
    ('☰', 0x0003f000fc003f000),
    ('⫸', 0x2042852a56a528420),
    ('・', 0x0000000c300000000),
    ('＃', 0x1453e51453e514000),
    ('Ｎ', 0x00031c69b658e3000),
    ('Ｕ', 0x0002186186187f000),
    ('🖈', 0x00f9c73ef88208000),
    ('🖹', 0x00f269e1b61b61fc0),
    ('🖿', 0x00038fffffffff000),
    ('🗑', 0x0073e02aaaaabe000),
    ('🗜', 0x007a0ba4ba4784380),
    ('🞁', 0x0000000873e000000),
    ('🞃', 0x0003e708000000000),
    ('🪦', 0x00722abeaa28be000),
    ('🯀', 0x00033b52312b73000),
    ('🯰', 0x007228a20228a2700),
    ('🯱', 0x00002082002082000),
    ('🯲', 0x00702082720820700),
    ('🯳', 0x00702082702082700),
    ('🯴', 0x000228a2702082000),
    ('🯵', 0x00720820702082700),
    ('🯶', 0x007208207228a2700),
    ('🯷', 0x00702082002082000),
    ('🯸', 0x007228a27228a2700),
    ('🯹', 0x007228a2702082700),
];

// Canonical complete-cell symbols, shared by Unicode lookup and the CP850 view.
const fn symbol_pixels(character: char) -> Option<u128> {
    let code = character as u32;
    if code <= 0x7f {
        Some(BASIC_SYMBOLS[code as usize].1)
    } else if code >= 0x2500 && code <= 0x257f {
        Some(BOX_DRAW_SYMBOLS[(code - 0x2500) as usize].1)
    } else if code >= 0x2580 && code <= 0x259f {
        Some(BLOCK_SYMBOLS[(code - 0x2580) as usize].1)
    } else {
        None
    }
}

/// Full 6x11 Unicode cell, with the top-left pixel at bit 65 and bottom-right
/// at bit 0. Includes Box Drawing, Braille, Block Elements, and (with `han`) GUI Han.
pub fn glyph_cell_pixels(character: char) -> u128 {
    if ('\u{e000}'..='\u{e004}').contains(&character) {
        let cell = character as usize - 0xe000;
        if cell < 2 { SPECIAL[0].1[cell] } else { SPECIAL[1].1[cell - 2] }
    } else if let Some(pixels) = symbol_pixels(character) {
        pixels
    } else if ('\u{2800}'..='\u{28ff}').contains(&character) {
        (braille::DATA.pixels[character as usize - 0x2800] as u128) << 2
    } else if let Ok(index) = UI_SYMBOLS.binary_search_by_key(&character, |&(code, _)| code) {
        UI_SYMBOLS[index].1
    } else {
        #[cfg(feature = "han")]
        if let Some(pixels) = gui_han::glyph_cell_pixels(character) {
            return pixels;
        }
        (font_pixels(glyph_byte(character)) as u128) << 2
    }
}

/// Legacy 64-bit Unicode mask (top-left pixel at bit 63). The final two cell
/// pixels are omitted; use `glyph_cell_pixels` for complete block glyphs.
pub fn glyph_pixels(character: char) -> u64 {
    (glyph_cell_pixels(character) >> 2) as u64
}
pub fn measure_text(text: &str) -> (usize, usize) {
    (text.chars().map(|ch| special_cells(ch).map_or(1, |cells| cells.len())).sum::<usize>() * FWIDTH, FHEIGHT)
}
pub fn measure_bytes(bytes: &[u8]) -> (usize, usize) {
    (bytes.len() * FWIDTH, FHEIGHT)
}
