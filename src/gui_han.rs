//! One-cell GUI subset. Font data is OFL-1.1; decoder code is MIT licensed.
//!
//! Records preserve exact source pixels. Most are 35-bit 5x7 literals padded
//! into the 6x11 cell; punctuation can use a full 66-bit literal. Shared shapes
//! refer directly to a literal and optionally toggle one or two pixels. There
//! are no reference chains, runtime allocations, or expanded glyph tables.

#[path = "gui_han_data.rs"]
mod data;

pub const GUI_HAN_COUNT: usize = data::CODES.len();
pub const GUI_HAN_TABLE_BYTES: usize =
    data::CODES.len() * 2 + data::CHECKPOINTS.len() * 2 + data::DATA.len();

/// Iterate the existing subset without embedding a second codepoint table.
pub fn characters() -> impl ExactSizeIterator<Item = char> {
    data::CODES.iter().map(|&cp| char::from_u32(cp as u32).unwrap())
}

fn read_u16(at: usize) -> usize {
    u16::from_le_bytes([data::DATA[at], data::DATA[at + 1]]) as usize
}

fn record_len(tag: u8) -> usize {
    match tag {
        0..=7 => 5,
        8..=11 => 9,
        32 | 128..=191 => 3,
        192..=255 => 4,
        _ => unreachable!("invalid generated glyph record"),
    }
}

fn literal(at: usize) -> u128 {
    let tag = data::DATA[at];
    if tag < 8 {
        let packed = ((tag as u64) << 32)
            | u32::from_le_bytes([
                data::DATA[at + 1],
                data::DATA[at + 2],
                data::DATA[at + 3],
                data::DATA[at + 4],
            ]) as u64;
        let mut mask = 0u128;
        for row in 0..7 {
            mask |= (((packed >> (row * 5)) & 31) as u128) << (12 + row * 6 + 1);
        }
        mask
    } else {
        debug_assert!((8..12).contains(&tag), "patch must reference a literal");
        let mut mask = ((tag & 3) as u128) << 64;
        for byte in 0..8 {
            mask |= (data::DATA[at + 1 + byte] as u128) << (byte * 8);
        }
        mask
    }
}

fn decode(at: usize) -> u128 {
    let tag = data::DATA[at];
    match tag {
        0..=11 => literal(at),
        32 => literal(read_u16(at + 1)),
        128..=191 => literal(read_u16(at + 1)) ^ (1u128 << (tag & 63)),
        192..=255 => {
            literal(read_u16(at + 2)) ^ (1u128 << (tag & 63)) ^ (1u128 << data::DATA[at + 1])
        }
        _ => unreachable!("invalid generated glyph record"),
    }
}

pub fn glyph_cell_pixels(character: char) -> Option<u128> {
    let cp = u16::try_from(character as u32).ok()?;
    // Keep ordinary ASCII/CP850 stamping out of the subset's binary search.
    if cp < data::CODES[0] || cp > data::CODES[GUI_HAN_COUNT - 1] {
        return None;
    }
    let index = data::CODES.binary_search(&cp).ok()?;
    // A checkpoint every 16 glyphs bounds the scan to at most 15 tiny records.
    let mut at = data::CHECKPOINTS[index / 16] as usize;
    for _ in 0..index % 16 {
        at += record_len(data::DATA[at]);
    }
    Some(decode(at))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_records_are_sorted_bounded_and_reference_only_earlier_literals() {
        assert_eq!(GUI_HAN_COUNT, 512);
        assert!(GUI_HAN_TABLE_BYTES < 4096);
        assert!(data::CODES.windows(2).all(|pair| pair[0] < pair[1]));
        let mut at = 0;
        for (index, &cp) in data::CODES.iter().enumerate() {
            if index % 16 == 0 {
                assert_eq!(data::CHECKPOINTS[index / 16] as usize, at);
            }
            let tag = data::DATA[at];
            if tag >= 32 {
                let base = read_u16(at + if tag < 192 { 1 } else { 2 });
                assert!(base < at);
                assert!(data::DATA[base] < 12);
            }
            let mask = decode(at);
            assert_eq!(mask >> 66, 0);
            assert_eq!(glyph_cell_pixels(char::from_u32(cp as u32).unwrap()), Some(mask));
            at += record_len(tag);
        }
        assert_eq!(at, data::DATA.len());
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
