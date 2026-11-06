//! Rebuild the release atlases from the actual installed font geometry.
//!
//! cargo run --example atlas_previews --no-default-features --features hex,raster
//! The optional first argument is the output directory (default: docs).
//! SVG generation uses only Rust's standard library and MicroFont itself.
use microfont::{bundle, raster_line, raster_quad, CP850_CHARACTERS};
use std::{fmt::Write as _, fs, path::PathBuf};

const WIDTH: usize = 1120;
const BG: &str = "#0b1118";
const PANEL: &str = "#101923";
const WHITE: &str = "#edf0e7";
const MUTED: &str = "#a3b3bd";
const DIM: &str = "#536676";
const RULE: &str = "#233442";
const MISSING: &str = "#e8a26d";
const COLORS: [&str; 8] = [
    "#ebd99b", "#b4d7b2", "#82c9bf", "#83bfda", "#a7b8ed", "#c2abe3", "#d7b5c7", "#dac6af",
];

#[derive(Clone, Copy)]
enum Mode {
    Hex,
    Raster,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Hex => "HEX",
            Self::Raster => "RASTER",
        }
    }
    fn filename(self) -> &'static str {
        match self {
            Self::Hex => "atlas-hex.svg",
            Self::Raster => "atlas-raster.svg",
        }
    }
    fn available(self, ch: char) -> bool {
        if intentional_blank(ch) {
            return true;
        }
        match self {
            Self::Hex => bundle::bitmap(ch).is_some_and(|bits| bits != 0),
            Self::Raster => {
                raster_line::glyph_strips(ch).is_some_and(|s| !s.is_empty())
                    || raster_quad::glyph_quads(ch).is_some_and(|q| !q.is_empty())
            }
        }
    }
}

fn intentional_blank(ch: char) -> bool {
    matches!(ch, '\0' | ' ' | '\u{a0}' | '\u{2800}')
}

struct Atlas {
    out: String,
    mode: Mode,
}

impl Atlas {
    fn new(mode: Mode) -> Self {
        Self {
            out: String::new(),
            mode,
        }
    }
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, fill: &str) {
        writeln!(
            self.out,
            "<rect x='{x}' y='{y}' width='{w}' height='{h}' fill='{fill}'/>"
        )
        .unwrap();
    }
    fn rule(&mut self, y: f32) {
        self.rect(64.0, y, 992.0, 1.0, RULE);
    }
    // Labels also use MicroFont, never the browser's installed fonts.
    fn label(&mut self, x: f32, y: f32, s: &str, scale: f32, color: &str) {
        for (i, ch) in s.chars().enumerate() {
            self.bitmap(x + i as f32 * 6.0 * scale, y, ch, scale, color);
        }
    }
    fn bitmap(&mut self, x: f32, y: f32, ch: char, scale: f32, color: &str) {
        let Some(bits) = bundle::bitmap(ch) else {
            return;
        };
        write!(self.out, "<path fill='{color}' shape-rendering='crispEdges' transform='translate({x} {y}) scale({scale})' d='").unwrap();
        // Coalesce each row into runs of lit pixels without changing the mask.
        for row in 0..11 {
            let mut col = 0;
            while col < 6 {
                if bits & (1 << (65 - row * 6 - col)) == 0 {
                    col += 1;
                    continue;
                }
                let start = col;
                while col < 6 && bits & (1 << (65 - row * 6 - col)) != 0 {
                    col += 1;
                }
                write!(
                    self.out,
                    "M{start} {row}h{}v1h-{}z",
                    col - start,
                    col - start
                )
                .unwrap();
            }
        }
        self.out.push_str("'/>\n");
    }
    fn glyph(&mut self, x: f32, y: f32, ch: char, scale: f32, color: &str) {
        match self.mode {
            Mode::Hex => self.bitmap(x, y, ch, scale, color),
            Mode::Raster => {
                write!(
                    self.out,
                    "<g color='{color}' transform='translate({x} {y}) scale({scale})'>"
                )
                .unwrap();
                microfont::svg::write_glyph(&mut self.out, ch, 0.72).unwrap();
                self.out.push_str("</g>\n");
            }
        }
    }
    fn specimen(&mut self, x: f32, y: f32, s: &str, scale: f32, color: &str) {
        for (i, ch) in s.chars().enumerate() {
            if self.mode.available(ch) {
                self.glyph(x + i as f32 * scale * 6.0, y, ch, scale, color);
            }
        }
    }
    fn empty(&mut self, x: f32, y: f32, scale: f32) {
        writeln!(self.out, "<rect x='{x}' y='{y}' width='{}' height='{}' fill='none' stroke='{DIM}' stroke-width='0.8'/>", scale * 6.0, scale * 11.0).unwrap();
    }
    fn missing(&mut self, x: f32, y: f32, scale: f32) {
        let (w, h) = (scale * 6.0, scale * 11.0);
        writeln!(self.out, "<rect x='{x}' y='{y}' width='{w}' height='{h}' rx='1' fill='none' stroke='{RULE}' stroke-width='0.8'/><path d='M{} {}l{} {}m-{} 0l{} -{}' fill='none' stroke='{MISSING}' stroke-width='1.1'/>", x + w * 0.25, y + h * 0.35, w * 0.5, h * 0.3, w * 0.5, w * 0.5, h * 0.3).unwrap();
    }
    fn section(&mut self, y: f32, title: &str, sub: &str, color: &str) {
        self.rule(y - 20.0);
        self.label(64.0, y, title, 1.85, color);
        self.label(64.0, y + 30.0, sub, 1.0, MUTED);
    }
    fn grid(&mut self, y: f32, chars: &[char], columns: usize, cp850: bool) -> f32 {
        let cell_width = 960.0 / columns as f32;
        let (scale, row_height, label_scale) = if columns == 16 {
            (3.3, 66.0, 0.85)
        } else {
            (2.3, 47.0, 0.65)
        };
        let rows = chars.len().div_ceil(columns);
        for row in 0..rows {
            let top = y + row as f32 * row_height;
            if row % 2 == 0 {
                self.rect(88.0, top - 4.0, 976.0, row_height, PANEL);
            }
            if cp850 {
                self.label(64.0, top + 12.0, &format!("{:X}_", row), 0.8, DIM);
            }
            for col in 0..columns {
                let index = row * columns + col;
                let Some(&ch) = chars.get(index) else {
                    break;
                };
                let x = 96.0 + col as f32 * cell_width;
                let gx = x + (cell_width - scale * 6.0) / 2.0;
                let color = COLORS[(if cp850 { row / 2 } else { row }) % COLORS.len()];
                writeln!(
                    self.out,
                    "<g><title>{}: U+{:04X}{}</title>",
                    if cp850 {
                        format!("CP850 {index:02X}")
                    } else {
                        format!("U+{:04X}", ch as u32)
                    },
                    ch as u32,
                    if intentional_blank(ch) {
                        " (intentional blank)"
                    } else if !self.mode.available(ch) {
                        " (unfinished)"
                    } else {
                        ""
                    }
                )
                .unwrap();
                if intentional_blank(ch) {
                    self.empty(gx, top, scale);
                } else if self.mode.available(ch) {
                    self.glyph(gx, top, ch, scale, color);
                } else {
                    self.missing(gx, top, scale);
                }
                let label = if cp850 {
                    format!("{index:02X}")
                } else {
                    format!("{:04X}", ch as u32)
                };
                let label_width = label.len() as f32 * 6.0 * label_scale;
                self.label(
                    x + (cell_width - label_width) / 2.0,
                    top + scale * 11.0 + 7.0,
                    &label,
                    label_scale,
                    DIM,
                );
                self.out.push_str("</g>\n");
            }
        }
        y + rows as f32 * row_height
    }
    fn render(mut self, extras: &[char]) -> String {
        let missing = CP850_CHARACTERS
            .iter()
            .filter(|&&c| !self.mode.available(c))
            .count();
        self.rect(0.0, 0.0, WIDTH as f32, 4000.0, BG);
        self.rect(64.0, 48.0, 36.0, 3.0, COLORS[0]);
        self.label(116.0, 44.0, "MICROFONT / RELEASE SPECIMEN", 1.25, MUTED);
        self.label(64.0, 80.0, self.mode.name(), 1.5, COLORS[0]);
        self.specimen(64.0, 124.0, "MicroFont", 12.0, WHITE);
        self.label(856.0, 148.0, "8.5.0", 4.0, COLORS[0]);
        self.label(858.0, 212.0, "6x11 CELL", 1.4, MUTED);
        let (descriptor, detail) = match self.mode {
            Mode::Hex => (
                "HEX / EXACT PIXEL MASKS",
                "Discrete 6x11 cells. Every visible square is a stored bitmap pixel.",
            ),
            Mode::Raster => (
                "RASTER / LINES + QUADS",
                "Continuous strokes and filled geometry. This image is actual vector SVG.",
            ),
        };
        self.label(64.0, 286.0, descriptor, 1.8, COLORS[0]);
        self.label(64.0, 322.0, detail, 1.0, MUTED);
        self.label(
            64.0,
            343.0,
            "no_std / zero dependencies / same font, two representations",
            1.0,
            MUTED,
        );
        self.missing(64.0, 376.0, 0.8);
        self.label(
            78.0,
            375.0,
            &format!("unfinished ({missing} CP850)"),
            0.9,
            MISSING,
        );
        self.empty(298.0, 376.0, 0.8);
        self.label(312.0, 375.0, "intentional blank", 0.9, MUTED);
        self.label(790.0, 375.0, "LABELS: HEX / NO FONT FILE", 0.8, DIM);

        self.section(
            432.0,
            "01 / CP850",
            "256 display slots / 00-FF / graphical controls included / labels are byte values",
            COLORS[0],
        );
        let mut y = self.grid(496.0, &CP850_CHARACTERS, 16, true);
        self.label(96.0, y + 7.0, "00 = NUL / 20 = SPACE / FF = NBSP. Control-byte APIs retain their documented behavior.", 0.9, MUTED);
        y += 81.0;

        self.section(
            y,
            "02 / BOX DRAWING",
            "U+2500-257F / all 128 / light, heavy, double, dashed, diagonal and rounded",
            COLORS[2],
        );
        let chars: Vec<_> = (0x2500..=0x257f)
            .map(|n| char::from_u32(n).unwrap())
            .collect();
        y = self.grid(y + 64.0, &chars, 32, false) + 56.0;

        self.section(
            y,
            "03 / BLOCK ELEMENTS",
            "U+2580-259F / all 32 / fractions, shades and quadrants",
            COLORS[3],
        );
        let chars: Vec<_> = (0x2580..=0x259f)
            .map(|n| char::from_u32(n).unwrap())
            .collect();
        y = self.grid(y + 64.0, &chars, 32, false) + 56.0;

        self.section(
            y,
            "04 / BRAILLE",
            "U+2800-28FF / all 256 patterns / first cell is intentionally empty",
            COLORS[4],
        );
        let chars: Vec<_> = (0x2800..=0x28ff)
            .map(|n| char::from_u32(n).unwrap())
            .collect();
        y = self.grid(y + 64.0, &chars, 32, false) + 56.0;

        self.section(
            y,
            "05 / ADDITIONAL GLYPHS",
            "Unicode labels / accents, UI symbols, icon cells and shared seven-segment digits",
            COLORS[5],
        );
        y = self.grid(y + 64.0, extras, 16, false) + 30.0;
        self.rule(y);
        self.label(64.0, y + 24.0, "MICROFONT 8.5.0", 1.1, WHITE);
        self.label(
            64.0,
            y + 47.0,
            "Generated from the crate's public glyph APIs. Missing shapes are never substituted.",
            0.9,
            MUTED,
        );
        self.label(64.0, y + 67.0, "SVG geometry only: no embedded font, no external font, no script, no image dependency.", 0.9, DIM);
        let height = (y + 110.0).ceil() as usize;
        format!("<svg xmlns='http://www.w3.org/2000/svg' width='{WIDTH}' height='{height}' viewBox='0 0 {WIDTH} {height}' role='img' aria-labelledby='title desc'>\n<title id='title'>MicroFont 8.5.0 {} atlas</title>\n<desc id='desc'>Actual MicroFont glyph geometry: all 256 CP850 display slots, 128 box-drawing glyphs, 32 block elements, all 256 Braille patterns, and additional glyphs. Orange crosses mark unfinished shapes; outlined cells mark intentional blanks.</desc>\n{}\n</svg>\n", self.mode.name(), self.out)
    }
}

fn additional_glyphs() -> Vec<char> {
    // Host-only discovery via the public lookup API also finds Hex-only glyphs.
    // Unfinished Misc placeholders have no usable geometry and stay excluded.
    (0x20..=0x10ffff)
        .filter_map(|id| {
            let record = bundle::record(id)?;
            let ch = char::from_u32(id)?;
            (record.bitmap().is_some_and(|mask| mask != 0) || Mode::Raster.available(ch))
                .then_some(ch)
        })
        .filter(|ch| {
            !CP850_CHARACTERS.contains(ch)
                && !('\u{2500}'..='\u{259f}').contains(ch)
                && !('\u{2800}'..='\u{28ff}').contains(ch)
        })
        .collect()
}

fn main() -> std::io::Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs"));
    fs::create_dir_all(&output)?;
    let extras = additional_glyphs();
    for mode in [Mode::Hex, Mode::Raster] {
        let path = output.join(mode.filename());
        fs::write(&path, Atlas::new(mode).render(&extras))?;
        println!(
            "{}: {} CP850 slots unfinished; {} additional glyphs",
            path.display(),
            CP850_CHARACTERS
                .iter()
                .filter(|&&c| !mode.available(c))
                .count(),
            extras.len()
        );
    }
    Ok(())
}
