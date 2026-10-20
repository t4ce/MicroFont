#version 450
// Native-AA lines supply coverage through the render-cache bypass path.
// Shader alpha adds coverage for circular ends and the outer edge of join patches.
layout(push_constant) uniform Params {
    vec4 start_point_direction; // xy endpoint, zw unit inward direction, pixels
    vec4 end_point_direction;
    vec4 style; // positive half-width: line; negative half-width: patch mode
                // negative radius: AA line, positive radius: legacy trim
                // patch mode: y=AA width, z=depth-to-pixel distance scale
    vec4 rgba;
} p;
layout(location=0) out vec4 color;
// The active native raster uses major-axis line width and perpendicular caps.
// Convert its configured half-width to the true perpendicular distance before
// rounding. A half-width radius gives a fully round cap on every slope.
bool trim_corner(vec4 endpoint, float enabled) {
    if (enabled == 0.0 || p.style.y <= 0.0) return false;
    vec2 inward = endpoint.zw;
    float half_width = p.style.x * max(abs(inward.x), abs(inward.y));
    float radius = min(p.style.y, half_width);
    vec2 d = gl_FragCoord.xy - endpoint.xy;
    float along = dot(d, inward);
    float across = abs(dot(d, vec2(-inward.y, inward.x)));
    if (along < 0.0 || along >= radius || across <= half_width-radius || across > half_width)
        return false;
    vec2 q = vec2(across-(half_width-radius), radius-along);
    return dot(q,q) > radius*radius;
}
float end_coverage(vec4 endpoint, float enabled) {
    if (enabled == 0.0) return 1.0;
    vec2 d = gl_FragCoord.xy - endpoint.xy;
    float along = dot(d, endpoint.zw);
    float radius = p.style.x; // Native AA uses true perpendicular line width.
    float across = abs(dot(d, vec2(-endpoint.w, endpoint.z)));
    // Only this endpoint's cap box. A strip can return behind its endpoint
    // elsewhere; those distant segments must not be cut by this circle.
    if (along < -0.5 || along >= radius || across > radius) return 1.0;
    float margin = radius-length(vec2(across, radius-along));
    return clamp(margin,0.0,1.0); // Same 1px inward fringe as the native side AA.
}
void main() {
    if (p.style.x < 0.0) {
        // Triangle Z is an interpolated distance to the exterior edge only.
        float coverage = clamp(gl_FragCoord.z*p.style.z/p.style.y,0.0,1.0);
        color = vec4(p.rgba.rgb,p.rgba.a*coverage);
    } else if (p.style.y < 0.0) {
        float coverage = min(end_coverage(p.start_point_direction,p.style.z),
                             end_coverage(p.end_point_direction,p.style.w));
        color = vec4(p.rgba.rgb,p.rgba.a*coverage);
    } else {
        if (trim_corner(p.start_point_direction,p.style.z) || trim_corner(p.end_point_direction,p.style.w))
            discard;
        color = p.rgba;
    }
}
