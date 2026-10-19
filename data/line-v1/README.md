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

The checked-in files preserve the supplied A and E coordinates and tags:

- `A.bin`: `C3 40 10 02 14 44 00 20 24` (7 points + 2 headers = **9 bytes**).
- `E.bin`: `C2 04 00 40 44 80 20 23` (6 points + 2 headers = **8 bytes**).

`raster_line::decode_glyph::<2>` validates these at compile time and exposes
point slices directly into `include_bytes!` data. The existing renderer consumes
those views; it generates adjacency vertices during upload. Rust strip views
contain pointers/lengths in memory, but these are not part of the file format.
Invalid coordinates, reserved point bits, truncation, missing records and
trailing bytes cause validation to fail. The caller supplies the expected strip
count; no strip-count byte is stored.
