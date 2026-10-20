//! Compact independent strokes in a 6×11 cell. Bitmap consumers are unchanged.
//! Coordinates are lattice sites; the renderer adds 0.5 to reach pixel centres.

/// Seven coordinate bits: xxx in bits 0..2, yyyy in bits 3..6. Bit 7 is reserved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct Point(u8);
impl Point {
    pub const fn new(x: u8, y: u8) -> Self {
        assert!(x < 6 && y < 11);
        Self(x | (y << 3))
    }
    pub const fn x(self) -> u8 { self.0 & 7 }
    pub const fn y(self) -> u8 { self.0 >> 3 }
    pub const fn packed(self) -> u8 { self.0 }
    pub const fn on_cell_edge(self) -> bool {
        self.x() == 0 || self.x() == 5 || self.y() == 0 || self.y() == 10
    }
}

/// Semantic boundary tag, independent of whether a coordinate is on a cell edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Boundary { Junction = 0, End = 1 }

/// One independent strip. N visible points form N−1 segments, with two extra
/// adjacency vertices generated at upload. Multiple strips require separate draws.
#[derive(Clone, Copy, Debug)]
pub struct Strip {
    /// Canonical point bytes, shared by transformed glyph views. Use point()
    /// or iter_points() for coordinates in the requested glyph's orientation.
    pub points: &'static [Point],
    pub start: Boundary,
    pub end: Boundary,
    flip_x_sum: u8,
    flip_y_sum: u8,
}
impl Strip {
    pub const fn new(points: &'static [Point], start: Boundary, end: Boundary) -> Self {
        assert!(points.len() >= 2);
        Self { points, start, end, flip_x_sum: u8::MAX, flip_y_sum: u8::MAX }
    }
    pub const fn point(self, index: usize) -> Option<Point> {
        if index >= self.points.len() { return None; }
        let p = self.points[index];
        Some(Point::new(
            if self.flip_x_sum == u8::MAX { p.x() } else { self.flip_x_sum - p.x() },
            if self.flip_y_sum == u8::MAX { p.y() } else { self.flip_y_sum - p.y() }))
    }
    pub fn iter_points(self) -> impl ExactSizeIterator<Item = Point> {
        (0..self.points.len()).map(move |i| self.point(i).unwrap())
    }
    pub const fn segments(self) -> usize { self.points.len() - 1 }
    /// Source coordinates plus z: real vertices use depth 0.5; adjacency-only
    /// vertices duplicate the boundary position and carry Junction=0 / End=1.
    /// This is a demo transport contract, not a generic position.z convention.
    pub fn adjacency_vertex(self, index: usize) -> Option<[f32; 3]> {
        let (point, z) = if index == 0 {
            (self.point(0)?, self.start as u8 as f32)
        } else if index <= self.points.len() {
            (self.point(index - 1)?, 0.5)
        } else if index == self.points.len() + 1 {
            (self.point(self.points.len() - 1)?, self.end as u8 as f32)
        } else { return None; };
        Some([point.x() as f32, point.y() as f32, z])
    }
}

/// Aliases only change coordinate interpretation; point bytes and tags are shared.
#[derive(Clone, Copy)]
pub enum GlyphTransform { MirrorX, MirrorY, Rotate180 }

pub const fn transformed_glyph<const N: usize>(source: &[Strip], transform: GlyphTransform) -> [Strip; N] {
    assert!(N > 0 && source.len() == N);
    let mut min_x = 5;
    let mut max_x = 0;
    let mut min_y = 10;
    let mut max_y = 0;
    let mut i = 0;
    while i < N {
        // Generated aliases always reference canonical bins, never other aliases.
        assert!(source[i].flip_x_sum == u8::MAX && source[i].flip_y_sum == u8::MAX);
        let mut j = 0;
        while j < source[i].points.len() {
            let p = source[i].points[j];
            if p.x() < min_x { min_x = p.x(); }
            if p.x() > max_x { max_x = p.x(); }
            if p.y() < min_y { min_y = p.y(); }
            if p.y() > max_y { max_y = p.y(); }
            j += 1;
        }
        i += 1;
    }
    let mut result = [source[0]; N];
    i = 0;
    while i < N {
        result[i] = source[i];
        result[i].flip_x_sum = match transform {
            GlyphTransform::MirrorY => u8::MAX,
            GlyphTransform::MirrorX | GlyphTransform::Rotate180 => min_x + max_x,
        };
        result[i].flip_y_sum = match transform {
            GlyphTransform::MirrorX => u8::MAX,
            GlyphTransform::MirrorY | GlyphTransform::Rotate180 => min_y + max_y,
        };
        i += 1;
    }
    result
}

use Boundary::{End, Junction};
// One glyph override may contain any number of independent strips.
const BAR: &[Strip] = &[Strip::new(&[Point::new(2, 0), Point::new(2, 8)], End, End)];
const SLASH: &[Strip] = &[Strip::new(&[Point::new(4, 0), Point::new(0, 8)], End, End)];
/// Decode the fixed v1 strip format without allocating or copying point data.
/// Each strip has a header: bits 0..5 = point count minus two, bit 6 =
/// start End, bit 7 = end End. It is followed by that many packed points.
/// The byte slice bounds the glyph; no terminator, alignment or native pointers
/// are stored in the file. Invalid data is rejected during const evaluation.
pub const fn decode_glyph<const STRIPS: usize>(bytes: &'static [u8]) -> [Strip; STRIPS] {
    const EMPTY: Strip = Strip::new(&[Point::new(0, 0), Point::new(0, 0)], Junction, Junction);
    let mut result = [EMPTY; STRIPS];
    let mut cursor = 0;
    let mut strip = 0;
    while strip < STRIPS {
        assert!(cursor < bytes.len(), "missing strip header");
        let header = bytes[cursor];
        let count = (header & 63) as usize + 2;
        cursor += 1;
        assert!(count <= bytes.len() - cursor, "truncated strip points");
        let mut i = 0;
        while i < count {
            let packed = bytes[cursor + i];
            assert!(packed & 128 == 0 && packed & 7 < 6 && packed >> 3 < 11,
                "invalid packed point");
            i += 1;
        }
        // SAFETY: Point is repr(transparent) over u8, with identical alignment.
        // The entire range is in this static slice and validated above. Header
        // bytes are excluded; bit 7 remains reserved in every actual point.
        let points = unsafe {
            core::slice::from_raw_parts(bytes.as_ptr().add(cursor).cast::<Point>(), count)
        };
        result[strip] = Strip::new(points,
            if header & 64 != 0 { End } else { Junction },
            if header & 128 != 0 { End } else { Junction });
        cursor += count;
        strip += 1;
    }
    assert!(cursor == bytes.len(), "unexpected trailing glyph bytes");
    result
}

/// Count records from their headers, without adding a count byte to the file.
pub const fn glyph_strip_count(bytes: &[u8]) -> usize {
    let mut cursor = 0;
    let mut count = 0;
    while cursor < bytes.len() {
        let points = (bytes[cursor] & 63) as usize + 2;
        cursor += 1;
        assert!(points <= bytes.len() - cursor, "truncated strip points");
        cursor += points;
        count += 1;
    }
    assert!(count > 0, "empty glyph");
    count
}

// Generated views reference embedded binaries directly, without copying points.
include!("generated_line_glyphs.rs");
pub const fn glyph_strips(character: char) -> Option<&'static [Strip]> {
    match character { '|' => Some(BAR), '/' => Some(SLASH), _ => binary_glyph_strips(character) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strip_count_follows_replaced_binary_layout() {
        assert_eq!(glyph_strip_count(&[0xc0,0,64]), 1);
        assert_eq!(glyph_strip_count(&[0xc0,0,64,0x80,32,68,0x00,0,32]), 3);
        assert!(std::panic::catch_unwind(|| glyph_strip_count(&[0xc1,0,64])).is_err());
        assert!(std::panic::catch_unwind(|| glyph_strip_count(&[])).is_err());
    }
    #[test]
    fn binary_rejects_bad_headers_lengths_and_coordinates() {
        for bytes in [&[][..], &[0xc0, 0][..], &[0xc0, 0, 0, 0][..],
            &[0xc0, 6, 0][..], &[0xc0, 88, 0][..], &[0xc0, 128, 0][..]] {
            assert!(std::panic::catch_unwind(|| decode_glyph::<1>(bytes)).is_err());
        }
        assert!(std::panic::catch_unwind(|| decode_glyph::<1>(&[0xff])).is_err());
    }
    #[test]
    fn overrides_preserve_original_bitmap_extremes() {
        for ch in ['|', '/'] {
            let original = crate::BASIC_SYMBOLS[ch as usize].1;
            let expected = (1u128 << original.trailing_zeros())
                | (1u128 << (127 - original.leading_zeros()));
            let strips = glyph_strips(ch).unwrap();
            assert_eq!(strips.len(), 1);
            assert_eq!(strips[0].segments(), 1);
            let mut mask = 0u128;
            for p in strips[0].points {
                mask |= 1 << (65 - (p.y() as usize * 6 + p.x() as usize));
            }
            assert_eq!(mask, expected);
            assert_eq!(strips[0].adjacency_vertex(0).unwrap()[2], 1.0);
            assert_eq!(strips[0].adjacency_vertex(3).unwrap()[2], 1.0);
        }
    }
    #[test]
    fn t_has_independent_strokes_and_three_ends_one_junction() {
        const T: &[Strip] = &[
            Strip::new(&[Point::new(0, 0), Point::new(4, 0)], End, End),
            Strip::new(&[Point::new(2, 0), Point::new(2, 8)], Boundary::Junction, End),
        ];
        assert_eq!(T.iter().map(|s| s.points.len()).sum::<usize>(), 4);
        let tags = [T[0].start, T[0].end, T[1].start, T[1].end];
        assert_eq!(tags.iter().filter(|&&tag| tag == End).count(), 3);
        assert_eq!(T[1].adjacency_vertex(0), Some([2.0, 0.0, 0.0]));
        assert_eq!(T[1].adjacency_vertex(1), Some([2.0, 0.0, 0.5]));
        assert_eq!(T[1].adjacency_vertex(3), Some([2.0, 8.0, 1.0]));
        assert!(T[1].adjacency_vertex(4).is_none());
    }
    #[test]
    fn longer_strip_keeps_internal_vertices_real() {
        const STRIP: Strip = Strip::new(&[
            Point::new(0, 0), Point::new(2, 4), Point::new(5, 10),
        ], End, Boundary::Junction);
        assert_eq!(STRIP.segments(), 2);
        assert_eq!(STRIP.adjacency_vertex(2), Some([2.0, 4.0, 0.5]));
        assert_eq!(STRIP.adjacency_vertex(4), Some([5.0, 10.0, 0.0]));
    }
    #[test]
    fn packed_points_are_unique_and_edges_are_geometric() {
        let mut seen = [false; 128];
        for y in 0..11 { for x in 0..6 {
            let p = Point::new(x, y);
            assert_eq!((p.x(), p.y()), (x, y));
            assert!(!seen[p.packed() as usize]);
            seen[p.packed() as usize] = true;
            assert_eq!(p.on_cell_edge(), x == 0 || x == 5 || y == 0 || y == 10);
        }}
        assert_eq!(core::mem::size_of::<Point>(), 1);
    }
    #[test]
    #[should_panic]
    fn one_point_is_not_a_line_strip() {
        static POINTS: [Point; 1] = [Point::new(2, 0)];
        Strip::new(&POINTS, End, End);
    }
    #[test]
    fn repeated_points_remain_a_degenerate_line_not_a_synthetic_point() {
        const DOT: Strip = Strip::new(&[Point::new(2, 2), Point::new(2, 2)], End, End);
        assert_eq!(DOT.adjacency_vertex(1), DOT.adjacency_vertex(2));
        assert_eq!(DOT.segments(), 1);
    }
}
