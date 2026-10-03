use std::env;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

use microfont::{ERRBUFF, ERRINV, FHEIGHT, measure_text, stamp_text};

const ERR_FILE_CREATE: &str = "file create failed";
const ERR_FILE_WRITE: &str = "file write failed";

fn main() -> Result<(), &'static str> {
    let path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/hello-world.bmp"));

    let text = "Hello World!";
    let margin = 8usize;
    let (text_width, _) = measure_text(text);
    let width = text_width + margin * 2;
    let height = FHEIGHT + margin * 2;

    let mut bitmap = vec![0x101820u32; width * height];
    stamp_text(&mut bitmap, width, height, margin as i32, margin as i32, text, 0xF7E27E)?;

    write_bmp24(&path, width, height, &bitmap)?;
    println!("ok: {}", path.display());
    Ok(())
}

fn write_bmp24(
    path: &Path,
    width: usize,
    height: usize,
    bitmap: &[u32],
) -> Result<(), &'static str> {
    let expected_len = width.checked_mul(height).ok_or(ERRINV)?;
    if bitmap.len() != expected_len {
        return Err(ERRBUFF);
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent).map_err(|_| ERR_FILE_CREATE)?;
        }
    }

    let row_stride = (width * 3).div_ceil(4) * 4;
    let pixel_bytes = row_stride.checked_mul(height).ok_or(ERRINV)?;
    let file_size = 54usize.checked_add(pixel_bytes).ok_or(ERRINV)?;

    let mut file = File::create(path).map_err(|_| ERR_FILE_CREATE)?;

    file.write_all(b"BM").map_err(|_| ERR_FILE_WRITE)?;
    write_u32(&mut file, file_size as u32)?;
    write_u16(&mut file, 0)?;
    write_u16(&mut file, 0)?;
    write_u32(&mut file, 54)?;

    write_u32(&mut file, 40)?;
    write_i32(&mut file, width as i32)?;
    write_i32(&mut file, height as i32)?;
    write_u16(&mut file, 1)?;
    write_u16(&mut file, 24)?;
    write_u32(&mut file, 0)?;
    write_u32(&mut file, pixel_bytes as u32)?;
    write_i32(&mut file, 2835)?;
    write_i32(&mut file, 2835)?;
    write_u32(&mut file, 0)?;
    write_u32(&mut file, 0)?;

    let padding = vec![0u8; row_stride - width * 3];
    for y in (0..height).rev() {
        for x in 0..width {
            let color = bitmap[y * width + x];
            file.write_all(&[
                (color & 0xFF) as u8,
                ((color >> 8) & 0xFF) as u8,
                ((color >> 16) & 0xFF) as u8,
            ])
            .map_err(|_| ERR_FILE_WRITE)?;
        }
        file.write_all(&padding).map_err(|_| ERR_FILE_WRITE)?;
    }

    Ok(())
}

fn write_u16(file: &mut File, value: u16) -> Result<(), &'static str> {
    file.write_all(&value.to_le_bytes())
        .map_err(|_| ERR_FILE_WRITE)
}

fn write_u32(file: &mut File, value: u32) -> Result<(), &'static str> {
    file.write_all(&value.to_le_bytes())
        .map_err(|_| ERR_FILE_WRITE)
}

fn write_i32(file: &mut File, value: i32) -> Result<(), &'static str> {
    file.write_all(&value.to_le_bytes())
        .map_err(|_| ERR_FILE_WRITE)
}
