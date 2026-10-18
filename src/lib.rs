#![no_std]

#[cfg(test)]
extern crate std;

mod gui_han;
mod ui_symbols;
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

/// Icons composed of consecutive 6x11 cell masks (top-left at bit 65).
pub const SPECIAL: [(char, &'static [u128]); 2] = [
    ('\u{2328}', &[0x007E0AB287B81F000, 0x00F81BC3AF507E000]),
    ('\u{1F5FA}', &[0x1F82FFAE8E186189F, 0x1F826BC70C78E289F, 0x1E87DF7DE51BED85E]),
];

/// Internal cell characters used by fixed-cell terminal renderers: keyboard
/// left/right, then the map's three cells, all in left-to-right order.
pub fn special_cells(character: char) -> Option<&'static [char]> {
    match character {
        '\u{2328}' => Some(&['\u{e000}', '\u{e001}']),
        '\u{1f5fa}' => Some(&['\u{e002}', '\u{e003}', '\u{e004}']),
        _ => None,
    }
}

pub const FPIXELS: [u64; 224] = [
    0,
    0x2082082080082000,
    z(0x5145, 48),
    0x514F94514F945000,
    0x21CAA870AA9C2000,
    0x4AA50421052A9000,
    0x2145085628A27400,
    z(0x1041, 49),
    0x0842082082040800,
    0x4081041041084000,
    z(0x54E395, 41),
    z(0x1047C41, 25),
    z(0x821, 7),
    z(0x1F, 35),
    z(0xC3, 12),
    0x0821042104208000,
    0x7228A28A28A27000,
    0x0862928820820800,
    0x722882108420F800,
    0x7228823028A27000,
    0x10C51493E1041000,
    0xFA083C082082F000,
    0x722820F228A27000,
    0xF821042084104000,
    0x7228A27228A27000,
    0x7228A27820A27000,
    z(0xC30000C3, 18),
    0x00030C00030C1080,
    0x0021084081020000,
    z(0x1F01F, 29),
    0x0204081084200000,
    0x7228842080082000,
    0x7A1B75D75D6E81E0,
    0x20851453E8A28800,
    0xF228A2F228A2F000,
    0x7228208208227000,
    0xF228A28A28A2F000,
    0xFA0820F20820F800,
    0xFA0820F208208000,
    0x722820B228A27000,
    0x8A28A2FA28A28800,
    0xF88208208208F800,
    0xF820820820A27000,
    0x8A4928C289248800,
    0x820820820820F800,
    0x8B6DAAAA28A28800,
    0x8B2CAAAAA9A68800,
    0xFA28A28A28A2F800,
    0xF228A2F208208000,
    0x7228A28A28A46800,
    0xF228A2F289228800,
    0x7228207020A27000,
    0xF882082082082000,
    0x8A28A28A28A27000,
    0x8A28A25145082000,
    0x8A28A28AAA945000,
    0x8A25142145228800,
    0x8A25142082082000,
    0xF82104210420F800,
    0x3882082082083800,
    0x8204102041020800,
    0xE08208208208E000,
    z(0x4291, 47),
    z(0x1F, 11),
    z(0x81, 54),
    z(0x7027A27, 12),
    0x820820F228A2F000,
    z(0xF41040F, 11),
    0x0820827A28A27800,
    z(0x722FA07, 12),
    0x1882087082082000,
    0x0000007A28A27827,
    0x820820F228A28800,
    0x0002002082082000,
    0x0002002082082284,
    0x4104104946144800,
    0x4104104104103000,
    z(0x1E555555, 11),
    z(0x1E451451, 11),
    z(0x7228A27, 12),
    0x0000007124927104,
    0x0000007249247041,
    z(0x1C49041, 14),
    z(0x3907827, 12),
    0x2082087082081800,
    z(0x1145144F, 11),
    z(0x45128A1, 13),
    z(0x8AA7145, 12),
    z(0x11284291, 11),
    0x0000008A25142084,
    z(0x1F08421F, 11),
    0x0841042041040800,
    0x2082082082082000,
    0x8104102104108000,
    z(0x42A1, 30),
    0,
    0x7228208208227084,
    0x0005008A28A27800,
    0x004200722FA07000,
    0x0085007027A27000,
    0x0005007027A27000,
    0x0102007027A27000,
    0x2142007027A27000,
    0x0000007A08207884,
    0x008500722FA07000,
    0x000500722FA07000,
    0x010200722FA07000,
    0x0005002082082000,
    0x0085002082082000,
    0x0102002082082000,
    0xA8851453E8A28800,
    0x21421453E8A28800,
    0x108FA083C820F800,
    z(0x3627FA17, 10),
    0x5D4B2CFA49249C00,
    0x0085007228A27000,
    0x0005007228A27000,
    0x0102007228A27000,
    0x0085008A28A27800,
    0x0102008A28A27800,
    0x0005008A25142084,
    0x500FA28A28A2F800,
    0x5008A28A28A27000,
    0x002726AB27200000,
    0x390410F10410F800,
    0x082726AB27208000,
    z(0x11284291, 23),
    0x188208708208C000,
    0x0042007027A27000,
    0x0042002082082000,
    0x0042007228A27000,
    0x0042008A28A27800,
    0x42A1007124924800,
    0x42A122CAAAA68800,
    0x604624700F800000,
    0x7228A2700F800000,
    0x2080082108A27000,
    0x01E86DAEDAE17800,
    z(0x7C1, 29),
    0x4B25142164A4B800,
    0x4B25142125AF8800,
    0x2080082082082000,
    z(0x94A4489, 22),
    z(0x9122529, 24),
    0x9004802409004800,
    0xA95222568095A120,
    0xA95A95A95A95A950,
    0x2082082082082082,
    0x2082082382082082,
    0x108008514FA28800,
    0x214008514FA28800,
    0x408008514FA28800,
    0x01E867A699E17800,
    0x514514D04D145145,
    0x5145145145145145,
    0x000000F04D145145,
    0x514514D04F000000,
    0x08472AA2A7108000,
    0x8A251421C21C2000,
    z(0x1C1041041, 1),
    0x20820820F0000000,
    0x20820823F0000000,
    z(0x1F9041041, 1),
    0x20820820F2082082,
    z(0x3F, 28),
    0x20820823F2082082,
    0x42A1007027A27000,
    0x42A108514FA28800,
    0x5145145D07C00000,
    0x0000007D05D45145,
    0x514514DC0FC00000,
    0x000000FC0DD45145,
    0x5145145D05D45145,
    z(0x3F03F, 22),
    0x514514DC0DD45145,
    0x0227228A27220000,
    0xA10A047228A27000,
    0x712492E924927000,
    0x214FA083C820F800,
    0x500FA083C820F800,
    0x408FA083C820F800,
    z(0xC10411F, 11),
    0x10803E208208F800,
    0x21403E208208F800,
    0x01403E208208F800,
    z(0x41041047, 31),
    z(0x79041041, 1),
    0xFFFFFFFFFFFFFFF0,
    z(0x3FFFFFFF, 4),
    0x2082080002082080,
    0x40803E208208F800,
    z(0x3FFFFFFF, 34),
    0x10803E8A28A2F800,
    0x3124944925104000,
    0x21403E8A28A2F800,
    0x40803E8A28A2F800,
    0x42A1007228A27000,
    0x42A13E8A28A2F800,
    0x0000008A28A2FA08,
    0x82082CCA28A2F208,
    0x000820F228A2F208,
    0x1088A28A28A2F800,
    0x2140228A28A2F800,
    0x4088A28A28A2F800,
    0x0042008A25142084,
    0x12A8945082082000,
    z(0x1F, 59),
    z(0x21, 55),
    z(0x7, 36),
    0x00823E20803E0000,
    z(0x1F01F, 5),
    0xC49C8AD0A3974800,
    0x7BAEBA6820A27000,
    0x31240C49230248C0,
    z(0x1007C01, 25),
    z(0x41, 7),
    z(0x7147, 48),
    z(0x5, 54),
    z(0x1, 37),
    z(0x2182087, 36),
    z(0x3023023, 37),
    z(0x6042107, 36),
    z(0x3FFFFFFF, 22),
    0,
];
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
/// Full 6x11 Unicode cell, with the top-left pixel at bit 65 and bottom-right
/// at bit 0. Includes Braille, Block Elements, and the one-cell GUI Han subset.
pub fn glyph_cell_pixels(character: char) -> u128 {
    if ('\u{e000}'..='\u{e004}').contains(&character) {
        let cell = character as usize - 0xe000;
        if cell < 2 { SPECIAL[0].1[cell] } else { SPECIAL[1].1[cell - 2] }
    } else if ('\u{2580}'..='\u{259f}').contains(&character) {
        BLOCK_SYMBOLS[character as usize - 0x2580].1
    } else if ('\u{2800}'..='\u{28ff}').contains(&character) {
        (braille::DATA.pixels[character as usize - 0x2800] as u128) << 2
    } else if let Some(pixels) = ui_symbols::glyph_cell_pixels(character) {
        pixels
    } else if let Some(pixels) = gui_han::glyph_cell_pixels(character) {
        pixels
    } else {
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
