//! Shared no_std recipe for a quarter-circle stroke and two straight arms.
//! Source geometry is independent of bitmap masks and renderer allocations.
pub const MIN_CURVE_QUADS: u8 = 4;
pub const MAX_CURVE_QUADS: u8 = 16;

#[derive(Clone, Copy, Debug)]
pub struct CornerRecipe {
    pub orientation: u8,
    pub junction: [f32; 2],
    pub radius: f32,
    pub width: f32,
}

impl CornerRecipe {
    /// Orientation 0/1/2/3 is ╭/╮/╯/╰. All lengths use the 6×11 source cell.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 17 || bytes[0] > 3 {
            return None;
        }
        let f = |at| f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let result = Self {
            orientation: bytes[0],
            junction: [f(1), f(5)],
            radius: f(9),
            width: f(13),
        };
        let [x, y] = result.junction;
        let h = result.width * 0.5;
        let [cx, cy] = result.centre();
        if [x, y, result.radius, result.width]
            .iter()
            .any(|v| !v.is_finite())
            || result.radius <= 0.0
            || result.width <= 0.0
            || h >= result.radius
            || x - h < 0.0
            || x + h > 6.0
            || y - h < 0.0
            || y + h > 11.0
            || cx <= 0.0
            || cx >= 6.0
            || cy <= 0.0
            || cy >= 11.0
        {
            return None;
        }
        Some(result)
    }
    fn signs(self) -> [f32; 2] {
        [
            if matches!(self.orientation, 0 | 3) {
                1.0
            } else {
                -1.0
            },
            if self.orientation < 2 { 1.0 } else { -1.0 },
        ]
    }
    fn centre(self) -> [f32; 2] {
        let [sx, sy] = self.signs();
        [
            self.junction[0] + sx * self.radius,
            self.junction[1] + sy * self.radius,
        ]
    }
    /// Physical-size floor: 4 at 10mm, 16 at 40mm. Increase it if needed to
    /// keep the projected outer arc's chord error within a quarter pixel.
    /// With no physical metrics, only pixel curvature decides the budget.
    pub fn curve_quads(self, em_height_px: u32, em_10mm_px: Option<u32>) -> u8 {
        let floor = em_10mm_px
            .filter(|&n| n != 0)
            .map_or(MIN_CURVE_QUADS, |reference| {
                (4 * u64::from(em_height_px))
                    .div_ceil(u64::from(reference))
                    .clamp(4, 16) as u8
            });
        let outer_px = (self.radius + self.width * 0.5) * em_height_px as f32 / 11.0;
        let mut count = floor;
        while count < MAX_CURVE_QUADS {
            let (cos, _) = quarter_circle(core::f32::consts::FRAC_PI_2 / (2.0 * count as f32));
            if outer_px * (1.0 - cos) <= 0.25 {
                break;
            }
            count += 1;
        }
        count
    }
    /// Two rectangles, followed by non-overlapping annular-sector quads.
    /// Curve endpoints are exact, so their edges meet the arms without seams.
    pub fn quad(self, index: usize, curve_quads: u8) -> Option<[[f32; 2]; 4]> {
        let count = curve_quads.clamp(MIN_CURVE_QUADS, MAX_CURVE_QUADS) as usize;
        let [x, y] = self.junction;
        let [cx, cy] = self.centre();
        let [sx, sy] = self.signs();
        let half = self.width * 0.5;
        if index == 0 {
            return Some(rectangle(
                x - half,
                if sy > 0.0 { cy } else { 0.0 },
                x + half,
                if sy > 0.0 { 11.0 } else { cy },
            ));
        }
        if index == 1 {
            return Some(rectangle(
                if sx > 0.0 { cx } else { 0.0 },
                y - half,
                if sx > 0.0 { 6.0 } else { cx },
                y + half,
            ));
        }
        let segment = index - 2;
        if segment >= count {
            return None;
        }
        let at = |step: usize, radius: f32| {
            let (cos, sin) = if step == 0 {
                (1.0, 0.0)
            } else if step == count {
                (0.0, 1.0)
            } else {
                quarter_circle(core::f32::consts::FRAC_PI_2 * step as f32 / count as f32)
            };
            [cx - sx * radius * cos, cy - sy * radius * sin]
        };
        let mut vertices = [
            at(segment, self.radius - half),
            at(segment, self.radius + half),
            at(segment + 1, self.radius + half),
            at(segment + 1, self.radius - half),
        ];
        if sx * sy < 0.0 {
            vertices.swap(1, 3);
        }
        Some(vertices)
    }
}

fn rectangle(left: f32, top: f32, right: f32, bottom: f32) -> [[f32; 2]; 4] {
    [[left, top], [right, top], [right, bottom], [left, bottom]]
}

// Taylor on [0, π/2], through x^13. No libm/std dependency is needed. Exact
// end coordinates are supplied by quad(); adjacent quads reuse identical math.
fn quarter_circle(angle: f32) -> (f32, f32) {
    fn sin(x: f32) -> f32 {
        let x2 = x * x;
        x * (1.0
            + x2 * (-1.0 / 6.0
                + x2 * (1.0 / 120.0
                    + x2 * (-1.0 / 5040.0
                        + x2 * (1.0 / 362880.0 + x2 * (-1.0 / 39916800.0 + x2 / 6227020800.0))))))
    }
    (sin(core::f32::consts::FRAC_PI_2 - angle), sin(angle))
}
