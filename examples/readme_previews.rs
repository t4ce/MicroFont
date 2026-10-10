//! Generate README previews: cargo run --example readme_previews
//! Convert target/*.ppm to PNG with an image converter, using nearest-neighbor scaling.
use microfont::{FLIPY, STRIKE, ULINE, stamp_argb, stamp_bytes, stamp_text};
use std::{fs, io::Write};
const W: usize = 320;
const H: usize = 570;
const COLORS: [u32; 7] = [
    0xFF3040, 0xFF9D2E, 0xFFE83A, 0x2FE66B, 0x2BC7FF, 0x635BFF, 0xD93DFF,
];
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
            COLORS[(i as usize / 10) % COLORS.len()],
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
            COLORS[(i as usize / 32) % COLORS.len()],
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
            COLORS[(i as usize / 4) % COLORS.len()],
        );
    }
    text(
        &mut p,
        12,
        416,
        "Unicode: café naïve Straße § ┌─┐ █▌░",
        0xE4EDF4,
    );
    text(&mut p, 12, 452, "ARGB STYLES / SAME GLYPHS", 0xF7E27E);
    for (row, (label, alpha, style)) in [
        ("Plain", 0xFF, 0),
        ("Alpha 50%", 0x80, 0),
        ("Underline", 0xFF, ULINE),
        ("Strikeout", 0xFF, STRIKE),
        ("Flip Y", 0xFF, FLIPY),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 474 + row as i32 * 17;
        text(&mut p, 12, y, label, 0xA7BBCB);
        for (i, byte) in b"MicroFont 0123456789 !?".iter().enumerate() {
            stamp_argb(
                &mut p,
                W,
                H,
                100 + i as i32 * 6,
                y,
                &[*byte],
                (alpha << 24) | COLORS[(i / 3) % COLORS.len()],
                style,
            )
            .unwrap();
        }
    }
    let mut f = fs::File::create("target/font-atlas.ppm")?;
    write!(f, "P6\n{W} {H}\n255\n")?;
    for c in p {
        // Composite ARGB samples over the preview background for the RGB export.
        let alpha = if c >> 24 == 0 { 255 } else { c >> 24 };
        let blend = |shift| {
            let fg = (c >> shift) & 255;
            let bg = (0x101820u32 >> shift) & 255;
            ((fg * alpha + bg * (255 - alpha) + 127) / 255) as u8
        };
        f.write_all(&[blend(16), blend(8), blend(0)])?;
    }
    Ok(())
}
