//! Basic Latin (U+0000–U+007F), copied from the existing font.
//! Full 6×11 masks, top-left at bit 65. Single-cell control/space/DEL masks are blank;
//! named control labels are prepared below as two- or three-cell composites.
//! No quad raster data yet.

use crate::z128 as z;

pub const BASIC_SYMBOLS: [(char, u128); 128] = [
    ('\u{00}', 0), // NUL
    ('\u{01}', 0), // SOH
    ('\u{02}', 0), // STX
    ('\u{03}', 0), // ETX
    ('\u{04}', 0), // EOT
    ('\u{05}', 0), // ENQ
    ('\u{06}', 0), // ACK
    ('\u{07}', 0), // BEL
    ('\u{08}', 0), // BS
    ('\u{09}', 0), // HT
    ('\u{0a}', 0), // LF
    ('\u{0b}', 0), // VT
    ('\u{0c}', 0), // FF
    ('\u{0d}', 0), // CR
    ('\u{0e}', 0), // SO
    ('\u{0f}', 0), // SI
    ('\u{10}', 0), // DLE
    ('\u{11}', 0), // DC1
    ('\u{12}', 0), // DC2
    ('\u{13}', 0), // DC3
    ('\u{14}', 0), // DC4
    ('\u{15}', 0), // NAK
    ('\u{16}', 0), // SYN
    ('\u{17}', 0), // ETB
    ('\u{18}', 0), // CAN
    ('\u{19}', 0), // EM
    ('\u{1a}', 0), // SUB
    ('\u{1b}', 0), // ESC
    ('\u{1c}', 0), // FS
    ('\u{1d}', 0), // GS
    ('\u{1e}', 0), // RS
    ('\u{1f}', 0), // US
    (' ', 0), // SP
    ('!', 0x8208208200208000),
    ('"', z(0x5145, 50)),
    ('#', 0x1453e51453e514000),
    ('$', 0x872aa1c2aa708000),
    ('%', 0x12a94108414aa4000),
    ('&', 0x85142158a289d000),
    ('\'', z(0x1041, 51)),
    ('(', 0x2108208208102000),
    (')', 0x10204104104210000),
    ('*', z(0x54e395, 43)),
    ('+', z(0x1047c41, 27)),
    (',', z(0x821, 9)),
    ('-', z(0x1f, 37)),
    ('.', z(0xc3, 14)),
    ('/', 0x2084108410820000),
    ('0', 0x1c8a28a28a289c000),
    ('1', 0x218a4a2082082000),
    ('2', 0x1c8a208421083e000),
    ('3', 0x1c8a208c0a289c000),
    ('4', 0x4314524f84104000),
    ('5', 0x3e820f020820bc000),
    ('6', 0x1c8a083c8a289c000),
    ('7', 0x3e084108210410000),
    ('8', 0x1c8a289c8a289c000),
    ('9', 0x1c8a289e08289c000),
    (':', z(0xc30000c3, 20)),
    (';', 0xc30000c304200),
    ('<', 0x84210204080000),
    ('=', z(0x1f01f, 31)),
    ('>', 0x810204210800000),
    ('?', 0x1c8a2108200208000),
    ('@', 0x1e86dd75d75ba0780),
    ('A', 0x8214514fa28a2000),
    ('B', 0x3c8a28bc8a28bc000),
    ('C', 0x1c8a082082089c000),
    ('D', 0x3c8a28a28a28bc000),
    ('E', 0x3e82083c82083e000),
    ('F', 0x3e82083c820820000),
    ('G', 0x1c8a082c8a289c000),
    ('H', 0x228a28be8a28a2000),
    ('I', 0x3e20820820823e000),
    ('J', 0x3e08208208289c000),
    ('K', 0x22924a30a24922000),
    ('L', 0x2082082082083e000),
    ('M', 0x22db6aaa8a28a2000),
    ('N', 0x22cb2aaaaa69a2000),
    ('O', 0x3e8a28a28a28be000),
    ('P', 0x3c8a28bc820820000),
    ('Q', 0x1c8a28a28a291a000),
    ('R', 0x3c8a28bca248a2000),
    ('S', 0x1c8a081c08289c000),
    ('T', 0x3e208208208208000),
    ('U', 0x228a28a28a289c000),
    ('V', 0x228a2894514208000),
    ('W', 0x228a28a2aaa514000),
    ('X', 0x228945085148a2000),
    ('Y', 0x22894508208208000),
    ('Z', 0x3e08410841083e000),
    ('[', 0xe20820820820e000),
    ('\\', 0x20810408104082000),
    (']', 0x38208208208238000),
    ('^', z(0x4291, 49)),
    ('_', z(0x1f, 13)),
    ('`', z(0x81, 56)),
    ('a', z(0x7027a27, 14)),
    ('b', 0x2082083c8a28bc000),
    ('c', z(0xf41040f, 13)),
    ('d', 0x208209e8a289e000),
    ('e', z(0x722fa07, 14)),
    ('f', 0x620821c208208000),
    ('g', z(0x7a28a27827, 2)),
    ('h', 0x2082083c8a28a2000),
    ('i', 0x8008208208000),
    ('j', 0x8008208208a10),
    ('k', 0x10410412518512000),
    ('l', 0x1041041041040c000),
    ('m', z(0x1e555555, 13)),
    ('n', z(0x1e451451, 13)),
    ('o', z(0x7228a27, 14)),
    ('p', z(0x1c49249c41, 4)),
    ('q', z(0x7249247041, 2)),
    ('r', z(0x1c49041, 16)),
    ('s', z(0x3907827, 14)),
    ('t', 0x820821c208206000),
    ('u', z(0x1145144f, 13)),
    ('v', z(0x45128a1, 15)),
    ('w', z(0x8aa7145, 14)),
    ('x', z(0x11284291, 13)),
    ('y', z(0x2289450821, 4)),
    ('z', z(0x1f08421f, 13)),
    ('{', 0x2104108104102000),
    ('|', 0x8208208208208000),
    ('}', 0x20410408410420000),
    ('~', z(0x42a1, 32)),
    ('\u{7f}', 0), // DEL
];

const CONTROL_NAMES: [&str; 33] = [
    "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL",
    "BS", "HT", "LF", "VT", "FF", "CR", "SO", "SI",
    "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB",
    "CAN", "EM", "SUB", "ESC", "FS", "GS", "RS", "US", "SP",
];

struct ControlData {
    characters: [[char; 3]; 33],
    pixels: [[u128; 3]; 33],
}

// Already in the program image: no allocation, lazy initialization or warming
// on first use. Both representations come from the existing Basic Latin data.
static CONTROL_DATA: ControlData = prepare_controls();

const fn prepare_controls() -> ControlData {
    let mut data = ControlData { characters: [['\0'; 3]; 33], pixels: [[0; 3]; 33] };
    let mut i = 0;
    while i < CONTROL_NAMES.len() {
        let name = CONTROL_NAMES[i].as_bytes();
        let mut cell = 0;
        while cell < name.len() {
            data.characters[i][cell] = name[cell] as char;
            data.pixels[i][cell] = BASIC_SYMBOLS[name[cell] as usize].1;
            cell += 1;
        }
        i += 1;
    }
    data
}

/// Ready-to-use control labels, in the same multi-cell mask format as `SPECIAL`.
/// Entries cover U+0000–U+0020, including the two-cell `SP` representation.
pub static CONTROL_SYMBOLS: [(char, &'static [u128]); 33] = {
    let mut symbols: [(char, &'static [u128]); 33] = [('\0', &[]); 33];
    let mut i = 0;
    while i < symbols.len() {
        symbols[i] = ((i as u8) as char, CONTROL_DATA.pixels[i].split_at(CONTROL_NAMES[i].len()).0);
        i += 1;
    }
    symbols
};

/// Explicit control-label mask lookup, including the two-cell `SP` label.
pub fn control_label_cells(character: char) -> Option<&'static [u128]> {
    CONTROL_SYMBOLS.get(character as usize).map(|&(_, cells)| cells)
}

pub(crate) fn control_characters(character: char) -> Option<&'static [char]> {
    let index = character as usize;
    if index > 32 { return None; }
    Some(CONTROL_DATA.characters[index].split_at(CONTROL_NAMES[index].len()).0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_composites_are_ready_and_use_existing_basic_masks() {
        for (index, &(ch, cells)) in CONTROL_SYMBOLS.iter().enumerate() {
            assert_eq!(ch as usize, index);
            let name = CONTROL_NAMES[index];
            assert_eq!(cells.len(), name.len());
            assert!((2..=3).contains(&cells.len()));
            for (&mask, letter) in cells.iter().zip(name.bytes()) {
                assert_ne!(mask, 0);
                assert_eq!(mask, crate::glyph_cell_pixels(letter as char));
            }
        }
        assert_eq!(control_label_cells(' ').unwrap().len(), 2);
        assert_eq!(crate::special_cells(' '), Some(&['S', 'P'][..]));
        assert_eq!(crate::measure_text(" "), (12, 11));
        assert_eq!(crate::glyph_cell_pixels(' '), 0);
        assert!(control_label_cells('A').is_none());
    }

    #[test]
    #[cfg(feature = "stamp")]
    fn control_text_uses_the_existing_multicell_stamping_path() {
        let mut actual = [0u8; 50 * 13];
        let mut expected = actual;
        crate::stamp_text_with_stride(&mut actual, 48, 12, 50, -2, 1, "\0\x08 ", 1).unwrap();
        crate::stamp_text_with_stride(&mut expected, 48, 12, 50, -2, 1, "NULBSSP", 1).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(crate::measure_text("\0\x08\t"), (42, 11));
        assert_eq!(crate::measure_text("⌨🗺"), (30, 11));
        assert_eq!(crate::measure_bytes(b"\0\x08\t"), (18, 11));
    }
}
