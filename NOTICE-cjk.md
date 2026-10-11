# MicroFont GUI Han subset

The generated glyph data in `src/gui_han_data.rs` and its atlas preview in
`docs/gui-han-atlas.png` are derived from **XiuZhenXiangSuTi** version 1.0.
They are licensed under the SIL Open Font License 1.1, reproduced in
[OFL-cjk.txt](OFL-cjk.txt). The decoder and generator code retain MicroFont's
MIT license. The subset is named **MicroFont GUI Han**; the upstream reserved
font names are not used as the name of the derived font.

Copyright (c) 2026 yzdnn (https://github.com/yzdnn)

Copyright (c) 2000–2023 Num Kadoma

Upstream: https://github.com/yzdnn/XiuZhenXiangSuTi

Pinned revision: `41fc94c12ff63913bc1913ef9de1b290957594bb`

Source: `Sources/XiuZhenXiangSuTi-8.bdf`

Source SHA-256: `6147632b94cc9e5ba9cc2adb3e983611302624daeecc64db292a35d42965ca38`

The subset has exactly 512 mappings: 308 Han characters from the authored GUI
vocabulary, 19 Chinese punctuation marks, and 185 common-character fillers.
Fillers follow Jun Da's Modern Chinese character frequency list dated
2004-03-30, after excluding already-required GUI characters:
https://lingua.mtsu.edu/chinese-computing/statistics/char/list.php?Which=MO

The coverage inventory and filler selection are in `tools/gui-zh-hans.txt` and
`tools/gui-zh-hans-fill.txt`. They describe character coverage; they do not
install Chinese translations, input methods, or a locale.

All selected source pixels are preserved. The BDF baseline is positioned at
row 9 of a 6×11 cell. Every selected drawing fits in a 5×7 area starting at
row 2, leaving an empty sixth column and vertical padding. No character is
scaled, clipped, or expanded into a special multi-cell glyph. The source's
out-of-bounds pixel in unselected U+91D5 does not enter this subset.

The tables occupy 3,627 bytes: 1,024 bytes of sorted BMP codepoints, 64 bytes
of record checkpoints, and 2,539 bytes of glyph records. Each record is a
packed literal or a reference to an earlier literal plus zero, one, or two
pixel toggles. Sixteen selected glyphs use patches; 496 are literals. Sharing
is based on exact pixels, with no assumptions about interchangeable radicals.
The renderer decodes one glyph at a time, without allocations or a full
decompressed atlas. Unsupported characters retain MicroFont's `?` fallback.

To reproduce the tables and atlas, download the pinned source into a temporary
directory, then run from the MicroFont directory:

```sh
python3 tools/mint_gui_han.py /tmp/xiuzhen-8.bdf \
    --preview docs/gui-han-atlas.png
python3 tools/mint_gui_han.py /tmp/xiuzhen-8.bdf --check
```

The generator rejects missing characters, source-hash mismatches, out-of-cell
pixels, incorrect counts, and any pixel changes during compression round trips.
Use `--oracle /tmp/gui-han-oracle.bin` to export original BDF masks for independent
host-side validation of the Rust decoder.
