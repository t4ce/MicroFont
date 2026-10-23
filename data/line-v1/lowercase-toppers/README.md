# Lowercase toppers

Six private line-bin components, independent of the regular Unicode glyphs
``¨ ´ ` ^ ° ~``. They use the same mask-first record envelope as the parent directory; their bitmap flag is clear and kind 1 carries the line strips.
Edit these files to tune one topper across all its lowercase compositions:

| File | Topper | Unicode combining identity (reference only) |
| --- | --- | --- |
| `diaeresis.bin` | ¨ | U+0308 |
| `acute.bin` | ´ | U+0301 |
| `grave.bin` | \` | U+0300 |
| `circumflex.bin` | ^ | U+0302 |
| `ring.bin` | ° | U+030A RING ABOVE |
| `tilde.bin` | ~ | U+0303 |

Coordinates are already placed in the 6×11 cell, above the lowercase body
(rows 0–2 in the initial assets; bodies start at row 4). No translation or
resizing is applied. The initial shapes come from the authored lowercase bins,
including the latest `ä` dots, rather than the standalone punctuation glyphs.

The bundle maps these 36 precomposed Unicode letters:

| Base | ¨ | ´ | \` | ^ | ° | ~ |
| --- | --- | --- | --- | --- | --- | --- |
| a | ä | á | à | â | å | ã |
| e | ë | é | è | ê | | ẽ |
| i | ï | í | ì | î | | ĩ |
| o | ö | ó | ò | ô | | õ |
| u | ü | ú | ù | û | ů | ũ |
| y | ÿ | ý | ỳ | ŷ | ẙ | ỹ |
| n | | ń | ǹ | | | ñ |

This includes all 26 CP850 letters in this category. In particular, CP850
includes `ã`. The other 10 are Unicode additions for the same bases and toppers.
An empty cell means Unicode has no single precomposed scalar for that pair;
it can still be written as a combining sequence. All 42 pairs are available
through the explicit `lowercase_topped_strips(base, LowercaseTopper::...)` API,
including `e̊ i̊ o̊ n̈ n̂ n̊`. This API returns a borrowed glyph view, without
inventing character codes. Automatic text-sequence shaping and stacked
accents are outside this single-character lookup. "ANSI" is not one encoding;
Windows-1252 adds no further letters in this particular grid beyond CP850.

Sources: [CP850 mapping](https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP850.TXT)
and [Unicode canonical decompositions](https://www.unicode.org/Public/UCD/latest/ucd/UnicodeData.txt).

The base strokes are borrowed from the existing lowercase line glyphs, including
the transformed `u` alias. Composed `i` omits its ordinary dot strokes (whole
strips above row 4), preserving its body and foot. Uppercase and all hex bitmap
tables remain unchanged. The components do not register standalone glyphs.

`GlyphStrips` is a borrowed view of the base and topper: point bytes and strip
arrays are shared, with no combined bin or per-letter combined strip array.
Only the character-to-component mapping is added per letter. The six topper payloads total 27 bytes; each source file also has its 20-byte header. Rendered
atlas cells and GPU draws still exist for each supported character.

Cargo packages and validates these source records automatically. Each composed character has one source record containing its independent bitmap field and a reference recipe. A missing referenced base or topper is rejected; recipes cannot silently fall back to duplicated strokes.
