//! CP850 byte/Unicode identities. Stored masks live only in the glyph bundle.

use crate::CP850_CHARACTERS;

/// Return the existing atlas slot for one Unicode character. Unsupported
/// characters occupy one replacement cell; byte APIs retain raw atlas indexing.
pub fn glyph_byte(character: char) -> u8 {
    if character.is_ascii() {
        character as u8
    } else {
        CP850_CHARACTERS[0x80..]
            .iter()
            .position(|&entry| entry == character)
            .map_or(b'?', |index| 0x80 + index as u8)
    }
}

/// Legacy byte mask; control bytes below 0x20 retain the blank space mask.
/// The 64-bit compatibility view omits the final two cell pixels.
#[cfg(feature = "hex")]
pub fn font_pixels(character: u8) -> u64 {
    let byte = character.max(0x20) as usize;
    (crate::bundle::bitmap(CP850_CHARACTERS[byte]).unwrap_or(0) >> 2) as u64
}

/// Unicode identity of a legacy atlas slot: ASCII (including DEL), then CP850.
/// `CP850_CHARACTERS` also exposes the graphical control slots.
pub fn glyph_character(byte: u8) -> Option<char> {
    match byte {
        0x20..=0x7f => Some(byte as char),
        0x80..=0xff => Some(CP850_CHARACTERS[byte as usize]),
        _ => None,
    }
}
