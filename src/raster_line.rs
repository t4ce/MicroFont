//! Two-endpoint line experiment. Bitmap glyphs remain unchanged.
//! A source cell is 6×11, top-left at bit 65; endpoints use pixel centres.
pub const fn seed_pixels(character: char) -> Option<u128> {
    match character {
        '|' => Some(0x08000000000008000),
        '/' => Some(0x02000000000020000),
        _ => None,
    }
}

/// Exactly one segment, with no reach, adjacency inference or tessellation.
pub fn endpoints(character: char) -> Option<[[f32; 2]; 2]> {
    let mask = seed_pixels(character)?;
    let mut points = [[0.0; 2]; 2];
    let mut count = 0;
    for cell in 0..66 {
        if mask & (1u128 << (65 - cell)) != 0 {
            if count == 2 { return None; }
            points[count] = [(cell % 6) as f32 + 0.5, (cell / 6) as f32 + 0.5];
            count += 1;
        }
    }
    (count == 2).then_some(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoints_are_extremes_of_original_bitmap() {
        for ch in ['|', '/'] {
            let original = crate::BASIC_SYMBOLS[ch as usize].1;
            let expected = (1u128 << original.trailing_zeros())
                | (1u128 << (127 - original.leading_zeros()));
            assert_eq!(seed_pixels(ch), Some(expected));
            assert_eq!(expected.count_ones(), 2);
            let points = endpoints(ch).unwrap();
            assert!(points[0][1] < points[1][1]);
            if ch == '|' { assert_eq!(points[0][0], points[1][0]); }
            else { assert!(points[0][0] > points[1][0]); }
        }
        assert_eq!(endpoints('A'), None);
    }
}
