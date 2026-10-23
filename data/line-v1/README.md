# Mask-first glyph records and the flat font image

Source `.bin` files have a 20-byte header: a 16-byte little-endian bitmap value,
a presence flag byte, a raster-kind byte, and two reserved zero bytes. Payload
starts at offset 20. See the [crate README](../../README.md) for every source
field and raster recipe. The old raw strip-only file format is intentionally
replaced; strip encoding itself is unchanged.

`build.rs` selects bitmap fields, raster fields, or both. It sorts IDs, writes
one index followed by selected payloads, and validates the entire result.
Absent representations occupy no payload bytes. Unknown versions, empty
feature selections, overlapping/truncated payloads, invalid coordinates,
missing references and non-canonical reference chains are rejected.

The uncompressed bundle is explicitly little endian and independent of host
pointer size, alignment and Rust struct layout:

| Header offset | Width | Field |
| --- | --- | --- |
| 0 | 8 | `MFBNDL2` followed by NUL |
| 8 | 2 | Version, 2 |
| 10 | 2 | Selected features: hex=1, raster=2 |
| 12 | 4 | Entry count |
| 16 | 4 | Index start, 24 |
| 20 | 4 | Total file length |

Each sorted index entry is 24 bytes:

| Entry offset | Width | Field |
| --- | --- | --- |
| 0 | 4 | Unicode scalar or private component ID |
| 4 | 4 | Bitmap payload offset; zero if absent |
| 8 | 4 | Raster payload offset; zero if absent |
| 12 | 4 | Raster payload byte length |
| 16 | 1 | Raster kind, as in source records |
| 17 | 1 | Bitmap-presence flag |
| 18 | 6 | Reserved, zero |

Payloads follow entry order, bitmap first (16 bytes when present), then raster.
No extra padding or trailing bytes are permitted. Canonical line sources are
shared by aliases, toppers and seven-segment digits. Recipes cannot form cycles.

The portable reader validates without allocating and borrows immutable bytes.
TRUEOS wraps exactly this selected file in a deterministic ustar/LZ4 archive;
it supplies decompression and installation before font consumers start.
