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
    pub points: &'static [Point],
    pub start: Boundary,
    pub end: Boundary,
}
impl Strip {
    pub const fn new(points: &'static [Point], start: Boundary, end: Boundary) -> Self {
        assert!(points.len() >= 2);
        Self { points, start, end }
    }
    pub const fn segments(self) -> usize { self.points.len() - 1 }
    /// Source coordinates plus z: real vertices use depth 0.5; adjacency-only
    /// vertices duplicate the boundary position and carry Junction=0 / End=1.
    /// This is a demo transport contract, not a generic position.z convention.
    pub fn adjacency_vertex(self, index: usize) -> Option<[f32; 3]> {
        let (point, z) = if index == 0 {
            (*self.points.first()?, self.start as u8 as f32)
        } else if index <= self.points.len() {
            (self.points[index - 1], 0.5)
        } else if index == self.points.len() + 1 {
            (*self.points.last()?, self.end as u8 as f32)
        } else { return None; };
        Some([point.x() as f32, point.y() as f32, z])
    }
}

use Boundary::End;
// One glyph override may contain any number of independent strips.
const BAR: &[Strip] = &[Strip::new(&[Point::new(2, 0), Point::new(2, 8)], End, End)];
const SLASH: &[Strip] = &[Strip::new(&[Point::new(4, 0), Point::new(0, 8)], End, End)];
pub const fn glyph_strips(character: char) -> Option<&'static [Strip]> {
    match character { '|' => Some(BAR), '/' => Some(SLASH), _ => None }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(glyph_strips('T').is_none()); // Example below is a format test only.
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
