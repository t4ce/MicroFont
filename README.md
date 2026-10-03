# MicroFont

MicroFont is a tiny `no_std` 6x11 bitmap font stamper for flat pixel buffers.

![MicroFont glyph atlas preview](https://raw.githubusercontent.com/t4ce/MicroFont/true/docs/font-atlas.png)

It exposes the packed font table, direct byte/text stamping helpers, and ARGB helpers for alpha, underline, strikeout, and vertical flip styling. The library has no dependencies.

Text helpers accept Unicode and select the existing CP850 atlas glyphs, including
`§`, accented Latin letters and box drawing characters. Each character occupies
one cell; unsupported characters display `?`. Byte and ARGB helpers continue to
accept raw atlas codes. `glyph_byte` exposes the same lookup to other renderers.

Unicode text also supports all 256 eight-dot Braille patterns (U+2800–U+28FF)
and all 32 Block Elements (U+2580–U+259F), generated at compile time. Block
fractions round to the nearest pixel in the default 6×11 cell; some horizontal
eighth steps share a bitmap because the cell is only six pixels wide. Full
blocks fill all 66 pixels, so adjacent cells join without gaps.
`glyph_cell_pixels` exposes the full cell as a `u128` with the top-left pixel
at bit 65. The legacy `glyph_pixels` mask uses bit 63 and omits the final two
pixels. Raw byte/ARGB atlas rendering remains unchanged.

```rust
let mut pixels = vec![0u32; 88 * 27];
microfont::stamp_text(&mut pixels, 88, 27, 8, 8, "Hello World!", 0xF7E27E)?;
```

The example writes an uncompressed BMP:

```sh
cargo run --example hello_world_bmp
```

Tests also generate visual atlas BMPs in `target/` for manual inspection.
