//! One-cell GUI subset identities. OFL-1.1 bitmap data lives in the glyph bundle.

#[path = "gui_han_data.rs"]
mod data;

pub const GUI_HAN_COUNT: usize = data::CODES.len();
/// Bitmap payload bytes for this subset, excluding the shared bundle index.
pub const GUI_HAN_TABLE_BYTES: usize = GUI_HAN_COUNT * 16;

/// Iterate the existing subset without expanding its bitmap records.
pub fn characters() -> impl ExactSizeIterator<Item = char> {
    data::CODES.iter().map(|&cp| char::from_u32(cp as u32).unwrap())
}

pub fn glyph_cell_pixels(character: char) -> Option<u128> {
    let cp = u16::try_from(character as u32).ok()?;
    data::CODES.binary_search(&cp).ok()?;
    crate::bundle::bitmap(character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_sorted_and_each_resolves_to_a_complete_mask() {
        assert_eq!(GUI_HAN_COUNT, 512);
        assert!(data::CODES.windows(2).all(|pair| pair[0] < pair[1]));
        for character in characters() {
            let mask = glyph_cell_pixels(character).unwrap();
            assert_eq!(mask >> 66, 0);
        }
    }

    #[test]
    fn every_gui_word_is_covered_and_unselected_characters_stay_unsupported() {
        for line in include_str!("../tools/gui-zh-hans.txt").lines() {
            if line.starts_with('#') {
                continue;
            }
            for ch in line
                .chars()
                .filter(|ch| !ch.is_ascii() && !ch.is_whitespace())
            {
                assert!(glyph_cell_pixels(ch).is_some(), "missing GUI glyph {ch}");
            }
        }
        assert_eq!(glyph_cell_pixels('釕'), None);
        assert_eq!(glyph_cell_pixels('🙂'), None);
    }

    #[test]
    #[cfg(feature = "stamp")]
    fn chinese_text_uses_one_cell_and_preserves_clipping_stride_and_padding() {
        let text = "文件夹";
        assert_eq!(crate::measure_text(text), (18, 11));
        let mut pixels = [0u8; 22 * 13];
        crate::stamp_text_with_stride(&mut pixels, 20, 12, 22, -2, 1, text, 1).unwrap();
        let masks: std::vec::Vec<_> = text
            .chars()
            .map(|ch| glyph_cell_pixels(ch).unwrap())
            .collect();
        for y in 0..13 {
            for x in 0..22 {
                let sx = x as i32 + 2;
                let sy = y as i32 - 1;
                let expected = x < 20
                    && y < 12
                    && (0..18).contains(&sx)
                    && (0..11).contains(&sy)
                    && (masks[sx as usize / 6] >> (65 - (sy as usize * 6 + sx as usize % 6))) & 1
                        != 0;
                assert_eq!(pixels[y * 22 + x] != 0, expected);
            }
        }
        assert_eq!(crate::glyph_cell_pixels('釕'), crate::glyph_cell_pixels('?'));
    }
}
