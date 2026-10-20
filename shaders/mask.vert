#version 450
// Immutable Float3 geometry. Lines/quads contain source-cell coordinates.
// Join vertices contain a packed centre and a width-independent offset.
layout(location=0) in vec3 source;
layout(push_constant) uniform Params {
    vec4 start_point_direction;
    vec4 end_point_direction;
    vec4 style;
    vec4 transform; // source scale, line width in pixels, inverse target extent
} p;
void main() {
    vec2 pixel;
    float depth = 0.5;
    if (p.style.x < 0.0) {
        uint centre = uint(source.x);
        vec2 at = vec2(centre & 127u, centre >> 7u) + 0.5;
        bool middle = source.y < -0.5;
        pixel = at*p.transform.x + (middle ? vec2(0.0) : source.yz*p.transform.y);
        // Distance to the exposed edge, interpolated by the rasterizer.
        depth = middle ? (-source.y-1.0)*p.transform.y/16.0 : 0.0;
    } else {
        pixel = source.xy*p.transform.x;
    }
    gl_Position = vec4(pixel*vec2(2.0,-2.0)*p.transform.zw + vec2(-1.0,1.0),depth,1.0);
}
