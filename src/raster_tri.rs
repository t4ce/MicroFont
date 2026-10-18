//! Dista triangles for single-cell Basic Latin and Braille.
//! Dista uses a rounded distance envelope around sites and straight links.
//! Braille dots use direct circle fans at their original Microfont positions.
//! No bitmap quads, font textures, or GPU field evaluation are used here.

use crate::{
    FontDetail, FontShape,
    raster::{GlyphSeed, basic_seed},
};
use alloc::vec::Vec;

mod mesh;
use mesh::Field;
#[cfg(test)]
mod tests;

/// Top-left source coordinates in the 6×11 cell. Indices wind clockwise when
/// displayed with Y pointing down. Each consecutive three indices is a triangle.
#[derive(Debug, Default)]
pub struct TriangleMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Dista: Basic Latin distance envelopes or direct Braille dot circles.
/// Control labels (including SP and DEL) and other characters return None.
pub fn dista_triangles(
    character: char,
    shape: FontShape,
    detail: FontDetail,
) -> Option<TriangleMesh> {
    if ('\u{2800}'..='\u{28ff}').contains(&character) {
        return dista_braille_triangles(character, shape, detail);
    }
    let Skeleton { sites, links } = basic_skeleton(character)?;
    let mut field = Field::new(detail);
    // Dista is the nearest union: points and complete straight links.
    for &point in &sites {
        field.max_with(
            [point[0] - RADIUS, point[1] - RADIUS],
            [point[0] + RADIUS, point[1] + RADIUS],
            |p| compact_gaussian(distance_squared(p, point) / (SIGMA * SIGMA)),
        );
    }
    for link in links {
        let a = sites[link.a];
        let b = sites[link.b];
        field.max_with(
            [a[0].min(b[0]) - RADIUS, a[1].min(b[1]) - RADIUS],
            [a[0].max(b[0]) + RADIUS, a[1].max(b[1]) + RADIUS],
            |p| {
                let d = [b[0] - a[0], b[1] - a[1]];
                let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1])
                    / (link.length * link.length))
                    .clamp(0.0, 1.0);
                link.weight
                    * compact_gaussian(
                        distance_squared(p, [a[0] + t * d[0], a[1] + t * d[1]]) / (SIGMA * SIGMA),
                    )
            },
        );
    }
    Some(field.triangulate(shape.body_threshold()))
}

/// Braille as independent circle fans, sharing the existing pixel/quad centres.
/// Fine uses 36 triangles per lit dot; High uses 72. Blank Braille is empty;
/// other characters return None. Dot size follows Dista Meso/Lean exactly.
pub fn dista_braille_triangles(
    character: char,
    shape: FontShape,
    detail: FontDetail,
) -> Option<TriangleMesh> {
    let dots = crate::raster::braille_quad_slice(character)?;
    if dots.is_empty() {
        return Some(TriangleMesh::default());
    }
    // The Dista point envelope is exp(-r² / (2σ²)). Solve at the same body
    // threshold to get its radius directly, without sampling a field or mesh.
    let radius = SIGMA * libm::sqrt(-2.0 * libm::log(shape.body_threshold()));
    let segments = detail.braille_triangles_per_dot();
    let ring: Vec<[f32; 2]> = (0..segments)
        .map(|i| {
            let angle = core::f64::consts::TAU * i as f64 / segments as f64;
            [
                (radius * libm::cos(angle)) as f32,
                (radius * libm::sin(angle)) as f32,
            ]
        })
        .collect();
    let mut mesh = TriangleMesh {
        vertices: Vec::with_capacity(dots.len() * (segments + 1)),
        indices: Vec::with_capacity(dots.len() * segments * 3),
    };
    for dot in dots {
        let center = [
            (f32::from(dot.top_left[0]) + f32::from(dot.bottom_right[0])) * 0.5,
            (f32::from(dot.top_left[1]) + f32::from(dot.bottom_right[1])) * 0.5,
        ];
        let base = mesh.vertices.len() as u32;
        mesh.vertices.push(center);
        mesh.vertices
            .extend(ring.iter().map(|&[x, y]| [center[0] + x, center[1] + y]));
        for i in 0..segments {
            mesh.indices.extend_from_slice(&[
                base,
                base + 1 + i as u32,
                base + 1 + ((i + 1) % segments) as u32,
            ]);
        }
    }
    Some(mesh)
}

// The remaining recipe is shared and neutral: shortcut pruning is enabled;
// there is no direction bias, staircase assist, or centerline snapping.
const SIGMA: f64 = FontShape::INFLUENCE_WIDTH;
const RADIUS: f64 = 3.5 * SIGMA;
type Point = [f64; 2];

struct Link {
    a: usize,
    b: usize,
    length: f64,
    weight: f64,
}
struct Skeleton {
    sites: Vec<Point>,
    links: Vec<Link>,
}

fn basic_skeleton(character: char) -> Option<Skeleton> {
    if !('!'..='~').contains(&character) {
        return None;
    }
    let seed = basic_seed(character)?;
    let sites: Vec<Point> = (0..66)
        .filter(|&i| seed.pixels & (1u128 << (65 - i)) != 0)
        .map(|i| [(i % 6) as f64 + 0.5, (i / 6) as f64 + 0.5])
        .collect();
    let links = links(&sites, seed);
    Some(Skeleton { sites, links })
}

fn distance_squared(a: Point, b: Point) -> f64 {
    (a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1])
}

fn links(points: &[Point], seed: GlyphSeed) -> Vec<Link> {
    let mut links = Vec::new();
    for a in 0..points.len() {
        for b in a + 1..points.len() {
            let d2 = distance_squared(points[a], points[b]);
            let length = libm::sqrt(d2);
            if length >= seed.connection_reach() {
                continue;
            }
            let midpoint = [
                (points[a][0] + points[b][0]) * 0.5,
                (points[a][1] + points[b][1]) * 0.5,
            ];
            // Closed diameter disc: ties suppress redundant diagonal shortcuts.
            if points.iter().enumerate().any(|(k, &p)| {
                k != a && k != b && distance_squared(p, midpoint) <= d2 * 0.25 + 1e-9
            }) {
                continue;
            }
            let t = ((seed.connection_reach() - length) / 0.30).clamp(0.0, 1.0);
            links.push(Link {
                a,
                b,
                length,
                weight: t * t * (3.0 - 2.0 * t),
            });
        }
    }
    links
}

fn compact_gaussian(q2: f64) -> f64 {
    if q2 >= 12.25 {
        return 0.0;
    }
    let v = libm::exp(-q2 * 0.5);
    if q2 <= 9.0 {
        return v;
    }
    let t = (libm::sqrt(q2) - 3.0) / 0.5;
    v * (1.0 - t * t * (3.0 - 2.0 * t))
}
