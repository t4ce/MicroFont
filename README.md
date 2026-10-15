# MicroFont

MicroFont is a tiny `no_std` 6x11 bitmap font stamper for flat pixel buffers.

![MicroFont 3.9.9: color CP850 atlas, Braille, block elements, and ARGB styles](https://raw.githubusercontent.com/t4ce/MicroFont/true/docs/font-atlas.png)

It exposes the packed font table, direct byte/text stamping helpers, and ARGB helpers for alpha, underline, strikeout, and vertical flip styling. The library has no dependencies.

Text helpers accept Unicode and select the existing CP850 atlas glyphs, including
`§`, accented Latin letters and box drawing characters. Ordinary characters occupy
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

Unicode text includes **MicroFont GUI Han**, a curated 512-character Chinese GUI
subset: 308 Han characters for menus, file actions, settings and error messages,
19 punctuation marks, and 185 common-character fillers. Each occupies exactly
one 6×11 cell, preserving the source pixels. Its packed lookup and glyph tables
use 3,627 bytes; `GUI_HAN_COUNT` and `GUI_HAN_TABLE_BYTES` expose those budgets.
Coverage is tailored to controlled GUI wording; arbitrary names can require
characters outside the set. See the [atlas](docs/gui-han-atlas.png) and
[source, license and regeneration notes](NOTICE-cjk.md).

The keyboard `⌨` and map `🗺` icons in `SPECIAL` use two and three cells
respectively. `measure_text` and Unicode stamping account for these widths;
`special_cells` exposes their internal half-cell characters to fixed-grid
terminal renderers.

Install with `cargo add microfont@3.9.9`.

```rust
let mut pixels = vec![0u32; 88 * 27];
microfont::stamp_text(&mut pixels, 88, 27, 8, 8, "Hello World!", 0xF7E27E)?;
```

The example writes an uncompressed BMP:

```sh
cargo run --example hello_world_bmp
```

Tests also generate visual atlas BMPs in `target/` for manual inspection.

Regenerate the README atlas with `cargo run --example readme_previews`; convert
`target/font-atlas.ppm` to `docs/font-atlas.png` at 2× scale with nearest-neighbor sampling.

Unicode text also includes 55 TermDir UI symbols: border joins, rounded corners,
arrows, menu markers, segmented digits, and file/action icons. Each occupies one
6×11 cell, including the fullwidth code points and emoji; raw byte atlas APIs
retain their existing mappings. The glyph masks are in `src/ui_symbols.rs`.
