//! Shared scalar-grid sampling, adaptive triangulation and vertex welding.

use super::{Point, TriangleMesh};
use crate::FontDetail;
use alloc::{collections::BTreeMap, vec, vec::Vec};

pub(super) struct Field {
    values: Vec<f32>,
    nx: usize,
    ny: usize,
    step: f64,
    origin: Point,
}

impl Field {
    pub(super) fn new(detail: FontDetail) -> Self {
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

    pub(super) fn max_with(&mut self, low: Point, high: Point, value: impl Fn(Point) -> f64) {
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
                *sample = (*sample as f64).max(v) as f32;
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

    pub(super) fn triangulate(&self, threshold: f64) -> TriangleMesh {
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
