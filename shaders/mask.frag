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
float end_coverage(vec4 endpoint, float enabled) {
    if (enabled == 0.0) return 1.0;
    vec2 d = gl_FragCoord.xy-endpoint.xy*p.transform.x;
    float along = dot(d,endpoint.zw);
    float radius = p.style.x;
    float across = abs(dot(d,vec2(-endpoint.w,endpoint.z)));
    if (along < -0.5 || along >= radius || across > radius) return 1.0;
    return clamp(radius-length(vec2(across,radius-along)),0.0,1.0);
}
void main() {
    float coverage = 1.0;
    if (p.style.x < 0.0) {
        if (gl_FragCoord.z >= 0.5) {
            uint centre = uint(floor((gl_FragCoord.z-0.5)*65536.0+0.5));
            vec2 at = (vec2(centre & 127u, centre >> 7u)+0.5)*p.transform.x;
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
