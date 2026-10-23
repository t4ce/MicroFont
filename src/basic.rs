//! Basic Latin identities and multi-cell control labels.
//! Bitmap values are loaded from the selected glyph bundle.

pub const BASIC_CHARACTERS: [char; 128] = {
    let mut characters = ['\0'; 128];
    let mut index = 0;
    while index < characters.len() {
        characters[index] = index as u8 as char;
        index += 1;
    }
    characters
};

const CONTROL_NAMES: [&str; 34] = [
    "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL",
    "BS", "HT", "LF", "VT", "FF", "CR", "SO", "SI",
    "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB",
    "CAN", "EM", "SUB", "ESC", "FS", "GS", "RS", "US", "SP", "DEL",
];

const CONTROL_DATA: [[char; 3]; 34] = {
    let mut characters = [['\0'; 3]; 34];
    let mut index = 0;
    while index < CONTROL_NAMES.len() {
        let name = CONTROL_NAMES[index].as_bytes();
        let mut cell = 0;
        while cell < name.len() {
            characters[index][cell] = name[cell] as char;
            cell += 1;
        }
        index += 1;
    }
    characters
};

/// Control-label character slices, including the two-cell `SP` label.
pub const CONTROL_CHARACTERS: [(char, &'static [char]); 34] = {
    let mut characters: [(char, &'static [char]); 34] = [('\0', &[]); 34];
    let mut index = 0;
    while index < characters.len() {
        characters[index] = (
            if index == 33 { '\u{7f}' } else { index as u8 as char },
            CONTROL_DATA[index].split_at(CONTROL_NAMES[index].len()).0,
        );
        index += 1;
    }
    characters
};

/// An owned control label, decoded without allocation from the installed bundle.
#[cfg(feature = "hex")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlLabelCells {
    pixels: [u128; 3],
    len: u8,
}

#[cfg(feature = "hex")]
impl ControlLabelCells {
    pub fn as_slice(&self) -> &[u128] { &self.pixels[..self.len()] }
    pub fn iter(&self) -> core::slice::Iter<'_, u128> { self.as_slice().iter() }
    pub const fn len(&self) -> usize { self.len as usize }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
}

#[cfg(feature = "hex")]
impl core::ops::Deref for ControlLabelCells {
    type Target = [u128];
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

/// Decode a control-label mask, including `SP`, without an expanded mask table.
#[cfg(feature = "hex")]
pub fn control_label_cells(character: char) -> Option<ControlLabelCells> {
    let characters = control_characters(character)?;
    let mut cells = ControlLabelCells { pixels: [0; 3], len: characters.len() as u8 };
    for (pixel, &letter) in cells.pixels.iter_mut().zip(characters) {
        *pixel = crate::glyph_cell_pixels(letter);
    }
    Some(cells)
}

fn control_index(character: char) -> Option<usize> {
    match character {
        '\u{00}'..=' ' => Some(character as usize),
        '\u{7f}' => Some(33),
        _ => None,
    }
}

pub(crate) fn control_characters(character: char) -> Option<&'static [char]> {
    Some(CONTROL_CHARACTERS[control_index(character)?].1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_keep_character_expansion_without_bitmap_storage() {
        for (index, &(ch, letters)) in CONTROL_CHARACTERS.iter().enumerate() {
            assert_eq!(ch as usize, if index == 33 { 0x7f } else { index });
            assert_eq!(letters.iter().copied().collect::<std::string::String>(), CONTROL_NAMES[index]);
        }
        assert_eq!(crate::special_cells(' '), Some(&['S', 'P'][..]));
        assert_eq!(crate::measure_text(" "), (12, 11));
        assert_eq!(crate::special_cells('\u{7f}'), Some(&['D', 'E', 'L'][..]));
        assert_eq!(crate::measure_text("\u{7f}"), (18, 11));
    }

    #[test]
    #[cfg(feature = "hex")]
    fn control_labels_read_the_installed_basic_masks() {
        for &(ch, letters) in &CONTROL_CHARACTERS {
            let cells = control_label_cells(ch).unwrap();
            assert_eq!(cells.len(), letters.len());
            for (&mask, &letter) in cells.iter().zip(letters) {
                assert_ne!(mask, 0);
                assert_eq!(mask, crate::glyph_cell_pixels(letter));
            }
        }
        assert_eq!(control_label_cells(' ').unwrap().len(), 2);
        assert_eq!(control_label_cells('\u{7f}').unwrap().len(), 3);
        assert!(control_label_cells('A').is_none());
        assert_eq!(crate::glyph_cell_pixels(' '), 0);
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
        actual.fill(0);
        expected.fill(0);
        crate::stamp_text(&mut actual, 18, 11, 0, 0, "\u{7f}", 1).unwrap();
        crate::stamp_text(&mut expected, 18, 11, 0, 0, "DEL", 1).unwrap();
        assert_eq!(actual, expected);
    }
}
