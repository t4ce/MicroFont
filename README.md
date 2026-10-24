# MicroFont

MicroFont is a dependency-free `no_std` 6×11 font with bitmap masks and line/quad
geometry. Glyph data lives in binary files, rather than Rust bitmap tables.

## Features and the single-file bundle

| Features | Included data |
| --- | --- |
| `hex` | Full 66-bit cell masks only |
| `raster` | Line strips, quads and shared component recipes only |
| `hex,raster` | Both representations |
| `stamp` (default) | Enables `hex` and buffer-stamping helpers |

At least one of `hex` or `raster` is required. Neither mode pulls in external
Rust dependencies. Han is not included.

```toml
microfont = { version = "3.9.9", default-features = false, features = ["raster"] }
```

The standard-library-only host build script packs the selected fields into one
uncompressed `microfont.bin`. Portable targets embed only that file. Runtime
lookup needs no allocator, filesystem, tar reader, compression library or
initialization call. The build script uses no external tools for portable builds.

For `cfg(target_os = "trueos")`, the build script instead embeds only one
`microfont.tar.lz4` archive. The tar has exactly one member, `microfont.bin`;
LZ4 packaging uses the host `lz4` executable. TRUEOS decompresses it after heap
setup, validates it with `install_bundle`, and retains the resulting bytes for
the kernel lifetime before the first font output. The uncompressed file is a
build intermediate, not a second embedded font. A second installation is rejected.

From the TRUEOS repository, produce a distributable archive with:

```sh
make microfont-bundle MICROFONT_FEATURES=hex,raster
# bld/microfont/microfont.tar.lz4
```

Use `MICROFONT_FEATURES=hex` or `raster` to produce either representation alone.
For a portable single file without compression:

```sh
python3 tools/microfont-pack/build.py --features raster --output /tmp/microfont.bin
```

## Editable source records

The existing `data/line-v1/` directory now holds version-2, mask-first records.
Its historical directory name is retained so glyph editing paths stay stable.
Each Unicode glyph is `uXXXX.bin` (uppercase hex, at least four digits). Private
components have descriptive filenames in their own subdirectories.

| Offset | Field |
| --- | --- |
| 0–15 | Little-endian `u128` bitmap; top-left pixel is bit 65, bottom-right bit 0 |
| 16 | Flags: bit 0 means a bitmap is present; all other bits are zero |
| 17 | Raster kind (below) |
| 18–19 | Reserved, zero |
| 20 onward | Raster payload, bounded by the file length |

A zero-valued bitmap can be a valid blank glyph. A clear presence flag means
that this record has no bitmap representation. Bitmap and raster fields are
independent; editing a bitmap does not change line or quad geometry.

| Kind | Payload |
| --- | --- |
| 0 | No raster data |
| 1 | Existing v1 line-strip records |
| 2 | Quads: four perimeter-ordered `[f32; 2]` corners, little endian, 32 bytes per quad |
| 3 | Shared alias: target ID `u32 LE`, then transform byte |
| 4 | Lowercase composition: base ID `u32 LE`, topper ID `u32 LE` (0 for none), omit-dot byte |
| 5 | Ordered list of shared line-component IDs, each `u32 LE` |
| 6 | Rounded box corner: orientation byte, then junction x/y, radius and stroke width as four `f32 LE` values |

Alias transforms are 1=mirror X, 2=mirror Y, 3=rotate 90°, 4=rotate 180°,
5=rotate 270°. Six private [lowercase toppers](data/line-v1/lowercase-toppers/README.md)
use IDs U+110000–U+110005. Seven-segment components use IDs 0x110100–0x110106.
These IDs are internal records, not Unicode characters or ordinary glyphs.

`homebutton.bin` is the line-only private start-button asset, ID `0x110200`.
Its 20-byte source header has a zero bitmap and presence flag zero, raster kind
1, then the original raw strip bytes. `raster_line::home_button_strips()` is its
named reader: it has no Unicode alias and does not enter either font atlas.
TRUEOS raster builds require this file and UI4 renders it independently once
at startup, with no replacement-glyph or section-sign fallback. Hex-only
bundles omit the asset.

A line-strip record starts with a header byte: bits 0–5 are point count minus
two, bit 6 marks an End at the start, and bit 7 marks an End at the finish.
Otherwise the boundary is a Junction. Each point occupies one byte: x in bits
0–2 (0–5), y in bits 3–6 (0–10); bit 7 is reserved. Two coincident points remain
a degenerate segment. Strokes retain their own boundary tags.

The build script validates selected records and their references. It discovers
new source records automatically; no Rust geometry generator is needed.

Rounded `╭ ╮ ╯ ╰` tiles each use two straight quads and a quarter-circle made
from 4–16 quads. The compact recipe uses orientation 0/1/2/3 respectively;
the initial junction is `(2.5,5.5)`, radius 2 and stroke width 1 source unit,
matching the adjoining `─` and `│` tiles. Bitmap fields remain independent.
`glyph_quads_at_size(character, em_height_px, em_10mm_px)` selects a budget:
the physical-size floor is four curve quads at 10mm, eight at 20mm and sixteen
at 40mm, with a sixteen-quad cap. A quarter-pixel outer-arc chord-error target
can increase the count on dense displays. If physical metrics are unavailable,
the same pixel-error target chooses the count with a minimum of four. The
default `glyph_quads` view uses four curve quads. All four orientations use the
same allocation-free recipe; no trig library or stored tessellation is needed.

## Runtime API

`glyph_cell_pixels(char)` returns the full bitmap; `glyph_pixels(char)` and
`font_pixels(u8)` retain the legacy 64-bit view. Bitmap APIs require `hex`.
Byte-to-character identities are unchanged, including the graphical CP850
control slots in `CP850_CHARACTERS`. Identity-only constants replace the old
bitmap-bearing `*_SYMBOLS` and `FPIXELS` tables.

With `stamp`, `stamp_text`, `stamp_bytes` and the ARGB variants write to caller
buffers. Controls and space can expand into named labels; keyboard/map icons
use private cells. `control_label_cells` returns a small owned mask view.
Unsupported Unicode uses the existing CP850 replacement cell.

With `raster`, `raster_line::glyph_strips` returns a borrowed `GlyphStrips` view.
Its iterator/get methods yield `Strip` values referencing the shared point
bytes. `raster_quad::glyph_quads` and the box/block/Braille helpers return quad
views with value iterators. `raster_quad::GLYPH_CHARACTERS` lists stored quad
glyphs, including `▬` and `■` outside the box/block ranges.
Raster-only builds contain no bitmap fields and
never reconstruct quads from hidden embedded masks.

The flat bundle layout and validation rules are documented in
[data/line-v1/README.md](data/line-v1/README.md).
