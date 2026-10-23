//! Unicode block identities, independent of representation.
//! Masks and quad geometry are stored in the selected glyph bundle.

pub const BLOCK_CHARACTERS: [char; 32] = {
    let mut characters = ['\0'; 32];
    let mut index = 0;
    while index < characters.len() {
        characters[index] = match char::from_u32(0x2580 + index as u32) {
            Some(character) => character,
            None => panic!("invalid Unicode glyph identity"),
        };
        index += 1;
    }
    characters
};
