#version 330 core
in vec2 pixel;
flat in vec4 segment;
flat in vec4 shape;
flat in vec4 tint;
out vec4 color;
void main() {
    float coverage = 1.0;
    if (shape.w > 0.5) {
        vec2 offset = pixel - segment.xy;
        float distance_from_edge;
        if (shape.w > 1.5) {
            // A repeated lattice point or a round internal join.
            distance_from_edge = length(offset) - shape.x;
        } else {
            vec2 delta = segment.zw - segment.xy;
            float span = length(delta);
            vec2 direction = delta / span;
            float along = dot(offset, direction);
            float across = dot(offset, vec2(-direction.y, direction.x));
            distance_from_edge = abs(across) - shape.x;
            if (along < 0.0) {
                distance_from_edge = shape.y > 0.5
                    ? length(vec2(along, across)) - shape.x
                    : max(distance_from_edge, -along);
            } else if (along > span) {
                distance_from_edge = shape.z > 0.5
                    ? length(vec2(along - span, across)) - shape.x
                    : max(distance_from_edge, along - span);
            }
        }
        coverage = clamp(0.5 - distance_from_edge, 0.0, 1.0);
    }
    // Straight alpha. The caller selects blending and target color space.
    color = vec4(tint.rgb, tint.a * coverage);
}
