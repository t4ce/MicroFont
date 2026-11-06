#![no_std]
#![no_main]
use core::hint::black_box;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop { core::hint::spin_loop(); } }

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let text = black_box("MicroFont 8.5.0 █░⠁");
    #[cfg(feature = "hex")]
    {
        let mut pixels = [0u32; 256 * 11];
        let result = microfont::stamp_text(&mut pixels, black_box(256), black_box(11), black_box(0), black_box(0), text, black_box(0xff102030));
        let _ = black_box(result);
        black_box(pixels);
    }
    #[cfg(feature = "svg")]
    {
        struct Sink(u64);
        impl core::fmt::Write for Sink {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                for b in s.bytes() { self.0 = self.0.rotate_left(5) ^ u64::from(b); }
                Ok(())
            }
        }
        let mut sink = Sink(black_box(0));
        let _ = black_box(microfont::svg::write_text(&mut sink, text, black_box(0.8)));
        black_box(sink.0);
    }
    black_box(text);
    loop { core::hint::spin_loop(); }
}

// Volatile byte operations prevent compiler-generated recursive libc calls.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
    for i in 0..n { unsafe { d.add(i).write_volatile(s.add(i).read_volatile()); } }
    d
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(d: *mut u8, c: i32, n: usize) -> *mut u8 {
    for i in 0..n { unsafe { d.add(i).write_volatile(c as u8); } }
    d
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(d: *mut u8, s: *const u8, n: usize) -> *mut u8 {
    if d.addr() <= s.addr() {
        for i in 0..n { unsafe { d.add(i).write_volatile(s.add(i).read_volatile()); } }
    } else {
        for i in (0..n).rev() { unsafe { d.add(i).write_volatile(s.add(i).read_volatile()); } }
    }
    d
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let (x,y) = unsafe { (a.add(i).read_volatile(), b.add(i).read_volatile()) };
        if x != y { return i32::from(x)-i32::from(y); }
    }
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    unsafe { memcmp(a,b,n) }
}
