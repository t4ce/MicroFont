//! Segmented digits U+1FBF0–U+1FBF9. Masks and shared segment records are stored
//! in the single MicroFont binary bundle, alongside all other font geometry.

pub const CHARACTERS: [char; 10] = ['🯰', '🯱', '🯲', '🯳', '🯴', '🯵', '🯶', '🯷', '🯸', '🯹'];

#[cfg(feature = "hex")]
pub fn glyph_mask(character: char) -> Option<u128> {
    if !('\u{1fbf0}'..='\u{1fbf9}').contains(&character) { return None; }
    crate::bundle::bitmap(character)
}

#[cfg(all(test, feature = "raster"))]
mod tests {
    #[test]
    fn line_digits_share_segments_and_preserve_native_end_tags() {
        use crate::raster_line::{glyph_strips, Boundary::End};
        let counts = [6, 2, 5, 5, 4, 5, 6, 3, 7, 6];
        for (digit, &ch) in super::CHARACTERS.iter().enumerate() {
            let strips = glyph_strips(ch).unwrap();
            assert_eq!(strips.len(), counts[digit]);
            for strip in strips {
                assert_eq!(strip.points.len(), 2);
                assert_eq!((strip.start, strip.end), (End, End));
            }
        }
        assert_eq!(glyph_strips('🯱').unwrap().get(0).unwrap().points.as_ptr(),
            glyph_strips('🯸').unwrap().get(1).unwrap().points.as_ptr());
    }
}
