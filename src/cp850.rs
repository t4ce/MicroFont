//! Complete 256-slot CP850 display-code table, in byte order.
//! Full 6×11 masks use bit 65 for the top-left pixel. Unfinished glyphs are zero.
//! Basic Latin, box drawing, blocks and existing UI masks are mapped at compile
//! time from their owning tables; only the remaining extended masks live here.

use crate::z128 as z;

/// CP850 display identities, including graphical control slots and `⌂` at 0x7F.
/// Unicode control labels remain owned by `basic.rs`.
pub const CP850_SYMBOLS: [(char, u128); 256] = {
    let mut symbols = [
        // 0x00–0x0F
        ('\0', 0), // NUL
        ('☺', 0),
        ('☻', 0),
        ('♥', 0),
        ('♦', 0),
        ('♣', 0),
        ('♠', 0),
        ('•', 0),
        ('◘', 0),
        ('○', 0),
        ('◙', 0),
        ('♂', 0),
        ('♀', 0),
        ('♪', 0),
        ('♫', 0),
        ('☼', 0),

        // 0x10–0x1F
        ('►', 0),
        ('◄', 0),
        ('↕', 0),
        ('‼', 0),
        ('¶', 0), // Reuse 0xF4 below.
        ('§', 0), // Reuse 0xF5 below.
        ('▬', 0),
        ('↨', 0),
        ('↑', 0),
        ('↓', 0),
        mapped('→'),
        ('←', 0),
        ('∟', 0),
        ('↔', 0),
        ('▲', 0),
        ('▼', 0),

        // 0x20–0x2F
        mapped(' '),
        mapped('!'),
        mapped('"'),
        mapped('#'),
        mapped('$'),
        mapped('%'),
        mapped('&'),
        mapped('\''),
        mapped('('),
        mapped(')'),
        mapped('*'),
        mapped('+'),
        mapped(','),
        mapped('-'),
        mapped('.'),
        mapped('/'),

        // 0x30–0x3F
        mapped('0'),
        mapped('1'),
        mapped('2'),
        mapped('3'),
        mapped('4'),
        mapped('5'),
        mapped('6'),
        mapped('7'),
        mapped('8'),
        mapped('9'),
        mapped(':'),
        mapped(';'),
        mapped('<'),
        mapped('='),
        mapped('>'),
        mapped('?'),

        // 0x40–0x4F
        mapped('@'),
        mapped('A'),
        mapped('B'),
        mapped('C'),
        mapped('D'),
        mapped('E'),
        mapped('F'),
        mapped('G'),
        mapped('H'),
        mapped('I'),
        mapped('J'),
        mapped('K'),
        mapped('L'),
        mapped('M'),
        mapped('N'),
        mapped('O'),

        // 0x50–0x5F
        mapped('P'),
        mapped('Q'),
        mapped('R'),
        mapped('S'),
        mapped('T'),
        mapped('U'),
        mapped('V'),
        mapped('W'),
        mapped('X'),
        mapped('Y'),
        mapped('Z'),
        mapped('['),
        mapped('\\'),
        mapped(']'),
        mapped('^'),
        mapped('_'),

        // 0x60–0x6F
        mapped('`'),
        mapped('a'),
        mapped('b'),
        mapped('c'),
        mapped('d'),
        mapped('e'),
        mapped('f'),
        mapped('g'),
        mapped('h'),
        mapped('i'),
        mapped('j'),
        mapped('k'),
        mapped('l'),
        mapped('m'),
        mapped('n'),
        mapped('o'),

        // 0x70–0x7F
        mapped('p'),
        mapped('q'),
        mapped('r'),
        mapped('s'),
        mapped('t'),
        mapped('u'),
        mapped('v'),
        mapped('w'),
        mapped('x'),
        mapped('y'),
        mapped('z'),
        mapped('{'),
        mapped('|'),
        mapped('}'),
        mapped('~'),
        ('⌂', 0),

        // 0x80–0x8F
        ('Ç', 0x1c8a082082089c210),
        ('ü', 0x140228a289e000),
        ('é', 0x10801c8be81c000),
        ('â', 0x21401c09e89c000),
        ('ä', 0x1401c09e89c000),
        ('à', 0x40801c09e89c000),
        ('å', 0x850801c09e89c000),
        ('ç', 0x1e82081e210),
        ('ê', 0x21401c8be81c000),
        ('ë', 0x1401c8be81c000),
        ('è', 0x40801c8be81c000),
        ('ï', 0x14008208208000),
        ('î', 0x214008208208000),
        ('ì', 0x408008208208000),
        ('Ä', 0x2a214514fa28a2000),
        ('Å', 0x8508514fa28a2000),

        // 0x90–0x9F
        ('É', 0x423e820f2083e000),
        ('æ', z(0x3627fa17, 12)),
        ('Æ', 0x1752cb3e924927000),
        ('ô', 0x21401c8a289c000),
        ('ö', 0x1401c8a289c000),
        ('ò', 0x40801c8a289c000),
        ('û', 0x2140228a289e000),
        ('ù', 0x4080228a289e000),
        ('ÿ', 0x14022894508210),
        ('Ö', 0x1403e8a28a28be000),
        ('Ü', 0x140228a28a289c000),
        ('ø', 0x9c9aac9c800000),
        ('£', 0xe41043c41043e000),
        ('Ø', 0x209c9aac9c820000),
        ('×', z(0x11284291, 25)),
        ('ƒ', 0x620821c208230000),

        // 0xA0–0xAF
        ('á', 0x10801c09e89c000),
        ('í', 0x108008208208000),
        ('ó', 0x10801c8a289c000),
        ('ú', 0x1080228a289e000),
        ('ñ', 0x10a8401c492492000),
        ('Ñ', 0x10a848b2aaa9a2000),
        ('ª', 0x1811891c03e000000),
        ('º', 0x1c8a289c03e000000),
        ('¿', 0x820020842289c000),
        ('®', 0x7a1b6bb6b85e000),
        ('¬', z(0x7c1, 31)),
        ('½', 0x12c9450859292e000),
        ('¼', 0x12c94508496be2000),
        ('¡', 0x8200208208208000),
        ('«', z(0x94a4489, 24)),
        ('»', z(0x9122529, 26)),

        // 0xB0–0xBF
        mapped('░'),
        mapped('▒'),
        mapped('▓'),
        mapped('│'),
        mapped('┤'),
        ('Á', 0x420021453e8a2000),
        ('Â', 0x850021453e8a2000),
        ('À', 0x1020021453e8a2000),
        ('©', 0x7a19e9a6785e000),
        mapped('╣'),
        mapped('║'),
        mapped('╗'),
        mapped('╝'),
        ('¢', 0x211caa8a9c420000),
        ('¥', 0x22894508708708000),
        mapped('┐'),

        // 0xC0–0xCF
        mapped('└'),
        mapped('┴'),
        mapped('┬'),
        mapped('├'),
        mapped('─'),
        mapped('┼'),
        ('ã', 0x10a8401c09e89c000),
        ('Ã', 0x10a8421453e8a2000),
        mapped('╚'),
        mapped('╔'),
        mapped('╩'),
        mapped('╦'),
        mapped('╠'),
        mapped('═'),
        mapped('╬'),
        ('¤', 0x89c8a289c880000),

        // 0xD0–0xDF
        ('ð', 0x2842811c8a289c000),
        ('Ð', 0x1c4924ba49249c000),
        ('Ê', 0x853e820f2083e000),
        ('Ë', 0x1403e820f2083e000),
        ('È', 0x1023e820f2083e000),
        ('ı', z(0xc10411f, 13)),
        ('Í', 0x4200f8820823e000),
        ('Î', 0x8500f8820823e000),
        ('Ï', 0x500f8820823e000),
        mapped('┘'),
        mapped('┌'),
        mapped('█'),
        mapped('▄'),
        ('¦', 0x8208200008208200),
        ('Ì', 0x10200f8820823e000),
        mapped('▀'),

        // 0xE0–0xEF
        ('Ó', 0x4200fa28a28be000),
        ('ß', 0xc492512494410000),
        ('Ô', 0x8500fa28a28be000),
        ('Ò', 0x10200fa28a28be000),
        ('õ', 0x10a8401c8a289c000),
        ('Õ', 0x10a84fa28a28be000),
        ('µ', 0x228a28be820),
        ('þ', 0x20820b328a28bc820),
        ('Þ', 0x2083c8a28bc820),
        ('Ú', 0x42228a28a28be000),
        ('Û', 0x85008a28a28be000),
        ('Ù', 0x102228a28a28be000),
        ('ý', 0x108022894508210),
        ('Ý', 0x4aa2514208208000),
        ('¯', z(0x1f, 61)),
        ('´', z(0x21, 57)),

        // 0xF0–0xFF
        ('\u{ad}', z(0x7, 38)),
        ('±', 0x208f88200f80000),
        ('‗', z(0x1f01f, 7)),
        ('¾', 0x312722b428e5d2000),
        ('¶', 0x1eebae9a08289c000),
        ('§', 0xc49031248c092300),
        ('÷', z(0x1007c01, 27)),
        ('¸', z(0x41, 9)),
        ('°', z(0x7147, 50)),
        ('¨', z(0x5, 56)),
        ('·', z(0x1, 39)),
        ('¹', z(0x2182087, 38)),
        ('³', z(0x3023023, 39)),
        ('²', z(0x6042107, 38)),
        ('■', z(0x3fffffff, 24)),
        ('\u{a0}', 0),
    ];
    // Both graphical control slots use their existing extended CP850 masks.
    symbols[0x14] = symbols[0xf4]; // ¶
    symbols[0x15] = symbols[0xf5]; // §
    symbols
};

const fn mapped(character: char) -> (char, u128) {
    if let Some(pixels) = crate::symbol_pixels(character) {
        return (character, pixels);
    }
    let mut index = 0;
    while index < crate::UI_SYMBOLS.len() {
        let symbol = crate::UI_SYMBOLS[index];
        if symbol.0 == character {
            return symbol;
        }
        index += 1;
    }
    panic!("missing shared CP850 glyph mask")
}

/// Legacy 224-slot CP850 atlas (bytes 0x20–0xFF), derived at compile time.
/// The 64-bit compatibility masks omit the final two pixels of each cell.
pub const FPIXELS: [u64; 224] = {
    let mut pixels = [0; 224];
    let mut slot = 0;
    while slot < pixels.len() {
        pixels[slot] = (CP850_SYMBOLS[slot + 0x20].1 >> 2) as u64;
        slot += 1;
    }
    pixels
};

// Keep Unicode-to-byte lookup compact; this view has no handwritten identities.
const EXTENDED_CHARS: [char; 128] = {
    let mut characters = ['\0'; 128];
    let mut index = 0;
    while index < characters.len() {
        characters[index] = CP850_SYMBOLS[index + 0x80].0;
        index += 1;
    }
    characters
};

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

/// Legacy byte mask; control bytes below 0x20 retain the blank space mask.
pub fn font_pixels(character: u8) -> u64 {
    FPIXELS[(i32::from(character) - 32).clamp(0, FPIXELS.len() as i32 - 1) as usize]
}

/// Unicode identity of a legacy atlas slot: ASCII (including DEL), then CP850.
/// The full graphical display identities are available in `CP850_SYMBOLS`.
pub fn glyph_character(byte: u8) -> Option<char> {
    match byte {
        0x20..=0x7f => Some(byte as char),
        0x80..=0xff => Some(EXTENDED_CHARS[(byte - 0x80) as usize]),
        _ => None,
    }
}
