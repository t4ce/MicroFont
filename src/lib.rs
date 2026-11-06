#![no_std]

#[cfg(test)]
extern crate std;

#[cfg(not(any(feature = "hex", feature = "raster")))]
compile_error!("enable at least one MicroFont representation: hex or raster");

pub mod bundle;
pub use bundle::{CP850_CHARACTERS, UNCOMPRESSED_BUNDLE_BYTES, install_bundle};
#[cfg(target_os = "trueos")]
pub use bundle::EMBEDDED_ARCHIVE;

mod basic;
pub use basic::{BASIC_CHARACTERS, CONTROL_CHARACTERS};
#[cfg(feature = "hex")]
pub use basic::{ControlLabelCells, control_label_cells};
mod cp850;
pub use cp850::{glyph_byte, glyph_character};
#[cfg(feature = "hex")]
pub use cp850::font_pixels;
#[path = "7seg.rs"]
pub mod seven_segment;
mod fontshape;
pub use fontshape::FontShape;
mod blocks;
mod boxdraw;
pub use boxdraw::BOX_DRAW_CHARACTERS;
pub use blocks::BLOCK_CHARACTERS;
#[cfg(feature = "stamp")]
mod stamp;
#[cfg(feature = "raster")]
pub mod raster_quad;
#[cfg(feature = "raster")]
pub mod raster_line;
#[cfg(feature = "raster")]
pub mod svg;
#[cfg(feature = "stamp")]
pub use stamp::{
    stamp_argb, stamp_argb_stride, stamp_bytes, stamp_bytes_with_stride,
    stamp_text, stamp_text_with_stride,
};

pub const FHEIGHT: usize = 11;
pub const FWIDTH: usize = 6;
pub const ERRINV: &str = "invalid dimensions";
pub const ERRBUFF: &str = "buffer too small";
pub const ULINE: u8 = 1;
pub const STRIKE: u8 = 2;
pub const FLIPY: u8 = 4;

/// Multi-cell icon identities. Their pixel masks live in the glyph bundle.
pub const SPECIAL_CHARACTERS: [(char, &'static [char]); 2] = [
    ('\u{2328}', &['\u{e000}', '\u{e001}']),
    ('\u{1f5fa}', &['\u{e002}', '\u{e003}', '\u{e004}']),
];

/// Multi-cell expansion: private cells for keyboard/map, existing Basic Latin
/// letters for control names, including `SP` for spaces.
pub fn special_cells(character: char) -> Option<&'static [char]> {
    match character {
        '\u{2328}' => Some(SPECIAL_CHARACTERS[0].1),
        '\u{1f5fa}' => Some(SPECIAL_CHARACTERS[1].1),
        _ => basic::control_characters(character),
    }
}

/// Full 6x11 Unicode cell, with the top-left pixel at bit 65 and bottom-right
/// at bit 0. Any stored Unicode bitmap is available, including graphical CP850
/// control-slot identities. Missing Unicode uses the CP850 replacement cell.
#[cfg(feature = "hex")]
pub fn glyph_cell_pixels(character: char) -> u128 {
    bundle::bitmap(character)
        .unwrap_or_else(|| (font_pixels(glyph_byte(character)) as u128) << 2)
}

/// Legacy 64-bit Unicode mask (top-left pixel at bit 63). The final two cell
/// pixels are omitted; use `glyph_cell_pixels` for complete block glyphs.
#[cfg(feature = "hex")]
pub fn glyph_pixels(character: char) -> u64 {
    (glyph_cell_pixels(character) >> 2) as u64
}

pub fn measure_text(text: &str) -> (usize, usize) {
    (text.chars().map(|ch| special_cells(ch).map_or(1, |cells| cells.len())).sum::<usize>() * FWIDTH, FHEIGHT)
}

pub fn measure_bytes(bytes: &[u8]) -> (usize, usize) {
    (bytes.len() * FWIDTH, FHEIGHT)
}

#[cfg(all(test, feature = "hex"))]
mod bitmap_lookup_tests {
    #[test]
    fn every_cp850_unicode_identity_uses_its_complete_stored_mask() {
        for character in crate::CP850_CHARACTERS {
            assert_eq!(crate::glyph_cell_pixels(character),
                crate::bundle::bitmap(character).unwrap(), "{character:?}");
        }
    }

    #[test]
    fn unicode_lookup_covers_stored_symbols_and_replaces_unknowns() {
        for character in ['←', '↑', '↓', '↔', '↕', '▲', '•', '○', '🯹', '⣿', '╭'] {
            assert_eq!(crate::glyph_cell_pixels(character),
                crate::bundle::bitmap(character).unwrap());
        }
        assert_eq!(crate::glyph_cell_pixels('界'), crate::glyph_cell_pixels('?'));
    }
}
