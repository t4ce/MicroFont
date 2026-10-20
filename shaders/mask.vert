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
        uint recipe = uint(source.x);
        uint centre = recipe & 32767u;
        vec2 at = vec2(centre & 127u, centre >> 7u) + 0.5;
        bool rounded = (recipe & 32768u) != 0u;
        bool middle = !rounded && source.y < -0.5;
        pixel = at*p.transform.x + (middle ? vec2(0.0) : source.yz*p.transform.y);
        // Depth is only a PS payload: low range = bevel edge distance;
        // upper range = constant exact packed centre for a circular sector.
        depth = rounded ? 0.5 + float(centre)/65536.0
            : (middle ? (-source.y-1.0)*p.transform.y/64.0 : 0.0);
    } else {
        pixel = source.xy*p.transform.x;
    }
    gl_Position = vec4(pixel*vec2(2.0,-2.0)*p.transform.zw + vec2(-1.0,1.0),depth,1.0);
}
