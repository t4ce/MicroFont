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
    swap_xy: bool,
    offset_x: i8,
    offset_y: i8,
}
impl Strip {
    pub const fn new(points: &'static [Point], start: Boundary, end: Boundary) -> Self {
        assert!(points.len() >= 2);
        Self { points, start, end, flip_x_sum: u8::MAX, flip_y_sum: u8::MAX,
            swap_xy: false, offset_x: 0, offset_y: 0 }
    }
    pub const fn point(self, index: usize) -> Option<Point> {
        if index >= self.points.len() { return None; }
        let p = self.points[index];
        let (x, y) = if self.swap_xy { (p.y(), p.x()) } else { (p.x(), p.y()) };
        let x = if self.flip_x_sum == u8::MAX { x } else { self.flip_x_sum - x };
        let y = if self.flip_y_sum == u8::MAX { y } else { self.flip_y_sum - y };
        Some(Point::new((x as i16 + self.offset_x as i16) as u8,
                        (y as i16 + self.offset_y as i16) as u8))
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

/// Aliases change coordinate interpretation; point bytes and tags stay shared.
#[derive(Clone, Copy, Debug)]
pub enum GlyphTransform { MirrorX, MirrorY, Rotate90, Rotate180, Rotate270 }

#[derive(Clone, Copy, Debug)]
struct Transform {
    flip_x_sum: u8,
    flip_y_sum: u8,
    swap_xy: bool,
    offset_x: i8,
    offset_y: i8,
}
impl Transform {
    const IDENTITY: Self = Self { flip_x_sum: u8::MAX, flip_y_sum: u8::MAX,
        swap_xy: false, offset_x: 0, offset_y: 0 };
    fn apply(self, mut strip: Strip) -> Strip {
        strip.flip_x_sum = self.flip_x_sum;
        strip.flip_y_sum = self.flip_y_sum;
        strip.swap_xy = self.swap_xy;
        strip.offset_x = self.offset_x;
        strip.offset_y = self.offset_y;
        strip
    }
    fn for_glyph(source: GlyphBytes, transform: GlyphTransform) -> Option<Self> {
        let mut min_x = 5;
        let mut max_x = 0;
        let mut min_y = 10;
        let mut max_y = 0;
        let mut found = false;
        for strip in source.iter() {
            for p in strip.points {
                min_x = min_x.min(p.x());
                max_x = max_x.max(p.x());
                min_y = min_y.min(p.y());
                max_y = max_y.max(p.y());
                found = true;
            }
        }
        if !found { return None; }
        let quarter_turn = matches!(transform, GlyphTransform::Rotate90 | GlyphTransform::Rotate270);
        let (offset_x, offset_y) = if quarter_turn {
            if max_y - min_y >= 6 { return None; }
            let left = ((min_x as i16 + max_x as i16 - (max_y - min_y) as i16) / 2)
                .clamp(0, (5 - (max_y - min_y)) as i16);
            let top = ((min_y as i16 + max_y as i16 - (max_x - min_x) as i16) / 2)
                .clamp(0, (10 - (max_x - min_x)) as i16);
            (left as i8 - min_y as i8, top as i8 - min_x as i8)
        } else { (0, 0) };
        Some(Self {
            swap_xy: quarter_turn, offset_x, offset_y,
            flip_x_sum: match transform {
                GlyphTransform::MirrorY | GlyphTransform::Rotate270 => u8::MAX,
                GlyphTransform::MirrorX | GlyphTransform::Rotate180 => min_x + max_x,
                GlyphTransform::Rotate90 => min_y + max_y,
            },
            flip_y_sum: match transform {
                GlyphTransform::MirrorX | GlyphTransform::Rotate90 => u8::MAX,
                GlyphTransform::MirrorY | GlyphTransform::Rotate180 => min_y + max_y,
                GlyphTransform::Rotate270 => min_x + max_x,
            },
        })
    }
}

/// A borrowed sequence of fixed v1 strip records. Headers encode the point
/// count minus two in bits 0..5, start End in bit 6, and end End in bit 7.
/// Every point is decoded directly from the bundle; no strip table is stored.
#[derive(Clone, Copy, Debug)]
struct GlyphBytes {
    bytes: &'static [u8],
    referenced: bool,
    transform: Transform,
}
impl GlyphBytes {
    const EMPTY: Self = Self { bytes: &[], referenced: false, transform: Transform::IDENTITY };
    fn new(bytes: &'static [u8]) -> Option<Self> {
        let mut rest = bytes;
        while !rest.is_empty() {
            let count = (rest[0] & 63) as usize + 2;
            let points = rest.get(1..count + 1)?;
            if points.iter().any(|&p| p & 128 != 0 || p & 7 >= 6 || p >> 3 >= 11) {
                return None;
            }
            rest = &rest[count + 1..];
        }
        Some(Self { bytes, referenced: false, transform: Transform::IDENTITY })
    }
    fn references(bytes: &'static [u8]) -> Option<Self> {
        if bytes.len() % 4 != 0 { return None; }
        for id in bytes.chunks_exact(4) {
            let record = crate::bundle::record(u32::from_le_bytes(id.try_into().ok()?))?;
            if record.kind() != 1 { return None; }
            Self::new(record.raster())?;
        }
        Some(Self { bytes, referenced: true, transform: Transform::IDENTITY })
    }
    fn iter(self) -> RawStripIter {
        RawStripIter { remaining: self.bytes, active: &[],
            referenced: self.referenced, transform: self.transform }
    }
    fn transformed(mut self, transform: GlyphTransform) -> Option<Self> {
        self.transform = Transform::for_glyph(self, transform)?;
        Some(self)
    }
}

#[derive(Clone, Debug)]
struct RawStripIter {
    remaining: &'static [u8],
    active: &'static [u8],
    referenced: bool,
    transform: Transform,
}
impl Iterator for RawStripIter {
    type Item = Strip;
    fn next(&mut self) -> Option<Strip> {
        let input = if self.referenced {
            while self.active.is_empty() {
                let (id, remaining) = self.remaining.split_at_checked(4)?;
                self.remaining = remaining;
                self.active = crate::bundle::record(u32::from_le_bytes(id.try_into().ok()?))?.raster();
            }
            &mut self.active
        } else { &mut self.remaining };
        let (&header, rest) = input.split_first()?;
        let count = (header & 63) as usize + 2;
        let bytes = rest.get(..count)?;
        *input = &rest[count..];
        // SAFETY: Point is transparent over u8 and has alignment one. GlyphBytes
        // validates the full record before constructing this iterator. The
        // returned slice borrows the retained bundle, with headers excluded.
        let points = unsafe { core::slice::from_raw_parts(bytes.as_ptr().cast::<Point>(), count) };
        Some(self.transform.apply(Strip::new(points,
            if header & 64 != 0 { Boundary::End } else { Boundary::Junction },
            if header & 128 != 0 { Boundary::End } else { Boundary::Junction })))
    }
}

/// Read raw line records without allocating. An empty glyph has zero strips;
/// malformed headers, truncated data, and out-of-cell points return `None`.
pub fn decode_strips(bytes: &'static [u8]) -> Option<GlyphStrips> {
    Some(GlyphStrips::plain(GlyphBytes::new(bytes)?))
}

/// Private component identities, separate from standalone accent glyphs.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum LowercaseTopper { Diaeresis, Acute, Grave, Circumflex, Ring, Tilde }

/// Explicit composition covers the entire seven-base / six-topper field,
/// including pairs which Unicode represents only as combining sequences.
/// It does not register the toppers as ordinary character glyphs.
pub fn lowercase_topped_strips(base: char, topper: LowercaseTopper) -> Option<GlyphStrips> {
    let body = lowercase_base_strips(base)?;
    let top = lowercase_topper_strips(topper)?;
    Some(GlyphStrips::topped(body, top, base == 'i'))
}

/// Borrowed line records plus an optional shared lowercase topper. Iteration
/// decodes each strip by value while preserving its underlying point bytes.
#[derive(Clone, Copy, Debug)]
pub struct GlyphStrips {
    base: GlyphBytes,
    topper: GlyphBytes,
    omit_i_dot: bool,
}
impl GlyphStrips {
    fn plain(base: GlyphBytes) -> Self {
        Self { base, topper: GlyphBytes::EMPTY, omit_i_dot: false }
    }
    fn topped(base: GlyphBytes, topper: GlyphBytes, omit_i_dot: bool) -> Self {
        Self { base, topper, omit_i_dot }
    }
    pub fn len(self) -> usize { self.iter().count() }
    pub fn is_empty(self) -> bool { self.iter().next().is_none() }
    pub fn get(self, index: usize) -> Option<Strip> { self.iter().nth(index) }
    pub fn iter(self) -> GlyphStripIter {
        GlyphStripIter { base: self.base.iter(), topper: self.topper.iter(), omit_i_dot: self.omit_i_dot }
    }
}
#[derive(Clone, Debug)]
pub struct GlyphStripIter {
    base: RawStripIter,
    topper: RawStripIter,
    omit_i_dot: bool,
}
impl Iterator for GlyphStripIter {
    type Item = Strip;
    fn next(&mut self) -> Option<Self::Item> {
        for strip in self.base.by_ref() {
            // Omit only complete i-dot strokes; preserve all body coordinates
            // and endpoint tags, including transformed lowercase bodies.
            if !self.omit_i_dot || strip.iter_points().any(|p| p.y() >= 4) {
                return Some(strip);
            }
        }
        self.topper.next()
    }
}
impl IntoIterator for GlyphStrips {
    type Item = Strip;
    type IntoIter = GlyphStripIter;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

/// Scalar line glyph identities, generated from the selected binary bundle.
pub const GLYPH_CHARACTERS: &[char] = include!(concat!(env!("OUT_DIR"), "/line_characters.rs"));

fn source(id: u32) -> Option<GlyphBytes> {
    let record = crate::bundle::record(id)?;
    let bytes = record.raster();
    match record.kind() {
        1 => GlyphBytes::new(bytes),
        3 => {
            let (&transform, target) = bytes.split_last()?;
            if target.len() != 4 { return None; }
            let target = crate::bundle::record(u32::from_le_bytes(target.try_into().ok()?))?;
            // Aliases are canonical views. Restrict their source to raw data or
            // shared segment references, avoiding cycles and nested transforms.
            let original = match target.kind() {
                1 => GlyphBytes::new(target.raster())?,
                5 => GlyphBytes::references(target.raster())?,
                _ => return None,
            };
            original.transformed(match transform {
                1 => GlyphTransform::MirrorX, 2 => GlyphTransform::MirrorY,
                3 => GlyphTransform::Rotate90, 4 => GlyphTransform::Rotate180,
                5 => GlyphTransform::Rotate270, _ => return None,
            })
        }
        5 => GlyphBytes::references(bytes),
        _ => None,
    }
}
fn lowercase_base_strips(base: char) -> Option<GlyphBytes> {
    match base { 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'n' => source(base as u32), _ => None }
}
fn lowercase_topper_strips(topper: LowercaseTopper) -> Option<GlyphBytes> {
    source(0x110000 + topper as u32)
}

/// Read a glyph from the installed bundle. Missing raster data stays missing;
/// aliases, shared seven-segment strokes, and lowercase toppers share bytes.
pub fn glyph_strips(character: char) -> Option<GlyphStrips> {
    let record = crate::bundle::record(character as u32)?;
    if record.kind() == 4 {
        let bytes = record.raster();
        if bytes.len() != 9 { return None; }
        let base = source(u32::from_le_bytes(bytes[..4].try_into().ok()?))?;
        let topper_id = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
        let topper = if topper_id == 0 { GlyphBytes::EMPTY } else { source(topper_id)? };
        if bytes[8] > 1 { return None; }
        Some(GlyphStrips::topped(base, topper, bytes[8] != 0))
    } else {
        Some(GlyphStrips::plain(source(character as u32)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Boundary::{End, Junction};
    #[test]
    fn dynamic_decoder_rejects_malformed_records() {
        for bytes in [&[0xc0, 0][..], &[0xc0, 0, 0, 0][..],
            &[0xc0, 6, 0][..], &[0xc0, 88, 0][..], &[0xc0, 128, 0][..], &[0xff][..]] {
            assert!(decode_strips(bytes).is_none());
        }
        assert!(decode_strips(&[]).unwrap().is_empty());
        let glyph = decode_strips(&[0xc0, 0, 64, 0x80, 32, 68, 0x00, 0, 32]).unwrap();
        assert_eq!(glyph.len(), 3);
        assert_eq!(glyph.get(1).unwrap().start, Junction);
        assert_eq!(glyph.get(1).unwrap().end, End);
        assert!(glyph.get(3).is_none());
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
