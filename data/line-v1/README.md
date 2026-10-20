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
