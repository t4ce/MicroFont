//! Shared, allocation-free validation of the selected single-file font image.
pub const HEADER: usize = 24;
pub const ENTRY: usize = 24;
pub fn u32_at(data: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
}
pub fn find(data: &[u8], id: u32) -> Option<usize> {
    let mut lo = 0;
    let mut hi = u32_at(data, 12) as usize;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let at = HEADER + mid * ENTRY;
        match u32_at(data, at).cmp(&id) {
            core::cmp::Ordering::Less => lo = mid + 1,
            core::cmp::Ordering::Greater => hi = mid,
            core::cmp::Ordering::Equal => return Some(at),
        }
    }
    None
}
fn lines(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.is_empty() {
        return Err("empty line glyph");
    }
    let mut cursor = 0;
    while cursor < bytes.len() {
        let count = (bytes[cursor] & 63) as usize + 2;
        cursor += 1;
        let end = cursor.checked_add(count).ok_or("line size overflow")?;
        if end > bytes.len() {
            return Err("truncated line strip");
        }
        for &p in &bytes[cursor..end] {
            if p & 128 != 0 || p & 7 >= 6 || p >> 3 >= 11 {
                return Err("invalid line point");
            }
        }
        cursor = end;
    }
    Ok(())
}
pub fn validate(data: &[u8]) -> Result<(), &'static str> {
    if data.len() < HEADER || &data[..8] != b"MFBNDL2\0" {
        return Err("font magic");
    }
    if data[8..10] != [2, 0] || data[10] & !3 != 0 || data[10] == 0 || data[11] != 0 {
        return Err("font version/features");
    }
    if u32_at(data, 16) != HEADER as u32 || u32_at(data, 20) as usize != data.len() {
        return Err("font extent");
    }
    let count = u32_at(data, 12) as usize;
    let mut cursor = count
        .checked_mul(ENTRY)
        .and_then(|n| n.checked_add(HEADER))
        .ok_or("font index overflow")?;
    if cursor > data.len() {
        return Err("truncated font index");
    }
    let mut previous = None;
    for i in 0..count {
        let at = HEADER + i * ENTRY;
        let id = u32_at(data, at);
        if previous.is_some_and(|p| p >= id) {
            return Err("unsorted/duplicate font ids");
        }
        if char::from_u32(id).is_none()
            && !(0x110000..=0x110005).contains(&id)
            && !(0x110100..=0x110106).contains(&id)
        {
            return Err("invalid font id");
        }
        previous = Some(id);
        let mask = u32_at(data, at + 4) as usize;
        let raster = u32_at(data, at + 8) as usize;
        let len = u32_at(data, at + 12) as usize;
        let kind = data[at + 16];
        if data[at + 17] & !1 != 0 || data[at + 18..at + ENTRY].iter().any(|&v| v != 0) {
            return Err("font entry flags");
        }
        if mask != 0 {
            if data[10] & 1 == 0 || data[at + 17] & 1 == 0 || mask != cursor {
                return Err("font mask placement");
            }
            cursor = cursor.checked_add(16).ok_or("mask overflow")?;
            if cursor > data.len()
                || data[mask + 8] & !3 != 0
                || data[mask + 9..cursor].iter().any(|&v| v != 0)
            {
                return Err("invalid bitmap mask");
            }
        } else if data[at + 17] & 1 != 0 {
            return Err("missing bitmap mask");
        }
        if kind == 0 {
            if raster != 0 || len != 0 || mask == 0 {
                return Err("empty font entry");
            }
        } else {
            if data[10] & 2 == 0 || raster != cursor || kind > 5 {
                return Err("font raster placement");
            }
            cursor = cursor.checked_add(len).ok_or("raster overflow")?;
            if cursor > data.len() {
                return Err("truncated font raster");
            }
            let bytes = &data[raster..cursor];
            match kind {
                1 => lines(bytes)?,
                2 => {
                    if len % 32 != 0 {
                        return Err("quad size");
                    }
                    for (j, value) in bytes.chunks_exact(4).enumerate() {
                        let f = f32::from_le_bytes(value.try_into().unwrap());
                        if !f.is_finite() || f < 0.0 || f > if j % 2 == 0 { 6.0 } else { 11.0 } {
                            return Err("invalid quad coordinate");
                        }
                    }
                }
                3 if len == 5 && (1..=5).contains(&bytes[4]) => {}
                4 if len == 9 && bytes[8] <= 1 => {}
                5 if len != 0 && len % 4 == 0 => {}
                _ => return Err("invalid raster recipe"),
            }
        }
    }
    if cursor != data.len() {
        return Err("font trailing bytes");
    }
    // Only canonical line sources are referenced, so recipes cannot cycle.
    for i in 0..count {
        let at = HEADER + i * ENTRY;
        let kind = data[at + 16];
        if !matches!(kind, 3..=5) {
            continue;
        }
        let start = u32_at(data, at + 8) as usize;
        let len = u32_at(data, at + 12) as usize;
        let target_kind = |id| {
            find(data, id)
                .map(|a| data[a + 16])
                .ok_or("missing raster source")
        };
        match kind {
            3 => {
                if target_kind(u32_at(data, start))? != 1 {
                    return Err("alias source must be canonical lines");
                }
            }
            4 => {
                if !matches!(target_kind(u32_at(data, start))?, 1 | 3)
                    || target_kind(u32_at(data, start + 4))? != 1
                {
                    return Err("composition source");
                }
            }
            5 => {
                for off in (start..start + len).step_by(4) {
                    if target_kind(u32_at(data, off))? != 1 {
                        return Err("segment source");
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}
