//! One selected font image. Portable builds need neither allocation nor deps.
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
#[path = "bundle_format.rs"]
mod format;
pub use format::assets::HOME_BUTTON_ID;
#[cfg(feature = "raster")]
pub(crate) use format::corner;
include!("cp850_characters.rs");

pub const UNCOMPRESSED_BUNDLE_BYTES: usize = include!(concat!(env!("OUT_DIR"), "/bundle_size.rs"));
pub const BUNDLE_FLAGS: u16 = include!(concat!(env!("OUT_DIR"), "/bundle_flags.rs"));
#[cfg(target_os = "trueos")]
pub const EMBEDDED_ARCHIVE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/microfont.tar.lz4"));
#[cfg(not(target_os = "trueos"))]
pub const EMBEDDED_BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/microfont.bin"));
static INSTALLED: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
static LENGTH: AtomicUsize = AtomicUsize::new(0);
static CLAIMED: AtomicBool = AtomicBool::new(false);

/// Validate an immutable process-lifetime image before publishing it. TRUEOS
/// calls this before text output. Installation cannot replace an active image.
pub fn install_bundle(data: &'static [u8]) -> Result<(), &'static str> {
    format::validate(data)?;
    if u16::from_le_bytes([data[10], data[11]]) != BUNDLE_FLAGS {
        return Err("font feature mismatch");
    }
    // A single writer owns both pointer and length. A losing concurrent caller
    // cannot replace the length belonging to the published static allocation.
    CLAIMED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| "font already installed")?;
    LENGTH.store(data.len(), Ordering::Relaxed);
    INSTALLED.store(data.as_ptr() as *mut u8, Ordering::Release);
    Ok(())
}

pub fn is_initialized() -> bool {
    #[cfg(not(target_os = "trueos"))]
    {
        true
    }
    #[cfg(target_os = "trueos")]
    {
        !INSTALLED.load(Ordering::Acquire).is_null()
    }
}
pub fn bytes() -> Option<&'static [u8]> {
    let ptr = INSTALLED.load(Ordering::Acquire);
    if !ptr.is_null() {
        // SAFETY: install_bundle only publishes validated static slices, never
        // replaces them, and its release synchronizes the initialized length.
        return Some(unsafe { core::slice::from_raw_parts(ptr, LENGTH.load(Ordering::Relaxed)) });
    }
    #[cfg(not(target_os = "trueos"))]
    {
        Some(EMBEDDED_BUNDLE)
    }
    #[cfg(target_os = "trueos")]
    {
        None
    }
}

#[derive(Clone, Copy)]
pub struct Record {
    data: &'static [u8],
    at: usize,
}
impl Record {
    pub fn kind(self) -> u8 {
        self.data[self.at + 16]
    }
    pub fn raster(self) -> &'static [u8] {
        let start = format::u32_at(self.data, self.at + 8) as usize;
        let len = format::u32_at(self.data, self.at + 12) as usize;
        &self.data[start..start + len]
    }
    #[cfg(feature = "hex")]
    pub fn bitmap(self) -> Option<u128> {
        let at = format::u32_at(self.data, self.at + 4) as usize;
        (at != 0).then(|| u128::from_le_bytes(self.data[at..at + 16].try_into().unwrap()))
    }
}
pub fn record(id: u32) -> Option<Record> {
    let data = bytes()?;
    Some(Record {
        data,
        at: format::find(data, id)?,
    })
}
#[cfg(feature = "hex")]
pub fn bitmap(character: char) -> Option<u128> {
    record(character as u32)?.bitmap()
}

/// Validate an external image without allocation or installation.
pub fn validate(data: &[u8]) -> Result<(), &'static str> {
    format::validate(data)
}
