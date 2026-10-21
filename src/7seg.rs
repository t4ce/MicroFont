//! Segmented digits U+1FBF0–U+1FBF9, generated from seven shared strokes.
//! Segment order: top, upper-right, lower-right, bottom, lower-left, upper-left, middle.
const FIRST: u32 = 0x1FBF0;
const SEGMENTS: [[[u8; 2]; 2]; 7] = [
    [[1, 1], [3, 1]], [[4, 2], [4, 4]], [[4, 6], [4, 8]],
    [[1, 9], [3, 9]], [[0, 6], [0, 8]], [[0, 2], [0, 4]],
    [[1, 5], [3, 5]],
];
const DIGITS: [u8; 10] = [0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F];

const fn digit_mask(digit: usize) -> u128 {
    let mut pixels = 0;
    let mut segment = 0;
    while segment < 7 {
        if DIGITS[digit] & (1 << segment) != 0 {
            let [[left, top], [right, bottom]] = SEGMENTS[segment];
            let mut y = top;
            while y <= bottom {
                let mut x = left;
                while x <= right {
                    pixels |= 1u128 << (65 - (y as usize * 6 + x as usize));
                    x += 1;
                }
                y += 1;
            }
        }
        segment += 1;
    }
    pixels
}

pub const SYMBOLS: [(char, u128); 10] = {
    let mut symbols = [('\0', 0); 10];
    let mut digit = 0;
    while digit < 10 {
        let ch = match char::from_u32(FIRST + digit as u32) { Some(ch) => ch, None => panic!("segmented digit") };
        symbols[digit] = (ch, digit_mask(digit));
        digit += 1;
    }
    symbols
};

pub const fn glyph_mask(ch: char) -> Option<u128> {
    let code = ch as u32;
    if code < FIRST || code >= FIRST + 10 { None }
    else { Some(SYMBOLS[(code - FIRST) as usize].1) }
}

#[cfg(feature = "raster")]
mod lines {
    use crate::raster_line::{Boundary::End, Point, Strip};
    use super::{DIGITS, FIRST, SEGMENTS};
    static POINTS: [[Point; 2]; 7] = {
        let mut points = [[Point::new(0, 0); 2]; 7];
        let mut i = 0;
        while i < 7 {
            points[i] = [Point::new(SEGMENTS[i][0][0], SEGMENTS[i][0][1]),
                         Point::new(SEGMENTS[i][1][0], SEGMENTS[i][1][1])];
            i += 1;
        }
        points
    };
    const COUNT: usize = {
        let mut count = 0;
        let mut i = 0;
        while i < 10 { count += DIGITS[i].count_ones() as usize; i += 1; }
        count
    };
    struct LineData { strips: [Strip; COUNT], offsets: [usize; 11] }
    const DATA: LineData = {
        let mut data = LineData { strips: [Strip::new(&POINTS[0], End, End); COUNT], offsets: [0; 11] };
        let mut digit = 0;
        let mut cursor = 0;
        while digit < 10 {
            let mut segment = 0;
            while segment < 7 {
                if DIGITS[digit] & (1 << segment) != 0 {
                    data.strips[cursor] = Strip::new(&POINTS[segment], End, End);
                    cursor += 1;
                }
                segment += 1;
            }
            data.offsets[digit + 1] = cursor;
            digit += 1;
        }
        data
    };
    pub const fn glyph_strips(ch: char) -> Option<&'static [Strip]> {
        let code = ch as u32;
        if code < FIRST || code >= FIRST + 10 { return None; }
        let digit = (code - FIRST) as usize;
        let start = DATA.offsets[digit];
        let count = DATA.offsets[digit + 1] - start;
        Some(DATA.strips.split_at(start).1.split_at(count).0)
    }
}
#[cfg(feature = "raster")]
pub(crate) use lines::glyph_strips;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_masks_match_existing_digits_exactly() {
        let expected = [0x007228a20228a2700, 0x00002082002082000,
            0x00702082720820700, 0x00702082702082700, 0x000228a2702082000,
            0x00720820702082700, 0x007208207228a2700, 0x00702082002082000,
            0x007228a27228a2700, 0x007228a2702082700];
        for (digit, &mask) in expected.iter().enumerate() {
            let ch = SYMBOLS[digit].0;
            assert_eq!(glyph_mask(ch), Some(mask));
            assert_eq!(crate::glyph_cell_pixels(ch), mask);
        }
    }
    #[cfg(feature = "raster")]
    #[test]
    fn line_digits_share_segments_and_preserve_native_end_tags() {
        use crate::raster_line::{glyph_strips, Boundary::End};
        for (digit, &(ch, _)) in SYMBOLS.iter().enumerate() {
            let strips = glyph_strips(ch).unwrap();
            assert_eq!(strips.len(), DIGITS[digit].count_ones() as usize);
            for strip in strips {
                assert_eq!(strip.points.len(), 2);
                assert_eq!((strip.start, strip.end), (End, End));
            }
        }
        assert_eq!(glyph_strips('🯱').unwrap()[0].points.as_ptr(),
                   glyph_strips('🯸').unwrap()[1].points.as_ptr());
        assert!(glyph_mask('0').is_none());
    }
}
