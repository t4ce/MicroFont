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

Place a file named `A.bin` through `Z.bin` in this directory, then run `make iso`
from TRUEOS. The build runs `tools/generate_line_glyphs.py` before compiling the
kernel, indexing every present uppercase letter. New files, replaced contents,
changed strip counts and removed files need no manual Rust registration.
Invalid data stops the build with the filename and byte offset.

The UI4 preview reserves 26 cells in two rows: A–M, then N–Z. Missing files leave
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
