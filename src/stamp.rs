//! Bitmap stamping into flat pixel buffers.
use crate::{
    CELL_BITS, ERRBUFF, ERRINV, FHEIGHT, FLIPY, FWIDTH, STRIKE, ULINE,
    font_pixels, glyph_byte, glyph_cell_pixels, special_cells,
};

const GLYBITS: usize = 64;

pub fn stamp_text<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    text: &str,
    color: T,
) -> Result<(), &'static str> {
    stamp_text_with_stride(buffer, width, height, width, x, y, text, color)
}
pub fn stamp_bytes<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    bytes: &[u8],
    color: T,
) -> Result<(), &'static str> {
    stamp_bytes_with_stride(buffer, width, height, width, x, y, bytes, color)
}
pub fn stamp_text_with_stride<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    stride: usize,
    x: i32,
    y: i32,
    text: &str,
    color: T,
) -> Result<(), &'static str> {
    stamp_glyphs_with_stride(
        buffer,
        width,
        height,
        stride,
        x,
        y,
        text.chars().flat_map(|ch| {
            let cells = special_cells(ch);
            let ordinary = if cells.is_none() { Some(ch) } else { None };
            cells.into_iter().flatten().copied().chain(ordinary)
                .map(|cell| (glyph_cell_pixels(cell), i32::from(glyph_byte(cell) == b'q')))
        }),
        color,
    )
}
pub fn stamp_bytes_with_stride<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    stride: usize,
    x: i32,
    y: i32,
    bytes: &[u8],
    color: T,
) -> Result<(), &'static str> {
    stamp_glyphs_with_stride(
        buffer,
        width,
        height,
        stride,
        x,
        y,
        bytes
            .iter()
            .copied()
            .map(|ch| ((font_pixels(ch) as u128) << 2, i32::from(ch == b'q'))),
        color,
    )
}
fn stamp_glyphs_with_stride<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    stride: usize,
    x: i32,
    y: i32,
    glyphs: impl Iterator<Item = (u128, i32)>,
    color: T,
) -> Result<(), &'static str> {
    if stride < width {
        return Err(ERRINV);
    }

    let required_len = if width == 0 || height == 0 {
        0
    } else {
        (height - 1)
            .checked_mul(stride)
            .and_then(|row| row.checked_add(width))
            .ok_or(ERRINV)?
    };
    if buffer.len() < required_len {
        return Err(ERRBUFF);
    }

    let width_i32 = i32::try_from(width).map_err(|_| ERRINV)?;
    let height_i32 = i32::try_from(height).map_err(|_| ERRINV)?;

    for (index, (pixels, x_bias)) in glyphs.enumerate() {
        let glyph_x = x + index as i32 * FWIDTH as i32 + x_bias;

        for bit in 0..CELL_BITS {
            if (pixels >> (CELL_BITS - 1 - bit)) & 1 == 0 {
                continue;
            }

            let px = glyph_x + (bit % FWIDTH) as i32;
            let py = y + (bit / FWIDTH) as i32;
            if px >= 0 && py >= 0 && px < width_i32 && py < height_i32 {
                buffer[py as usize * stride + px as usize] = color;
            }
        }
    }
    Ok(())
}
pub fn stamp_argb(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    bytes: &[u8],
    argb: u32,
    style: u8,
) -> Result<(), &'static str> {
    stamp_argb_stride(buffer, width, height, width, x, y, bytes, argb, style)
}
pub fn stamp_argb_stride(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    stride: usize,
    x: i32,
    y: i32,
    bytes: &[u8],
    argb: u32,
    style: u8,
) -> Result<(), &'static str> {
    if stride < width {
        return Err(ERRINV);
    }
    let required_len = if width == 0 || height == 0 {
        0
    } else {
        (height - 1)
            .checked_mul(stride)
            .and_then(|row| row.checked_add(width))
            .ok_or(ERRINV)?
    };
    if buffer.len() < required_len {
        return Err(ERRBUFF);
    }
    let width_i32 = i32::try_from(width).map_err(|_| ERRINV)?;
    let height_i32 = i32::try_from(height).map_err(|_| ERRINV)?;
    for (index, character) in bytes.iter().copied().enumerate() {
        let pixels = font_pixels(character);
        let x_bias = i32::from(character == b'q');
        let glyph_x = x + index as i32 * FWIDTH as i32 + x_bias;
        for bit in 0..GLYBITS {
            if (pixels >> (63 - bit)) & 1 == 0 {
                continue;
            }
            let row = (bit / FWIDTH) as i32;
            let py = y + if style & FLIPY != 0 {
                FHEIGHT as i32 - 1 - row
            } else {
                row
            };
            let px = glyph_x + (bit % FWIDTH) as i32;
            put_argb(buffer, width_i32, height_i32, stride, px, py, argb);
        }
    }
    let w = bytes.len() as i32 * FWIDTH as i32;
    if style & STRIKE != 0 {
        hline_argb(buffer, width_i32, height_i32, stride, x, y + FHEIGHT as i32 / 2, w, argb);
    }
    if style & ULINE != 0 {
        hline_argb(buffer, width_i32, height_i32, stride, x, y + FHEIGHT as i32, w, argb);
    }
    Ok(())
}
fn put_argb(buffer: &mut [u32], width: i32, height: i32, stride: usize, x: i32, y: i32, argb: u32) {
    if x >= 0 && y >= 0 && x < width && y < height {
        buffer[y as usize * stride + x as usize] = argb;
    }
}
fn hline_argb(
    buffer: &mut [u32],
    width: i32,
    height: i32,
    stride: usize,
    x: i32,
    y: i32,
    w: i32,
    argb: u32,
) {
    for dx in 0..w {
        put_argb(buffer, width, height, stride, x + dx, y, argb);
    }
}
