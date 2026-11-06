//! Write scalable font geometry: cargo run --example svg_text --no-default-features --features raster
use std::{
    fmt,
    io::{self, Write},
};

/// Bridge the no_std formatter API to an ordinary streaming I/O sink.
struct Sink<W>(W);
impl<W: Write> fmt::Write for Sink<W> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0.write_all(value.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = Sink(stdout.lock());
    out.0.write_all(b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"-1 -1 188 13\" width=\"1128\" height=\"78\" color=\"#102030\">\n")?;
    microfont::svg::write_text(
        &mut out,
        "MicroFont: lines + quads \u{2588}\u{2591}\u{2801}",
        1.0,
    )
    .map_err(|error| io::Error::other(format!("SVG output: {error:?}")))?;
    out.0.write_all(b"\n</svg>\n")
}
