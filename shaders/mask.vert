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
// Zero style.y retains the regular atlas used by existing consumers.
// Demo cells add pixel gutters without modifying the resident source vertices.
vec2 source_pixel(vec2 at) {
    float scale = p.transform.x;
    if (p.style.y == 0.0) return at*scale;
    vec2 grid = vec2(6.0,11.0);
    vec2 cell = floor(at/grid);
    float gutter = max(6.0,ceil(p.transform.y*0.5)+1.0);
    vec2 pitch = vec2(ceil(p.style.y*6.0/11.0),p.style.y)+2.0*gutter;
    return cell*pitch + (at-cell*grid)*scale + gutter;
}
void main() {
    vec2 pixel;
    float depth = 0.5;
    if (p.style.x < 0.0) {
        uint recipe = uint(source.x);
        uint centre = recipe & 32767u;
        vec2 at = vec2(centre & 127u, centre >> 7u) + 0.5;
        bool rounded = (recipe & 32768u) != 0u;
        bool middle = !rounded && source.y < -0.5;
        pixel = source_pixel(at) + (middle ? vec2(0.0) : source.yz*p.transform.y);
        // Depth is only a PS payload: low range = bevel edge distance;
        // upper range = constant exact packed centre for a circular sector.
        depth = rounded ? 0.5 + float(centre)/65536.0
            : (middle ? (-source.y-1.0)*p.transform.y/64.0 : 0.0);
    } else {
        pixel = source_pixel(source.xy);
        if (p.style.x > 0.0) {
            // Grow the native bar outwards; the seed remains the cap centre.
            // Adjacency vertices and junction endpoints are never extended.
            if (source.z == 0.25 && p.style.z != 0.0)
                pixel -= p.start_point_direction.zw*p.style.x;
            if (source.z == 0.75 && p.style.w != 0.0)
                pixel -= p.end_point_direction.zw*p.style.x;
        }
    }
    gl_Position = vec4(pixel*vec2(2.0,-2.0)*p.transform.zw + vec2(-1.0,1.0),depth,1.0);
}
