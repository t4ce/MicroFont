# Line glyph binary v1

A glyph is a bounded byte slice containing consecutive strip records. The grid
is fixed at 6 columns × 11 rows. No text encoding, padding, delimiter, file header,
or adjacency vertices are stored. Version and glyph identity come from the
containing directory/lookup, rather than being repeated in every glyph.

Each record starts with one control byte:

| Bits | Meaning |
| --- | --- |
| 0–5 | Number of points minus 2 (2–65 points) |
| 6 | Start boundary: 1 = End, 0 = Junction |
| 7 | End boundary: 1 = End, 0 = Junction |

Then follow that many one-byte points:

| Bits | Meaning |
| --- | --- |
| 0–2 | x, 0–5 |
| 3–6 | y, 0–10 |
| 7 | Reserved, must be zero |

A new record starts immediately after those points. End-of-slice ends the glyph.
There is no byte order dependency because every field fits in one byte.

## Adding glyphs to the demo

Place a file named `A.bin`–`Z.bin`, `0.bin`–`8.bin`, or `a.bin`–`z.bin` (except `d.bin`, `q.bin` and `u.bin`), in this directory, then run `make iso`
from TRUEOS. The build runs `tools/generate_line_glyphs.py` before compiling the
kernel, indexing every present letter and digit. New files, replaced contents,
changed strip counts and removed files need no manual Rust registration.
Invalid data stops the build with the filename and byte offset.

The UI4 preview reserves five rows: A–M, N–Z, 0–9, a–m, then n–z. Missing files leave
empty cells. Files are embedded at build time; adding a file does not change a
running image. Each glyph stays at 96×176 pixels, with scroll-controlled width.

For example, a two-point End/End strip from (0,0) to (0,8) is `C0 00 40`: two
coordinate bytes plus one control byte. Multiple records concatenate directly.

`raster_line::decode_glyph` validates these at compile time and exposes
point slices directly into `include_bytes!` data. The existing renderer consumes
those views; it generates adjacency vertices during upload. Rust strip views
contain pointers/lengths in memory, but these are not part of the file format.
Invalid coordinates, reserved point bits, truncation, missing records and
trailing bytes cause validation to fail. The loader infers the strip count from the record headers at compile time;
no strip-count byte is stored.

## Shared flipped glyphs

`9` uses `6.bin`, rotated 180 degrees. `d` uses `b.bin`, mirrored left-to-right. `q` likewise mirrors `p.bin`. `u` uses `n.bin`, rotated 180 degrees.
Do not store `9.bin`, `d.bin`, `q.bin` or `u.bin`: these names are aliases, so separate files with
those names are not embedded. If the canonical source is missing, the alias is
missing too. Changes to the source automatically affect its alias on rebuild.

Reflection uses the complete source glyph's occupied coordinate bounds, shared
across all of its strips. For the current 6 and b, these are x=0–4 and y=0–8:
9 maps (x,y) to (4−x,8−y), while d maps (x,y) to (4−x,y). The q alias uses p’s own bounds, preserving its descender.
The original point slices and boundary tags are shared. No transformed point
array or second binary is stored; Strip::point/iter_points and adjacency_vertex
apply the transform when consumed. The public points slice is canonical data.
