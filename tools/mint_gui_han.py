#!/usr/bin/env python3
"""Mint the 512-character, single-cell GUI subset from a pinned upstream BDF.

No resampling or clipping: every source pixel must fit the 6x11 cell. The
upstream baseline is placed at row 9, aligning 7-pixel Han with Latin capitals.
This script needs only Python's standard library; --preview additionally uses
Pillow. Generated glyph data is OFL-1.1; the generator itself is MIT licensed.
"""

import argparse
import hashlib
import json
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REVISION = "41fc94c12ff63913bc1913ef9de1b290957594bb"
SOURCE_SHA256 = "6147632b94cc9e5ba9cc2adb3e983611302624daeecc64db292a35d42965ca38"
SOURCE_URL = (
    f"https://raw.githubusercontent.com/yzdnn/XiuZhenXiangSuTi/{REVISION}/"
    "Sources/XiuZhenXiangSuTi-8.bdf"
)
COUNT = 512
STRIDE = 16


def inventory(path):
    return "\n".join(
        line for line in path.read_text().splitlines() if not line.startswith("#")
    )


def selection():
    required = {ch for ch in inventory(ROOT / "tools/gui-zh-hans.txt")
                if ord(ch) > 127 and not ch.isspace()}
    fill = inventory(ROOT / "tools/gui-zh-hans-fill.txt").split()
    selected = required | set("".join(fill))
    if len(selected) != COUNT:
        raise ValueError(f"expected {COUNT} selected characters, got {len(selected)}")
    if any(ord(ch) > 0xffff for ch in selected):
        raise ValueError("the compact GUI lookup requires BMP codepoints")
    return sorted(map(ord, selected)), required


def read_masks(path, codes):
    source = path.read_bytes()
    if hashlib.sha256(source).hexdigest() != SOURCE_SHA256:
        raise ValueError("BDF differs from pinned source; audit changes before minting")
    masks = {}
    for block in source.decode().split("STARTCHAR ")[1:]:
        lines = block.splitlines()
        cp = int(next(line.split()[1] for line in lines if line.startswith("ENCODING ")))
        if cp not in codes:
            continue
        w, h, ox, oy = map(int, next(line[4:] for line in lines
                                    if line.startswith("BBX ")).split())
        rows = lines[lines.index("BITMAP") + 1:lines.index("ENDCHAR")]
        if len(rows) != h:
            raise ValueError(f"wrong bitmap height for U+{cp:04X}")
        mask = 0
        for row, value in enumerate(rows):
            bits = int(value, 16)
            padded_width = (w + 7) // 8 * 8
            for column in range(w):
                if not bits & (1 << (padded_width - 1 - column)):
                    continue
                x, y = ox + column, 9 - (oy + h) + row
                if not (0 <= x < 6 and 0 <= y < 11):
                    raise ValueError(f"U+{cp:04X}: pixel ({x},{y}) exceeds 6x11")
                mask |= 1 << (65 - y * 6 - x)
        masks[cp] = mask
    if set(masks) != set(codes):
        raise ValueError(f"missing source glyphs: {set(codes) - set(masks)}")
    return masks


def narrow_literal(mask):
    """Pack the usual 5x7 drawing, dropping its empty side and vertical padding."""
    packed, expanded = 0, 0
    for row in range(7):
        pixels = (mask >> (12 + row * 6 + 1)) & 31
        packed |= pixels << (row * 5)
        expanded |= pixels << (12 + row * 6 + 1)
    if expanded != mask:
        return None
    return bytes([packed >> 32]) + (packed & 0xffffffff).to_bytes(4, "little")


def literal(mask):
    narrow = narrow_literal(mask)
    if narrow is not None:
        return narrow
    # Full 66-bit literal for punctuation that uses other rows or columns.
    return bytes([8 | (mask >> 64)]) + (mask & ((1 << 64) - 1)).to_bytes(8, "little")


def decode_literal(data, at):
    tag = data[at]
    if tag < 8:
        packed = tag << 32 | int.from_bytes(data[at + 1:at + 5], "little")
        return sum(((packed >> (row * 5)) & 31) << (12 + row * 6 + 1)
                   for row in range(7))
    if 8 <= tag < 12:
        return (tag & 3) << 64 | int.from_bytes(data[at + 1:at + 9], "little")
    raise ValueError("patch must reference a literal, never another patch")


def record_len(tag):
    if tag < 8:
        return 5
    if tag < 12:
        return 9
    if tag == 32 or 128 <= tag < 192:
        return 3
    if tag >= 192:
        return 4
    raise ValueError("invalid glyph record")


def decode(data, at):
    tag = data[at]
    if tag < 12:
        return decode_literal(data, at)
    if tag == 32:
        return decode_literal(data, int.from_bytes(data[at + 1:at + 3], "little"))
    if tag < 192:
        base = int.from_bytes(data[at + 1:at + 3], "little")
        return decode_literal(data, base) ^ (1 << (tag & 63))
    base = int.from_bytes(data[at + 2:at + 4], "little")
    return decode_literal(data, base) ^ (1 << (tag & 63)) ^ (1 << data[at + 1])


def encode(codes, masks):
    data = bytearray()
    checkpoints = []
    roots = []
    statistics = {"literal": 0, "same": 0, "one_pixel": 0, "two_pixels": 0}
    for index, cp in enumerate(codes):
        if index % STRIDE == 0:
            checkpoints.append(len(data))
        mask = masks[cp]
        best = min(((mask ^ root).bit_count(), offset, root)
                   for root, offset in roots) if roots else None
        raw = literal(mask)
        if best is not None and best[0] <= 2 and (mask ^ best[2]).bit_length() <= 64:
            count, offset, root = best
            diff = mask ^ root
            positions = [bit for bit in range(64) if diff >> bit & 1]
            if count == 0:
                record = bytes([32]) + struct.pack("<H", offset)
                kind = "same"
            elif count == 1:
                record = bytes([128 | positions[0]]) + struct.pack("<H", offset)
                kind = "one_pixel"
            else:
                record = bytes([192 | positions[0], positions[1]]) + struct.pack("<H", offset)
                kind = "two_pixels"
            if len(record) < len(raw):
                data.extend(record)
                statistics[kind] += 1
                continue
        roots.append((mask, len(data)))
        data.extend(raw)
        statistics["literal"] += 1
    if len(data) > 65535:
        raise ValueError("glyph data exceeds u16 reference budget")
    at = 0
    for index, cp in enumerate(codes):
        if index % STRIDE == 0 and checkpoints[index // STRIDE] != at:
            raise ValueError("incorrect checkpoint")
        if decode(data, at) != masks[cp]:
            raise ValueError(f"round trip changed pixels for U+{cp:04X}")
        at += record_len(data[at])
    assert at == len(data)
    return data, checkpoints, statistics


def rust_array(name, kind, values, digits, columns):
    lines = [f"pub(super) const {name}: [{kind}; {len(values)}] = ["]
    for start in range(0, len(values), columns):
        lines.append("    " + ", ".join(f"0x{v:0{digits}x}"
                                          for v in values[start:start + columns]) + ",")
    return "\n".join(lines + ["];", ""])


def preview(path, codes, masks, required):
    from PIL import Image, ImageDraw, ImageFont
    scale, cw, ch = 4, 66, 64
    image = Image.new("RGB", (16 * cw, 64 + 32 * ch), "#181c22")
    draw = ImageDraw.Draw(image)
    font = ImageFont.load_default()
    draw.text((12, 10), "GUI Han: 512 glyphs, each 6x11; pixels shown at 4x", fill="white", font=font)
    draw.text((12, 30), "Cyan: GUI vocabulary   White: common-character fillers", fill="white", font=font)
    for index, cp in enumerate(codes):
        x, y = (index % 16) * cw + 8, 64 + (index // 16) * ch
        color = "#89deec" if chr(cp) in required else "#e6e6e6"
        draw.rectangle((x - 1, y - 1, x + 6 * scale, y + 11 * scale), outline="#3c444e")
        for bit in range(66):
            if not masks[cp] >> (65 - bit) & 1:
                continue
            px, py = x + bit % 6 * scale, y + bit // 6 * scale
            draw.rectangle((px, py, px + scale - 1, py + scale - 1), fill=color)
        draw.text((x - 2, y + 11 * scale + 3), f"{cp:04X}", fill="#a0aab6", font=font)
    image.save(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bdf", type=Path, help=f"download pinned BDF: {SOURCE_URL}")
    parser.add_argument("--out", type=Path, default=ROOT / "src/gui_han_data.rs")
    parser.add_argument("--check", action="store_true", help="verify checked-in data without writing")
    parser.add_argument("--preview", type=Path)
    parser.add_argument("--oracle", type=Path, help="write independent source masks for Rust host validation")
    args = parser.parse_args()
    codes, required = selection()
    masks = read_masks(args.bdf, set(codes))
    data, checkpoints, statistics = encode(codes, masks)
    output = (
        "// Generated by tools/mint_gui_han.py; do not edit by hand.\n"
        "// Glyph data: SIL OFL-1.1; see OFL-cjk.txt and NOTICE-cjk.md.\n"
        "// Copyright (c) 2026 yzdnn; Copyright (c) 2000-2023 Num Kadoma.\n"
        f"// Source revision: {REVISION}; all source pixels preserved.\n\n"
        + rust_array("CODES", "u16", codes, 4, 12)
        + rust_array("CHECKPOINTS", "u16", checkpoints, 4, 12)
        + rust_array("DATA", "u8", data, 2, 20)
    )
    if args.check:
        if args.out.read_text() != output:
            raise ValueError("checked-in glyph tables do not match the source and selection")
    else:
        args.out.write_text(output)
    if args.preview:
        preview(args.preview, codes, masks, required)
    if args.oracle:
        args.oracle.write_bytes(b"".join(struct.pack("<I", cp) + masks[cp].to_bytes(16, "little")
                                         for cp in codes))
    size = len(data) + 2 * len(codes) + 2 * len(checkpoints)
    print(json.dumps({"glyphs": len(codes), "required_gui_glyphs": len(required),
                      "common_fillers": len(codes) - len(required),
                      "distinct_bitmaps": len(set(masks.values())),
                      "table_bytes": size, "records": statistics,
                      "unshared_literal_bytes": sum(len(literal(masks[cp])) for cp in codes)}, indent=2))


if __name__ == "__main__":
    main()
