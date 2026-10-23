//! Dependency-free packer. Only TRUEOS builds invoke the host LZ4 tool.
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Source {
    id: u32,
    mask: [u8; 16],
    flags: u8,
    kind: u8,
    raster: Vec<u8>,
}

fn sources(dir: &Path, records: &mut BTreeMap<u32, Source>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    for entry in fs::read_dir(dir).expect("read glyph directory") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, records);
            continue;
        }
        if path.extension().is_none_or(|e| e != "bin") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let parent = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let id = if parent == "lowercase-toppers" {
            0x110000
                + ["diaeresis", "acute", "grave", "circumflex", "ring", "tilde"]
                    .iter()
                    .position(|n| *n == stem)
                    .expect("unknown lowercase topper") as u32
        } else if parent == "seven-segment" {
            let n: u32 = stem
                .strip_prefix("segment")
                .expect("segment name")
                .parse()
                .unwrap();
            assert!(n < 7);
            0x110100 + n
        } else {
            let id = u32::from_str_radix(stem.strip_prefix('u').expect("uXXXX.bin"), 16).unwrap();
            assert!(
                char::from_u32(id).is_some() && stem == format!("u{id:04X}"),
                "invalid glyph name"
            );
            id
        };
        let bytes = fs::read(&path).unwrap();
        assert!(
            bytes.len() >= 20 && bytes[18..20] == [0, 0],
            "{}: invalid mask-first header",
            path.display()
        );
        let mask: [u8; 16] = bytes[..16].try_into().unwrap();
        assert!(u128::from_le_bytes(mask) >> 66 == 0, "{}: mask exceeds 6x11", path.display());
        assert!(bytes[16] & !1 == 0 && bytes[17] <= 5, "{}: invalid flags/kind", path.display());
        assert!(
            bytes[16] & 1 != 0 || mask == [0; 16],
            "{}: bitmap value needs its presence flag",
            path.display()
        );
        assert!(
            bytes[17] != 0 || bytes.len() == 20,
            "{}: kind 0 cannot have a raster payload",
            path.display()
        );
        assert!(
            records
                .insert(
                    id,
                    Source {
                        id,
                        mask,
                        flags: bytes[16],
                        kind: bytes[17],
                        raster: bytes[20..].to_vec()
                    }
                )
                .is_none(),
            "duplicate glyph id"
        );
    }
}

fn put32(bytes: &mut [u8], at: usize, value: usize) {
    bytes[at..at + 4].copy_from_slice(
        &u32::try_from(value)
            .expect("font exceeds 4GiB")
            .to_le_bytes(),
    );
}

fn pack(records: &BTreeMap<u32, Source>, flags: u16) -> Vec<u8> {
    let selected: Vec<_> = records
        .values()
        .filter(|r| (flags & 1 != 0 && r.flags & 1 != 0) || (flags & 2 != 0 && r.kind != 0))
        .collect();
    let mut bytes = vec![0; 24 + 24 * selected.len()];
    bytes[..8].copy_from_slice(b"MFBNDL2\0");
    bytes[8..10].copy_from_slice(&2u16.to_le_bytes());
    bytes[10..12].copy_from_slice(&flags.to_le_bytes());
    put32(&mut bytes, 12, selected.len());
    put32(&mut bytes, 16, 24);
    for (i, r) in selected.iter().enumerate() {
        let at = 24 + i * 24;
        put32(&mut bytes, at, r.id as usize);
        if flags & 1 != 0 && r.flags & 1 != 0 {
            let offset = bytes.len();
            put32(&mut bytes, at + 4, offset);
            bytes.extend(r.mask);
            bytes[at + 17] |= 1;
        }
        if flags & 2 != 0 && r.kind != 0 {
            let offset = bytes.len();
            put32(&mut bytes, at + 8, offset);
            put32(&mut bytes, at + 12, r.raster.len());
            bytes[at + 16] = r.kind;
            bytes.extend(&r.raster);
        }
    }
    let len = bytes.len();
    put32(&mut bytes, 20, len);
    bytes
}

// The same parser validates build output and installed runtime bundles.
#[path = "src/bundle_format.rs"]
mod format;

fn tar(payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 512];
    out[..13].copy_from_slice(b"microfont.bin");
    for (at, len, value) in [
        (100, 8, 0o444u64),
        (108, 8, 0),
        (116, 8, 0),
        (124, 12, payload.len() as u64),
        (136, 12, 0),
    ] {
        let field = format!("{value:0width$o}\0", width = len - 1);
        out[at..at + len].copy_from_slice(field.as_bytes());
    }
    out[148..156].fill(b' ');
    out[156] = b'0';
    out[257..263].copy_from_slice(b"ustar\0");
    out[263..265].copy_from_slice(b"00");
    let checksum: usize = out.iter().map(|&v| v as usize).sum();
    out[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
    out.extend(payload);
    out.resize(out.len().div_ceil(512) * 512 + 1024, 0);
    out
}

fn main() {
    println!("cargo:rustc-check-cfg=cfg(target_os, values(\"trueos\"))");
    println!("cargo:rerun-if-env-changed=MICROFONT_SOURCE_DIR");
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let dir = env::var_os("MICROFONT_SOURCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("data/line-v1"));
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::create_dir_all(&out).unwrap();
    let mut flags = 0;
    for (feature, bit) in [("HEX", 1), ("RASTER", 2)] {
        if env::var_os(format!("CARGO_FEATURE_{feature}")).is_some() {
            flags |= bit;
        }
    }
    assert!(flags & 3 != 0, "MicroFont needs feature hex, raster, or both");
    let mut records = BTreeMap::new();
    sources(&dir, &mut records);
    // Validate source edits even when this build omits that representation.
    let full = pack(&records, 3);
    format::validate(&full).expect("invalid MicroFont source records");
    let bytes = if flags == 3 {
        full
    } else {
        pack(&records, flags)
    };
    format::validate(&bytes).expect("invalid MicroFont bundle");
    fs::write(out.join("microfont.bin"), &bytes).unwrap();
    fs::write(out.join("bundle_size.rs"), bytes.len().to_string()).unwrap();
    fs::write(out.join("bundle_flags.rs"), flags.to_string()).unwrap();
    let mut characters = String::from("&[");
    if flags & 2 != 0 {
        for r in records
            .values()
            .filter(|r| matches!(r.kind, 1 | 3 | 4 | 5) && r.id <= 0x10ffff)
        {
            characters.push_str(&format!("'\\u{{{:X}}}',", r.id));
        }
    }
    characters.push(']');
    fs::write(out.join("line_characters.rs"), characters).unwrap();
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("trueos") {
        let tar_path = out.join("microfont.tar");
        fs::write(&tar_path, tar(&bytes)).unwrap();
        let status = Command::new("lz4")
            .args(["-q", "-12", "-f", "--content-size"])
            .arg(&tar_path)
            .arg(out.join("microfont.tar.lz4"))
            .status()
            .expect("TRUEOS font packaging requires the host lz4 tool");
        assert!(status.success(), "LZ4 font packaging failed");
        fs::remove_file(tar_path).unwrap();
    }
}
