//! CPU reconstruction of single-cell Basic Latin into indexed triangles.
//! Dista takes a distance envelope; Convo convolves and balances the same links.
//! No bitmap quads, font textures, or GPU field evaluation are used here.

use crate::{
    FontDetail, FontShape,
    raster::{GlyphSeed, basic_seed},
};
use alloc::{collections::BTreeMap, vec, vec::Vec};

/// Top-left source coordinates in the 6×11 cell. Indices wind clockwise when
/// displayed with Y pointing down. Each consecutive three indices is a triangle.
#[derive(Debug, Default)]
pub struct TriangleMesh {
    pub vertices: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

/// Dista: a rounded union of point and segment distance envelopes.
/// Control labels (including SP and DEL) and non-Basic characters return None.
pub fn dista_triangles(
    character: char,
    shape: FontShape,
    detail: FontDetail,
) -> Option<TriangleMesh> {
    reconstruct(character, shape, detail, false)
}

/// Convo: arclength convolution with endpoint caps and local density balance.
/// Control labels (including SP and DEL) and non-Basic characters return None.
pub fn convo_triangles(
    character: char,
    shape: FontShape,
    detail: FontDetail,
) -> Option<TriangleMesh> {
    reconstruct(character, shape, detail, true)
}

// The remaining recipe is shared and neutral: shortcut pruning is enabled;
// there is no direction bias, staircase assist, or centerline snapping.
const DENSITY_BALANCE: f64 = 0.66;
const SIGMA: f64 = FontShape::INFLUENCE_WIDTH;
const RADIUS: f64 = 3.5 * SIGMA;
type Point = [f64; 2];

struct Link {
    a: usize,
    b: usize,
    length: f64,
    weight: f64,
}
struct Kernel {
    point: Point,
    mass: f64,
}

fn reconstruct(
    character: char,
    shape: FontShape,
    detail: FontDetail,
    convo: bool,
) -> Option<TriangleMesh> {
    if !('!'..='~').contains(&character) {
        return None;
    }
    let seed = basic_seed(character)?;
    let points: Vec<Point> = (0..66)
        .filter(|&i| seed.pixels & (1u128 << (65 - i)) != 0)
        .map(|i| [(i % 6) as f64 + 0.5, (i / 6) as f64 + 0.5])
        .collect();
    if points.is_empty() {
        return Some(TriangleMesh::default());
    }
    let links = links(&points, seed);
    let kernels = if convo {
        convo_kernels(&points, &links)
    } else {
        points
            .iter()
            .map(|&point| Kernel { point, mass: 1.0 })
            .collect()
    };
    let mut field = Field::new(detail);
    for k in kernels {
        field.apply(
            [k.point[0] - RADIUS, k.point[1] - RADIUS],
            [k.point[0] + RADIUS, k.point[1] + RADIUS],
            convo,
            |p| k.mass * compact_gaussian(distance_squared(p, k.point) / (SIGMA * SIGMA)),
        );
    }
    if !convo {
        for link in links {
            let a = points[link.a];
            let b = points[link.b];
            field.apply(
                [a[0].min(b[0]) - RADIUS, a[1].min(b[1]) - RADIUS],
                [a[0].max(b[0]) + RADIUS, a[1].max(b[1]) + RADIUS],
                false,
                |p| {
                    let d = [b[0] - a[0], b[1] - a[1]];
                    let t = (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1])
                        / (link.length * link.length))
                        .clamp(0.0, 1.0);
                    link.weight
                        * compact_gaussian(
                            distance_squared(p, [a[0] + t * d[0], a[1] + t * d[1]])
                                / (SIGMA * SIGMA),
                        )
                },
            );
        }
    }
    Some(field.triangulate(shape.body_threshold()))
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

fn convo_kernels(points: &[Point], links: &[Link]) -> Vec<Kernel> {
    let mut kernels = Vec::new();
    let mut degree = vec![0.0; points.len()];
    for link in links {
        degree[link.a] += link.weight;
        degree[link.b] += link.weight;
        let steps = (libm::ceil(link.length / 0.22_f64.min(SIGMA * 0.55)) as usize).max(2);
        let mass = link.weight * link.length
            / (steps as f64 * libm::sqrt(2.0 * core::f64::consts::PI) * SIGMA);
        for i in 0..steps {
            let t = (i as f64 + 0.5) / steps as f64;
            kernels.push(Kernel {
                point: [
                    points[link.a][0] + (points[link.b][0] - points[link.a][0]) * t,
                    points[link.a][1] + (points[link.b][1] - points[link.a][1]) * t,
                ],
                mass,
            });
        }
    }
    for (i, &point) in points.iter().enumerate() {
        let mass = (1.0 - degree[i] * 0.5).max(0.0);
        if mass > 1e-5 {
            kernels.push(Kernel { point, mass });
        }
    }
    // Compute all densities from original masses before changing any mass.
    let densities: Vec<f64> = kernels
        .iter()
        .map(|a| {
            kernels
                .iter()
                .filter(|b| {
                    (a.point[0] - b.point[0]).abs() <= RADIUS
                        && (a.point[1] - b.point[1]).abs() <= RADIUS
                })
                .map(|b| {
                    b.mass * compact_gaussian(distance_squared(a.point, b.point) / (SIGMA * SIGMA))
                })
                .sum::<f64>()
                .max(0.10)
        })
        .collect();
    for (kernel, density) in kernels.iter_mut().zip(densities) {
        kernel.mass /= libm::pow(density, DENSITY_BALANCE);
    }
    kernels
}

struct Field {
    values: Vec<f32>,
    nx: usize,
    ny: usize,
    step: f64,
    origin: Point,
}

impl Field {
    fn new(detail: FontDetail) -> Self {
        let d = detail.subdivisions_per_cell();
        // One source unit of padding, as in the HTML at sigma=0.20. Roots are
        // 16×16 samples so uniform interiors can merge without losing holes.
        let nx = (8 * d).div_ceil(16) * 16;
        let ny = (13 * d).div_ceil(16) * 16;
        let step = 1.0 / d as f64;
        Self {
            values: vec![0.0; (nx + 1) * (ny + 1)],
            nx,
            ny,
            step,
            origin: [3.0 - nx as f64 * step * 0.5, 5.5 - ny as f64 * step * 0.5],
        }
    }

    fn apply(&mut self, low: Point, high: Point, additive: bool, value: impl Fn(Point) -> f64) {
        let bound = |v: f64, axis: usize, ceil: bool| {
            let grid = (v - self.origin[axis]) / self.step;
            let grid = if ceil {
                libm::ceil(grid)
            } else {
                libm::floor(grid)
            };
            grid.clamp(0.0, if axis == 0 { self.nx } else { self.ny } as f64) as usize
        };
        let xa = bound(low[0], 0, false);
        let xb = bound(high[0], 0, true);
        let ya = bound(low[1], 1, false);
        let yb = bound(high[1], 1, true);
        for y in ya..=yb {
            for x in xa..=xb {
                let v = value([
                    self.origin[0] + x as f64 * self.step,
                    self.origin[1] + y as f64 * self.step,
                ]);
                let sample = &mut self.values[y * (self.nx + 1) + x];
                if additive {
                    *sample = (*sample as f64 + v) as f32;
                } else {
                    *sample = (*sample as f64).max(v) as f32;
                }
            }
        }
    }

    fn sample(&self, x: f64, y: f64) -> f64 {
        let ix = x as usize;
        let iy = y as usize;
        let fx = x - ix as f64;
        let fy = y - iy as f64;
        let at = |x: usize, y: usize| {
            self.values[y.min(self.ny) * (self.nx + 1) + x.min(self.nx)] as f64
        };
        (at(ix, iy) * (1.0 - fx) + at(ix + 1, iy) * fx) * (1.0 - fy)
            + (at(ix, iy + 1) * (1.0 - fx) + at(ix + 1, iy + 1) * fx) * fy
    }

    fn triangulate(&self, threshold: f64) -> TriangleMesh {
        let stride = self.nx + 2;
        let mut sums = vec![0i32; stride * (self.ny + 2)];
        for y in 0..=self.ny {
            let mut row = 0;
            for x in 0..=self.nx {
                row += i32::from(self.values[y * (self.nx + 1) + x] as f64 >= threshold);
                sums[(y + 1) * stride + x + 1] = sums[y * stride + x + 1] + row;
            }
        }
        let mut leaves = Vec::new();
        for y in (0..self.ny).step_by(16) {
            for x in (0..self.nx).step_by(16) {
                split(&sums, stride, x, y, 16, &mut leaves);
            }
        }
        // Stitch neighboring coarse/fine leaves before clipping their triangle
        // fans. All boundary intersections are welded, preserving shared edges.
        let mut horizontal = vec![Vec::new(); self.ny + 1];
        let mut vertical = vec![Vec::new(); self.nx + 1];
        for &[x, y, size] in &leaves {
            for [xx, yy] in [[x, y], [x + size, y], [x + size, y + size], [x, y + size]] {
                horizontal[yy].push(xx);
                vertical[xx].push(yy);
            }
        }
        for line in horizontal.iter_mut().chain(vertical.iter_mut()) {
            line.sort_unstable();
            line.dedup();
        }
        let mut builder = MeshBuilder {
            field: self,
            threshold,
            mesh: TriangleMesh::default(),
            weld: BTreeMap::new(),
        };
        for [x, y, size] in leaves {
            let mut ring = Vec::new();
            let mut add = |x: usize, y: usize| {
                ring.push([x as f64, y as f64, self.sample(x as f64, y as f64)])
            };
            for &t in &horizontal[y] {
                if t >= x && t < x + size {
                    add(t, y);
                }
            }
            for &t in &vertical[x + size] {
                if t >= y && t < y + size {
                    add(x + size, t);
                }
            }
            for &t in horizontal[y + size].iter().rev() {
                if t > x && t <= x + size {
                    add(t, y + size);
                }
            }
            for &t in vertical[x].iter().rev() {
                if t > y && t <= y + size {
                    add(x, t);
                }
            }
            let cx = x as f64 + size as f64 * 0.5;
            let cy = y as f64 + size as f64 * 0.5;
            let center = [cx, cy, self.sample(cx, cy)];
            for i in 0..ring.len() {
                builder.clip_triangle([center, ring[i], ring[(i + 1) % ring.len()]]);
            }
        }
        builder.mesh
    }
}

fn split(
    sums: &[i32],
    stride: usize,
    x: usize,
    y: usize,
    size: usize,
    leaves: &mut Vec<[usize; 3]>,
) {
    let count = sums[(y + size + 1) * stride + x + size + 1]
        - sums[y * stride + x + size + 1]
        - sums[(y + size + 1) * stride + x]
        + sums[y * stride + x];
    if count == 0 {
        return;
    }
    if count as usize == (size + 1) * (size + 1) || size == 1 {
        leaves.push([x, y, size]);
        return;
    }
    let half = size / 2;
    for [xx, yy] in [[x, y], [x + half, y], [x, y + half], [x + half, y + half]] {
        split(sums, stride, xx, yy, half, leaves);
    }
}

struct MeshBuilder<'a> {
    field: &'a Field,
    threshold: f64,
    mesh: TriangleMesh,
    weld: BTreeMap<(i64, i64), u32>,
}

impl MeshBuilder<'_> {
    fn vertex(&mut self, p: [f64; 3]) -> u32 {
        let key = (libm::round(p[0] * 1e8) as i64, libm::round(p[1] * 1e8) as i64);
        if let Some(&index) = self.weld.get(&key) {
            return index;
        }
        let index = self.mesh.vertices.len() as u32;
        self.mesh.vertices.push([
            (self.field.origin[0] + p[0] * self.field.step) as f32,
            (self.field.origin[1] + p[1] * self.field.step) as f32,
        ]);
        self.weld.insert(key, index);
        index
    }

    fn clip_triangle(&mut self, source: [[f64; 3]; 3]) {
        let mut polygon = [[0.0; 3]; 4];
        let mut count = 0;
        for i in 0..3 {
            let u = source[i];
            let v = source[(i + 1) % 3];
            let inside = u[2] >= self.threshold;
            if inside {
                polygon[count] = u;
                count += 1;
            }
            if inside != (v[2] >= self.threshold) {
                let t = (self.threshold - u[2]) / (v[2] - u[2]);
                polygon[count] = [
                    u[0] + (v[0] - u[0]) * t,
                    u[1] + (v[1] - u[1]) * t,
                    self.threshold,
                ];
                count += 1;
            }
        }
        for i in 1..count.saturating_sub(1) {
            let [a, b, c] = [polygon[0], polygon[i], polygon[i + 1]];
            if ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() < 1e-10 {
                continue;
            }
            let indices = [self.vertex(a), self.vertex(b), self.vertex(c)];
            if indices[0] != indices[1] && indices[1] != indices[2] && indices[2] != indices[0] {
                self.mesh.indices.extend_from_slice(&indices);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;

    // Check welded topology rather than just comparing a builder with itself:
    // interior edges have two faces, each boundary vertex has two neighbors,
    // and Euler's formula gives the expected holes and connected bodies.
    fn topology(mesh: &TriangleMesh) -> (usize, usize, f64) {
        let mut edges = BTreeMap::new();
        let mut parents: Vec<usize> = (0..mesh.vertices.len()).collect();
        fn root(parents: &mut [usize], mut p: usize) -> usize {
            while parents[p] != p {
                parents[p] = parents[parents[p]];
                p = parents[p];
            }
            p
        }
        let mut area = 0.0;
        assert_eq!(mesh.indices.len() % 3, 0);
        for t in mesh.indices.chunks_exact(3) {
            let [a, b, c] = [t[0], t[1], t[2]];
            let [pa, pb, pc] = [a, b, c].map(|i| mesh.vertices[i as usize]);
            let cross = (pb[0] - pa[0]) as f64 * (pc[1] - pa[1]) as f64
                - (pb[1] - pa[1]) as f64 * (pc[0] - pa[0]) as f64;
            assert!(cross > 0.0, "inverted or degenerate triangle");
            area += cross * 0.5;
            for (a, b) in [(a, b), (b, c), (c, a)] {
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                let ra = root(&mut parents, a as usize);
                let rb = root(&mut parents, b as usize);
                parents[ra] = rb;
            }
        }
        let mut boundary_degree = vec![0; mesh.vertices.len()];
        for (&(a, b), &count) in &edges {
            assert!(count == 1 || count == 2, "nonmanifold edge");
            if count == 1 {
                boundary_degree[a as usize] += 1;
                boundary_degree[b as usize] += 1;
            }
        }
        assert!(boundary_degree.iter().all(|&d| d == 0 || d == 2), "open boundary or T-junction");
        let components = (0..parents.len())
            .map(|i| root(&mut parents, i))
            .collect::<BTreeSet<_>>()
            .len();
        let holes = components + edges.len() - mesh.vertices.len() - mesh.indices.len() / 3;
        (components, holes, area)
    }

    #[test]
    fn printable_basic_has_closed_triangle_geometry_in_all_four_modes() {
        for ch in '!'..='~' {
            for build in [dista_triangles, convo_triangles] {
                for shape in [FontShape::Meso, FontShape::Lean] {
                    let mesh = build(ch, shape, FontDetail::Fine).unwrap();
                    assert!(!mesh.indices.is_empty(), "missing {ch:?}");
                    for &[x, y] in &mesh.vertices {
                        assert!(x.is_finite() && y.is_finite());
                        assert!(
                            (0.0..=6.0).contains(&x) && (0.0..=11.0).contains(&y),
                            "outside cell: {ch:?} {x} {y}"
                        );
                    }
                    topology(&mesh);
                }
            }
        }
    }

    #[test]
    fn reach_shapes_holes_and_detail_change_real_geometry() {
        for build in [dista_triangles, convo_triangles] {
            for shape in [FontShape::Meso, FontShape::Lean] {
                let slash = build('/', shape, FontDetail::Fine).unwrap();
                let (bodies, holes, _) = topology(&slash);
                assert_eq!((bodies, holes), (1, 0), "sparse long-reach slash must be one ribbon");
                let equals = build('=', shape, FontDetail::Fine).unwrap();
                assert_eq!(topology(&equals).0, 2, "separated strokes must stay separated");
                let ring = build('0', shape, FontDetail::Fine).unwrap();
                assert_eq!(topology(&ring).1, 1, "zero must retain its counter");
                let high = build('0', shape, FontDetail::High).unwrap();
                assert!(high.indices.len() > ring.indices.len());
                assert!((topology(&high).2 / topology(&ring).2 - 1.0).abs() < 0.03);
            }
            let meso = build('A', FontShape::Meso, FontDetail::Fine).unwrap();
            let lean = build('A', FontShape::Lean, FontDetail::Fine).unwrap();
            assert!(topology(&meso).2 > topology(&lean).2);
            for ch in ['\0', '\n', ' ', '\u{7f}', 'ü', '文'] {
                assert!(build(ch, FontShape::Meso, FontDetail::Fine).is_none());
            }
        }
    }
}
