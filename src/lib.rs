#![no_std]

#[cfg(test)]
extern crate std;

mod basic;
pub use basic::{BASIC_SYMBOLS, CONTROL_SYMBOLS, control_label_cells};
mod fontshape;
pub use fontshape::FontShape;
mod font;
pub use font::Font;
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
pub mod raster;
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
const fn z(bits: u64, shift: u32) -> u64 {
    bits << shift
}
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

// Extended CP850 masks not owned by basic.rs, boxdraw.rs or blocks.rs.
const CORE_SYMBOLS: [(char, u64); 100] = [
    ('Ç', 0x7228208208227084),
    ('ü', 0x0005008A28A27800),
    ('é', 0x004200722FA07000),
    ('â', 0x0085007027A27000),
    ('ä', 0x0005007027A27000),
    ('à', 0x0102007027A27000),
    ('å', 0x2142007027A27000),
    ('ç', 0x0000007A08207884),
    ('ê', 0x008500722FA07000),
    ('ë', 0x000500722FA07000),
    ('è', 0x010200722FA07000),
    ('ï', 0x0005002082082000),
    ('î', 0x0085002082082000),
    ('ì', 0x0102002082082000),
    ('Ä', 0xA8851453E8A28800),
    ('Å', 0x21421453E8A28800),
    ('É', 0x108FA083C820F800),
    ('æ', z(0x3627FA17, 10)),
    ('Æ', 0x5D4B2CFA49249C00),
    ('ô', 0x0085007228A27000),
    ('ö', 0x0005007228A27000),
    ('ò', 0x0102007228A27000),
    ('û', 0x0085008A28A27800),
    ('ù', 0x0102008A28A27800),
    ('ÿ', 0x0005008A25142084),
    ('Ö', 0x500FA28A28A2F800),
    ('Ü', 0x5008A28A28A27000),
    ('ø', 0x002726AB27200000),
    ('£', 0x390410F10410F800),
    ('Ø', 0x082726AB27208000),
    ('×', z(0x11284291, 23)),
    ('ƒ', 0x188208708208C000),
    ('á', 0x0042007027A27000),
    ('í', 0x0042002082082000),
    ('ó', 0x0042007228A27000),
    ('ú', 0x0042008A28A27800),
    ('ñ', 0x42A1007124924800),
    ('Ñ', 0x42A122CAAAA68800),
    ('ª', 0x604624700F800000),
    ('º', 0x7228A2700F800000),
    ('¿', 0x2080082108A27000),
    ('®', 0x01E86DAEDAE17800),
    ('¬', z(0x7C1, 29)),
    ('½', 0x4B25142164A4B800),
    ('¼', 0x4B25142125AF8800),
    ('¡', 0x2080082082082000),
    ('«', z(0x94A4489, 22)),
    ('»', z(0x9122529, 24)),
    ('Á', 0x108008514FA28800),
    ('Â', 0x214008514FA28800),
    ('À', 0x408008514FA28800),
    ('©', 0x01E867A699E17800),
    ('¢', 0x08472AA2A7108000),
    ('¥', 0x8A251421C21C2000),
    ('ã', 0x42A1007027A27000),
    ('Ã', 0x42A108514FA28800),
    ('¤', 0x0227228A27220000),
    ('ð', 0xA10A047228A27000),
    ('Ð', 0x712492E924927000),
    ('Ê', 0x214FA083C820F800),
    ('Ë', 0x500FA083C820F800),
    ('È', 0x408FA083C820F800),
    ('ı', z(0xC10411F, 11)),
    ('Í', 0x10803E208208F800),
    ('Î', 0x21403E208208F800),
    ('Ï', 0x01403E208208F800),
    ('¦', 0x2082080002082080),
    ('Ì', 0x40803E208208F800),
    ('Ó', 0x10803E8A28A2F800),
    ('ß', 0x3124944925104000),
    ('Ô', 0x21403E8A28A2F800),
    ('Ò', 0x40803E8A28A2F800),
    ('õ', 0x42A1007228A27000),
    ('Õ', 0x42A13E8A28A2F800),
    ('µ', 0x0000008A28A2FA08),
    ('þ', 0x82082CCA28A2F208),
    ('Þ', 0x000820F228A2F208),
    ('Ú', 0x1088A28A28A2F800),
    ('Û', 0x2140228A28A2F800),
    ('Ù', 0x4088A28A28A2F800),
    ('ý', 0x0042008A25142084),
    ('Ý', 0x12A8945082082000),
    ('¯', z(0x1F, 59)),
    ('´', z(0x21, 55)),
    ('\u{ad}', z(0x7, 36)),
    ('±', 0x00823E20803E0000),
    ('‗', z(0x1F01F, 5)),
    ('¾', 0xC49C8AD0A3974800),
    ('¶', 0x7BAEBA6820A27000),
    ('§', 0x31240C49230248C0),
    ('÷', z(0x1007C01, 25)),
    ('¸', z(0x41, 7)),
    ('°', z(0x7147, 48)),
    ('¨', z(0x5, 54)),
    ('·', z(0x1, 37)),
    ('¹', z(0x2182087, 36)),
    ('³', z(0x3023023, 37)),
    ('²', z(0x6042107, 36)),
    ('■', z(0x3FFFFFFF, 22)),
    ('\u{a0}', 0),
];

// Used only while assembling FPIXELS; table order does not determine byte slots.
const fn core_pixels(character: char) -> u64 {
    let mut index = 0;
    while index < CORE_SYMBOLS.len() {
        let (symbol, pixels) = CORE_SYMBOLS[index];
        if symbol == character {
            return pixels;
        }
        index += 1;
    }
    panic!("missing CP850 glyph mask")
}

/// Legacy CP850 atlas view, assembled at compile time from the canonical masks.
/// Slots retain their byte identities; the final two pixels of each 6×11 cell
/// are omitted by this 64-bit compatibility format.
pub const FPIXELS: [u64; 224] = {
    let mut pixels = [0; 224];
    let mut slot = 0;
    while slot < pixels.len() {
        let character = if slot < 96 { (slot as u8 + 32) as char }
            else { EXTENDED_CHARS[slot - 96] };
        pixels[slot] = match symbol_pixels(character) {
            Some(mask) => (mask >> 2) as u64,
            None => core_pixels(character),
        };
        slot += 1;
    }
    pixels
};
// Atlas slots 0x80..0xFF follow CP850, not Unicode/Latin-1.
// https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP850.TXT
const EXTENDED_CHARS: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ',
    'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', 'ø', '£', 'Ø', '×', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ',
    'ª', 'º', '¿', '®', '¬', '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À', '©',
    '╣', '║', '╗', '╝', '¢', '¥', '┐', '└', '┴', '┬', '├', '─', '┼', 'ã', 'Ã', '╚', '╔', '╩', '╦',
    '╠', '═', '╬', '¤', 'ð', 'Ð', 'Ê', 'Ë', 'È', 'ı', 'Í', 'Î', 'Ï', '┘', '┌', '█', '▄', '¦', 'Ì',
    '▀', 'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', 'þ', 'Þ', 'Ú', 'Û', 'Ù', 'ý', 'Ý', '¯', '´', '\u{ad}',
    '±', '‗', '¾', '¶', '§', '÷', '¸', '°', '¨', '·', '¹', '³', '²', '■', '\u{a0}',
];

/// Return the existing atlas slot for one Unicode character. Unsupported
/// characters occupy one replacement cell; byte APIs retain raw atlas indexing.
pub fn glyph_byte(character: char) -> u8 {
    if character.is_ascii() {
        character as u8
    } else {
        EXTENDED_CHARS
            .iter()
            .position(|&entry| entry == character)
            .map_or(b'?', |index| 0x80 + index as u8)
    }
}
pub fn font_pixels(character: u8) -> u64 {
    FPIXELS[(i32::from(character) - 32).clamp(0, FPIXELS.len() as i32 - 1) as usize]
}
/// Unicode identity of a raw packed atlas slot, including CP850 entries.
pub fn glyph_character(byte: u8) -> Option<char> {
    match byte {
        0x20..=0x7f => Some(byte as char),
        0x80..=0xff => Some(EXTENDED_CHARS[(byte - 0x80) as usize]),
        _ => None,
    }
}
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
