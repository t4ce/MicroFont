//! Generate README previews: cargo run --example readme_previews
//! Convert target/*.ppm to PNG with an image converter, using nearest-neighbor scaling.
use microfont::{stamp_bytes, stamp_text};
use std::{fs, io::Write};
const W: usize = 320;
const H: usize = 440;
fn text(p: &mut [u32], x: i32, y: i32, s: &str, c: u32) {
    stamp_text(p, W, H, x, y, s, c).unwrap();
}
fn main() -> std::io::Result<()> {
    fs::create_dir_all("target")?;
    let mut p = vec![0x101820; W * H];
    text(&mut p, 12, 12, "MicroFont 3.9.9", 0xF7E27E);
    text(
        &mut p,
        12,
        30,
        "6x11 pixels / no_std / zero dependencies",
        0xA7BBCB,
    );
    text(
        &mut p,
        12,
        60,
        "CP850 ATLAS / RAW BYTES 0x20-0xFF",
        0xF7E27E,
    );
    for i in 0..224 {
        stamp_bytes(
            &mut p,
            W,
            H,
            12 + (i % 32) * 9,
            80 + (i / 32) * 16,
            &[(i + 32) as u8],
            0xE4EDF4,
        )
        .unwrap();
    }
    text(
        &mut p,
        12,
        204,
        "BRAILLE / U+2800-U+28FF / ALL 256",
        0x62D6BD,
    );
    for i in 0..256 {
        text(
            &mut p,
            12 + (i % 32) * 9,
            224 + (i / 32) * 16,
            &char::from_u32(0x2800 + i as u32).unwrap().to_string(),
            0x62D6BD,
        );
    }
    text(
        &mut p,
        12,
        368,
        "BLOCK ELEMENTS / U+2580-U+259F / ALL 32",
        0x89BCFF,
    );
    for i in 0..32 {
        text(
            &mut p,
            12 + i * 9,
            388,
            &char::from_u32(0x2580 + i as u32).unwrap().to_string(),
            0x89BCFF,
        );
    }
    text(
        &mut p,
        12,
        416,
        "Unicode: café naïve Straße § ┌─┐ █▌░",
        0xE4EDF4,
    );
    let mut f = fs::File::create("target/font-atlas.ppm")?;
    write!(f, "P6\n{W} {H}\n255\n")?;
    for c in p {
        f.write_all(&[(c >> 16) as u8, (c >> 8) as u8, c as u8])?;
    }
    Ok(())
}
