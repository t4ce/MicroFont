#version 450
// R8 stores coverage. White RGB and SRC_ALPHA blending preserve both native
// line-AA alpha (render-cache bypass) and analytic end/join coverage.
layout(push_constant) uniform Params {
    vec4 start_point_direction; // source coordinates, unit inward direction
    vec4 end_point_direction;
    vec4 style; // 0: quads; positive half-width: lines; negative: join patches
    vec4 transform;
} p;
layout(location=0) out vec4 color;
// Zero style.y retains the regular atlas used by existing consumers.
// Demo cells add pixel gutters without modifying the resident source vertices.
vec2 source_pixel(vec2 at) {
    float scale = p.transform.x;
    if (p.style.y == 0.0) return at*scale;
    vec2 grid = vec2(6.0,11.0);
    vec2 cell = floor(at/grid);
    float half_width = p.transform.y*0.5;
    float dot_overhang = half_width*sqrt(2.0)-scale*0.5+0.5;
    float gutter = max(6.0,ceil(max(half_width+1.0,dot_overhang)));
    vec2 pitch = vec2(ceil(p.style.y*6.0/11.0),p.style.y)+2.0*gutter;
    return cell*pitch + (at-cell*grid)*scale + gutter;
}
float end_coverage(vec4 endpoint, float enabled) {
    if (enabled == 0.0) return 1.0;
    vec2 d = gl_FragCoord.xy-source_pixel(endpoint.xy);
    float along = dot(d,endpoint.zw);
    float radius = p.style.x;
    float across = abs(dot(d,vec2(-endpoint.w,endpoint.z)));
    // Only shade the added outer half-disc. The original body stays intact.
    // Bound the test so other segments returning behind this end are untouched.
    if (along < -radius-0.5 || along >= 0.0 || across > radius) return 1.0;
    return clamp(radius-length(vec2(across,along)),0.0,1.0);
}
void main() {
    float coverage = 1.0;
    if (p.style.x < 0.0) {
        if (gl_FragCoord.z >= 0.5) {
            uint centre = uint(floor((gl_FragCoord.z-0.5)*65536.0+0.5));
            vec2 at = source_pixel(vec2(centre & 127u, centre >> 7u)+0.5);
            coverage = clamp(p.transform.y*0.5-length(gl_FragCoord.xy-at),0.0,1.0);
        } else {
            coverage = clamp(gl_FragCoord.z*64.0,0.0,1.0);
        }
    } else if (p.style.x > 0.0) {
        coverage = min(end_coverage(p.start_point_direction,p.style.z),
                       end_coverage(p.end_point_direction,p.style.w));
    }
    color = vec4(1.0,1.0,1.0,coverage);
}
