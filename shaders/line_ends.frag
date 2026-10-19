#version 450
// Native lines supply coverage. Remove only End corner pixels; no body geometry.
layout(push_constant) uniform Params {
    vec4 start_point_direction; // xy endpoint, zw unit inward direction, pixels
    vec4 end_point_direction;
    vec4 style; // half-width, corner radius, start enabled, end enabled
    vec4 rgba;
} p;
layout(location=0) out vec4 color;
bool trim_corner(vec4 endpoint, float enabled) {
    if (enabled == 0.0 || p.style.y <= 0.0) return false;
    vec2 d = gl_FragCoord.xy - endpoint.xy;
    float along = dot(d, endpoint.zw);
    float across = abs(dot(d, vec2(-endpoint.w, endpoint.z)));
    float radius = p.style.y;
    if (along < 0.0 || along >= radius || across <= p.style.x-radius || across > p.style.x)
        return false;
    vec2 q = vec2(across-(p.style.x-radius), radius-along);
    return dot(q,q) > radius*radius;
}
void main() {
    if (trim_corner(p.start_point_direction,p.style.z) || trim_corner(p.end_point_direction,p.style.w))
        discard;
    color = p.rgba;
}
