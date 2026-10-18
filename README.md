# MicroFont

MicroFont is a tiny `no_std` 6x11 bitmap font stamper for flat pixel buffers.

![MicroFont 3.9.9: color CP850 atlas, Braille, block elements, and ARGB styles](https://raw.githubusercontent.com/t4ce/MicroFont/true/docs/font-atlas.png)

It exposes the packed font table, direct byte/text stamping helpers, and ARGB helpers for alpha, underline, strikeout, and vertical flip styling. The library has no dependencies.

The default `stamp` feature enables all buffer stamping helpers. Use
`default-features = false` for pixel data and lookup APIs alone. The independent
`raster` feature adds `raster::BRAILLE_QUADS`, `braille_quad_slice` and
`braille_quads`; it does not require `stamp`. The independent, non-default
`han` feature includes the Chinese GUI subset, its tables and accessors.
Without `han`, those tables and lookup code are excluded and unsupported Han
characters use the usual `?` fallback. For example:

```toml
microfont = { version = "3.9.9", default-features = false, features = ["raster"] }
```

Braille pixels are always generated at compile time. With `raster`, the same pass
also prepares every glyph's quads in 6×11 source coordinates. Access borrows
static data; no runtime warm-up or allocation is required. This precomputation
does not guarantee that the hardware CPU cache contains the data.

`BASIC_SYMBOLS` owns the existing single-cell Basic Latin masks in codepoint
order. `CONTROL_SYMBOLS` holds ready-made two- or three-cell representations
for U+0000–U+0020 and U+007F (`NUL`, `BS`, `SP`, `DEL`, etc.), assembled at compile time from the
existing capital-letter and digit masks. `special_cells`, `measure_text` and
Unicode stamping expand them just like the keyboard/map icons. Space displays
`SP` in two cells. `control_label_cells` exposes their precomputed mask slices.
Single-cell/raw byte masks retain their existing layout; Unicode DEL expands
to the three-cell `DEL` label.
Basic Latin has no quad raster data yet.
`FontShape::INFLUENCE_WIDTH` is fixed at 0.20 source units for both shapes and
both Dista/Convo families. `FontShape::body_threshold()` returns 0.24 for Meso
and 0.70 for Lean. Per-glyph seed/reach overrides are independent of these settings.
`FontDetail` selects triangle tessellation independently of family and shape:
`Fine = 0` (the default) uses 12 subdivisions per source-cell unit, and `High = 1`
uses 24, matching the HTML presets. `FontDetail::from_level` accepts a 0/1 setting;
`subdivisions_per_cell()` exposes the geometry resolution for the future triangle
builder. The existing pixel and quad data are unaffected.


With `raster`, `basic_seed` selects the common reconstruction input before any
Dista/Convo or Meso/Lean choice. `BASIC_RASTER_OVERRIDES` currently contains only
`/`: five collinear sites instead of the bitmap's nine staircase sites, tagged
`long_reach`. Its full-cell mask is `0x2004008010020000`. Short reach is 1.76;
long reach is 2.76. All other Basic Latin seeds use their existing masks and
short reach. This is seed data for future triangle reconstruction, not quad
geometry, and the original bitmap masks remain unchanged.

`BLOCK_SYMBOLS` lists all 32 Block Elements as visible characters beside embedded
hex masks. With `raster`, `BLOCK_QUADS`, `block_quad_slice` and `block_quads`
provide their exact 6×11 rectangles, derived at compile time from those masks
by one shared cover algorithm. All 32 characters share a packed array of 104
quads and 33 offsets (482 bytes total); no per-glyph padding or shade-specific
lookup is needed. Pixel masks remain unchanged.

`BOX_DRAW_SYMBOLS` is a complete 128-character draft table for Box
Drawing glyphs in a 6×11 cell. It draws on 37 existing masks and drafts the
missing 84 variants. Double vertical rails use columns 1 and 4, with matching
corner/junction connections. Heavy vertical strokes are two columns wide
(columns 2 and 3); heavy horizontal strokes remain three rows high. Only curves `╭╮╯╰` remain pixel-only; their raster lookup returns `None`.
With `raster`, `BOX_DRAW_QUADS` and `boxdraw_quad_slice` provide 346 four-corner
quads for the other 124 characters. The 121 straight glyphs use the same exact
rectangle cover as blocks. Diagonals use continuous slanted geometry: one quad
for `╱`, one for `╲`, and three non-overlapping quads for their union `╳`.
`BoxDrawQuad::vertices` contains four perimeter-ordered `[f32; 2]` corners in
6×11 source units, ready to split into two triangles. Raw diagonal hex masks
retain their staircase pixels. The four rounded corners are the only masks
without raster data. Four separated horizontal dashes cannot fit in six
columns, so those drafts are approximations.
`glyph_cell_pixels` and Unicode stamping use these canonical masks directly.
`CP850_SYMBOLS` in `src/cp850.rs` lists all 256 display-code slots as visible
characters with full 6×11 masks, including the graphical control slots and
`⌂` at 0x7F. Missing masks are zero. Shared Basic Latin, box drawing, block
and UI masks are referenced at compile time; the remaining 100 extended
masks are owned here. The low-slot `¶` and `§` reuse their extended-slot masks.
The legacy 224-slot `FPIXELS` view and byte lookup helpers are derived here
as well, preserving their existing indexing and control-byte behavior.
The remaining UI symbol masks live directly in `lib.rs`.


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
pixels. Byte/ARGB slot identities remain unchanged, with their box and block
masks now derived from the canonical tables.

With the `han` feature, Unicode text includes **MicroFont GUI Han**, a curated 512-character Chinese GUI
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
retain their existing mappings. The glyph masks are in the `UI_SYMBOLS` table in `src/lib.rs`.
