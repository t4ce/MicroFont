//! Unicode box draw identities, independent of representation.
//! Masks and quad geometry are stored in the selected glyph bundle.

pub const BOX_DRAW_CHARACTERS: [char; 128] = {
    let mut characters = ['\0'; 128];
    let mut index = 0;
    while index < characters.len() {
        characters[index] = match char::from_u32(0x2500 + index as u32) {
            Some(character) => character,
            None => panic!("invalid Unicode glyph identity"),
        };
        index += 1;
    }
    characters
};
