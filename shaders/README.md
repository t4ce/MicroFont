# Drawing MicroFont geometry

These optional shaders are for developers writing a **native GPU font renderer**.
Hex bitmap use, software raster rendering, and the SVG stamper require none of
the shipped shaders. The SVG stamper writes geometry for an SVG viewer or
renderer to display.

The `raster` feature supplies borrowed line strips and filled quads. It does not
create a window, graphics context, pipeline, vertex buffer, or render target.
A software renderer can consume the coordinates directly; a hardware renderer
can turn them into triangles and provide the graphics API setup.

Run the dependency-free host example:

```sh
cargo run --example geometry_sink --no-default-features --features raster
```

`examples/geometry_sink.rs` shows a small sink interface and a triangle sink for
`glyph.vert` / `glyph.frag`. These two shaders use desktop GLSL 330, ordinary
uniforms, and vertex attributes. They need no font texture or geometry shader.
The example builds vertices and prints counts; it does not open a GPU or draw
an image. It uses `Vec` for the caller's vertex buffer. The glyph iterators and
the sink traversal do not require allocation.

## Coordinates and stroke semantics

- The cell is 6 units wide by 11 high, with its origin at the top left.
- Line points are lattice sites. Add `0.5` to both coordinates before scaling
  to place the stroke on the corresponding pixel centre.
- Use `Strip::iter_points()` or `point()`. The public `points` slice contains
  canonical source points; aliases may need a mirror or rotation.
- Preserve `Boundary::End` and `Boundary::Junction` independently of position.
  The sample uses round end caps for `End`, butt caps for `Junction`, and round
  joins at internal points. These are a renderer policy, not stored tessellation.
- A segment with identical endpoints is a dot. The sample emits a disc rather
  than normalizing a zero-length direction.
- Quad vertices describe filled cell edges; do not add `0.5`. Split a convex
  quad `(a,b,c,d)` into `(a,b,c)` and `(a,c,d)`. Box drawing, Block Elements and
  Braille are available through the quad lookup.
- Missing geometry returns `None`; the sink does not silently substitute a glyph.

## Portable shader contract

The example's `#[repr(C)] Vertex` is 56 bytes. Bind floating-point attributes:

| Location | Floats | Offset | Meaning |
| --- | --- | --- | --- |
| 0 | 2 | 0 bytes | Position in target pixels |
| 1 | 4 | 8 bytes | Segment start XY, end XY, in target pixels |
| 2 | 4 | 24 bytes | Radius, round-start flag, round-end flag, kind |
| 3 | 4 | 40 bytes | Straight RGBA color |

Kind `0` is a solid triangle, `1` is a nonzero-length stroke, and `2` is a disc.
The three vertices of each triangle must have identical locations 1–3 values.
Provide a positive `viewport_px` uniform equal to the render target dimensions
and use a matching full-target viewport. Coordinates increase right and down;
the vertex shader converts them to OpenGL clip coordinates. Disable depth and
face culling for this flat example, and draw non-indexed triangles.

The fragment shader computes a one-pixel coverage fringe for strokes and discs.
For direct drawing, use straight-alpha blending (`SRC_ALPHA`,
`ONE_MINUS_SRC_ALPHA` for color). Where destination alpha is retained, use
`ONE`, `ONE_MINUS_SRC_ALPHA` for the alpha channel. Choose a consistent color
space for your target. Filled quads use triangle coverage; MSAA or a separate
coverage technique is needed to smooth their diagonal/curved outer edges.

Overlapping segments and join discs can blend more than once. For translucent
text or consistent union coverage, draw coverage into a cleared intermediate
mask with a maximum blend operation, then composite the color once. Glyphs
are not guaranteed to be a non-overlapping mesh. This is a simple renderer
example, not a promise to reproduce TRUEOS's join and anti-aliasing pipeline.

Scale, width, color, cell advance, decoration, clipping and missing-glyph policy
belong to the caller. The example expands stroke geometry after scaling, so its
width is expressed in target pixels. Rotation or another affine transform can
be applied to the geometry before drawing; consistent nonuniform stroke
scaling needs an explicit renderer policy.

## Existing TRUEOS shader references

`mask.vert`, `mask.frag`, and `line_ends.frag` are the existing GLSL 450 TRUEOS
renderer references. They use push constants and TRUEOS-specific atlas,
adjacency, native line anti-aliasing and render-cache contracts. They are
included as implementation references, not as interchangeable partners for the
portable shaders or as a complete graphics backend.
