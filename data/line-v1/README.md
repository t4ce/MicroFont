# Mask-first glyph records and the flat font image

Source `.bin` files have a 20-byte header: a 16-byte little-endian bitmap value,
a presence flag byte, a raster-kind byte, and two reserved zero bytes. Payload
starts at offset 20. See the [crate README](../../README.md) for every source
field and raster recipe. The old raw strip-only file format is intentionally
replaced; strip encoding itself is unchanged.

`homebutton.bin` is a named exception to Unicode filenames: private ID
`0x110200`, bitmap zero with presence flag zero, kind 1. It is used exclusively
by the UI4 start button, outside the regular atlases. TRUEOS raster builds fail
if the file is absent; malformed or non-line home-button records are rejected.
Its raw exports can use the same `import-lines.py` workflow as Unicode bins.

Kind 6 contains a 17-byte rounded-corner recipe: orientation byte
(0=╭, 1=╮, 2=╯, 3=╰), followed by junction x/y, centreline radius and stroke
width as four little-endian `f32` values. Its straight arms join cell edges;
the quarter-circle has an adaptive 4–16 quad budget. Invalid or out-of-cell
recipes are rejected even when only bitmap fields are selected.

## Importing raw line exports

Keep exporting the same raw line-strip bytes from your glyph editor. From the
TRUEOS root, import an export before replacing the canonical source record:

```sh
python3 tools/microfont-pack/import-lines.py /tmp/u2194.bin
```

The importer preserves the existing glyph's bitmap value and presence flag,
sets raster kind 1, and replaces only its line payload. A new glyph starts with
no bitmap. Use `--glyph U+2194` if the export has another filename.

If an export has already overwritten its canonical `uXXXX.bin`, explicitly
recover the last committed bitmap header while retaining the exported strokes:

```sh
python3 tools/microfont-pack/import-lines.py --recover-header-from-git \
  vendor/microfont/data/line-v1/u2194.bin
```

Multiple exports can be passed together. Every input is validated before any
target is written. The ordinary build remains strict so a damaged bitmap header
cannot silently become a raw-line glyph.

## Packed image

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
