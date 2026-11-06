#version 330 core
// Portable triangle transport; see examples/geometry_sink.rs and shaders/README.md.
layout(location = 0) in vec2 position_px;
layout(location = 1) in vec4 segment_px;
layout(location = 2) in vec4 style; // radius, round start, round end, primitive kind
layout(location = 3) in vec4 rgba;
uniform vec2 viewport_px;
out vec2 pixel;
flat out vec4 segment;
flat out vec4 shape;
flat out vec4 tint;
void main() {
    pixel = position_px;
    segment = segment_px;
    shape = style;
    tint = rgba;
    gl_Position = vec4(position_px / viewport_px * vec2(2.0, -2.0)
                       + vec2(-1.0, 1.0), 0.0, 1.0);
}
