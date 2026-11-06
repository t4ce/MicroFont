//! Convert raster glyphs to a caller-owned sink with no window/GPU dependency.
//! cargo run --example geometry_sink --no-default-features --features raster
//! The triangle sink uses the vertex contract in shaders/glyph.{vert,frag}.
use microfont::raster_line::{Boundary, Point, glyph_strips};
use microfont::raster_quad::glyph_quads;

/// A software renderer may implement these methods directly instead of making
/// triangles. Positions and widths are already in target pixel coordinates.
trait Sink {
    fn line(&mut self, a: [f32; 2], b: [f32; 2], width: f32, start: Boundary, end: Boundary);
    fn dot(&mut self, center: [f32; 2], width: f32);
    fn triangle(&mut self, points: [[f32; 2]; 3]);
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Vertex {
    position_px: [f32; 2], // location 0, two floats, byte offset 0
    segment_px: [f32; 4],  // location 1, four floats, byte offset 8
    style: [f32; 4],       // location 2, four floats, byte offset 24
    rgba: [f32; 4],        // location 3, four floats, byte offset 40
}

struct Triangles {
    vertices: Vec<Vertex>,
    rgba: [f32; 4],
    lines: usize,
    dots: usize,
    fills: usize,
}
impl Triangles {
    fn primitive(&mut self, points: [[f32; 2]; 3], segment_px: [f32; 4], style: [f32; 4]) {
        self.vertices.extend(points.map(|position_px| Vertex {
            position_px,
            segment_px,
            style,
            rgba: self.rgba,
        }));
    }
    fn bounds(&mut self, a: [f32; 2], b: [f32; 2], style: [f32; 4]) {
        // A generous bounding box includes the complete 1px coverage fringe.
        let margin = style[0] + 1.0;
        let [left, top] = [a[0].min(b[0]) - margin, a[1].min(b[1]) - margin];
        let [right, bottom] = [a[0].max(b[0]) + margin, a[1].max(b[1]) + margin];
        let segment = [a[0], a[1], b[0], b[1]];
        self.primitive([[left, top], [right, top], [right, bottom]], segment, style);
        self.primitive(
            [[left, top], [right, bottom], [left, bottom]],
            segment,
            style,
        );
    }
}
impl Sink for Triangles {
    fn line(&mut self, a: [f32; 2], b: [f32; 2], width: f32, start: Boundary, end: Boundary) {
        if a == b {
            self.dot(a, width); // Never normalize a zero-length segment.
            return;
        }
        self.lines += 1;
        self.bounds(
            a,
            b,
            [
                width * 0.5,
                (start == Boundary::End) as u8 as f32,
                (end == Boundary::End) as u8 as f32,
                1.0,
            ],
        );
    }
    fn dot(&mut self, center: [f32; 2], width: f32) {
        self.dots += 1;
        self.bounds(center, center, [width * 0.5, 1.0, 1.0, 2.0]);
    }
    fn triangle(&mut self, points: [[f32; 2]; 3]) {
        self.fills += 1;
        self.primitive(points, [0.0; 4], [0.0; 4]);
    }
}

fn glyph(sink: &mut impl Sink, character: char, origin: [f32; 2], scale: f32, width: f32) -> bool {
    if character == ' ' {
        return true;
    } // A space advances without geometry.
    let place = |xy: [f32; 2]| [origin[0] + xy[0] * scale, origin[1] + xy[1] * scale];
    if let Some(strips) = glyph_strips(character) {
        for strip in strips {
            // Do not read strip.points directly: aliases can mirror/rotate the
            // coordinates. iter_points() applies that glyph's transformation.
            let center = |point: Point| place([point.x() as f32 + 0.5, point.y() as f32 + 0.5]);
            let mut points = strip.iter_points();
            let mut previous = center(points.next().unwrap());
            let segments = strip.segments();
            for (index, point) in points.enumerate() {
                let next = center(point);
                sink.line(
                    previous,
                    next,
                    width,
                    if index == 0 {
                        strip.start
                    } else {
                        Boundary::Junction
                    },
                    if index + 1 == segments {
                        strip.end
                    } else {
                        Boundary::Junction
                    },
                );
                if index + 1 < segments {
                    // This sample chooses round internal joins. Native line
                    // strips or a different join policy are equally valid sinks.
                    sink.dot(next, width);
                }
                previous = next;
            }
        }
        true
    } else if let Some(quads) = glyph_quads(character) {
        for quad in quads {
            let [a, b, c, d] = quad.vertices.map(place);
            sink.triangle([a, b, c]);
            sink.triangle([a, c, d]);
        }
        true
    } else {
        false // Missing data stays missing; caller chooses its fallback.
    }
}

fn main() {
    let mut sink = Triangles {
        vertices: Vec::new(),
        rgba: [0.9, 0.95, 1.0, 1.0],
        lines: 0,
        dots: 0,
        fills: 0,
    };
    let text = "MicroFont .: ╭─╮ ▓ ⣿";
    let mut missing = 0;
    for (column, character) in text.chars().enumerate() {
        missing += !glyph(
            &mut sink,
            character,
            [12.0 + column as f32 * 24.0, 12.0],
            4.0,
            3.0,
        ) as usize;
    }
    assert_eq!(
        missing, 0,
        "sample text must be covered by the raster bundle"
    );
    assert!(sink.lines > 0 && sink.dots > 0 && sink.fills > 0);
    assert_eq!(core::mem::size_of::<Vertex>(), 56);
    assert!(sink.vertices.iter().all(|v| {
        v.position_px
            .iter()
            .chain(v.segment_px.iter())
            .chain(v.style.iter())
            .all(|x| x.is_finite())
    }));
    println!(
        "{} line segments, {} dots/joins, {} filled triangles -> {} vertices ({} bytes)",
        sink.lines,
        sink.dots,
        sink.fills,
        sink.vertices.len(),
        sink.vertices.len() * core::mem::size_of::<Vertex>()
    );
    println!(
        "Upload as non-indexed triangles; vertex stride is {} bytes.",
        core::mem::size_of::<Vertex>()
    );
}
