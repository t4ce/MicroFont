# MicroFont

MicroFont is a tiny `no_std` 6x11 bitmap font stamper for flat pixel buffers.

It exposes the packed font table, direct byte/text stamping helpers, and ARGB helpers for alpha, underline, strikeout, and vertical flip styling. The library has no dependencies.

```rust
let mut pixels = vec![0u32; 88 * 27];
microfont::stamp_text(&mut pixels, 88, 27, 8, 8, "Hello World!", 0xF7E27E)?;
```

The example writes an uncompressed BMP:

```sh
cargo run --example hello_world_bmp
```

Tests also generate visual atlas BMPs in `target/` for manual inspection.
