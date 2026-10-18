#version 450
#extension GL_GOOGLE_include_directive : require

// Procedural Dista body for TRUEOS's UI4 stamp and optional background atlas.
// Float3 input is a task descriptor, never a CPU-generated position:
// x = vertex token; y = codepoint | column << 7 | shape_row << 11;
// y bits12..14 = atlas grid row, bit15 = atlas, bit16 = exact fast blockers.
// z = circle segments. Each site owns 1+segments disc vertices, followed
// by 66*10 four-vertex link tasks. Unoccupied and rejected tasks degenerate.
// Atlas layout: 16 columns, six rows, 1536x1056 RGBA8 target per shape.
// The generated include snapshots the crate's Basic masks and seed overrides.
#include "basic_seeds.glsl"
layout(location = 0) in vec3 task;

const float SIGMA = 0.20;
const float TAU = 6.28318530717958647692;
// (1,0),(0,1),(1,1),(-1,1),(2,0),(0,2),(2,1),(-2,1),
// (1,2),(-1,2). Packed constants avoid a shader constant-memory allocation.
ivec2 link_offset(uint index) {
    return ivec2(int((0x0b1142d3u >> (3u*index)) & 7u)-2,
                 int((0x000a5854u >> (2u*index)) & 3u));
}

bool occupied(uvec3 mask, ivec2 p) {
    if (p.x < 0 || p.x >= 6 || p.y < 0 || p.y >= 11) return false;
    uint bit = uint(65 - p.y*6 - p.x);
    return ((mask[bit >> 5u] >> (bit & 31u)) & 1u) != 0u;
}

// Gabriel graph pruning: a third occupied site in the closed diameter disc
// suppresses this shortcut. Integer doubled coordinates retain boundary ties.
bool blocked(uvec3 mask, ivec2 a, ivec2 b, int length2) {
    // Cardinal length-two links have blockers one lattice row/column outside
    // their endpoint box, on the diameter-circle boundary.
    ivec2 lo = min(a,b)-ivec2(1), hi = max(a,b)+ivec2(1);
    for (int y = lo.y; y <= hi.y; ++y) {
        for (int x = lo.x; x <= hi.x; ++x) {
            ivec2 p = ivec2(x,y);
            if (all(equal(p,a)) || all(equal(p,b))) continue;
            ivec2 q = 2*p-a-b;
            if (q.x*q.x+q.y*q.y <= length2 && occupied(mask,p)) return true;
        }
    }
    return false;
}

// Exactly the integer sites in each candidate's closed diameter disc, with
// both endpoints excluded. This changes only graph-check work, not geometry.
// The baseline loop above remains available for matched timing experiments.
bool blocked_fast(uvec3 mask, ivec2 a, uint offset) {
    switch (offset) {
    case 0u: case 1u: return false;
    case 2u: return occupied(mask,a+ivec2(1,0)) || occupied(mask,a+ivec2(0,1));
    case 3u: return occupied(mask,a+ivec2(-1,0)) || occupied(mask,a+ivec2(0,1));
    case 4u: return occupied(mask,a+ivec2(1,-1)) || occupied(mask,a+ivec2(1,0)) || occupied(mask,a+ivec2(1,1));
    case 5u: return occupied(mask,a+ivec2(-1,1)) || occupied(mask,a+ivec2(0,1)) || occupied(mask,a+ivec2(1,1));
    case 6u: return occupied(mask,a+ivec2(1,0)) || occupied(mask,a+ivec2(2,0)) || occupied(mask,a+ivec2(0,1)) || occupied(mask,a+ivec2(1,1));
    case 7u: return occupied(mask,a+ivec2(-2,0)) || occupied(mask,a+ivec2(-1,0)) || occupied(mask,a+ivec2(-1,1)) || occupied(mask,a+ivec2(0,1));
    case 8u: return occupied(mask,a+ivec2(1,0)) || occupied(mask,a+ivec2(0,1)) || occupied(mask,a+ivec2(1,1)) || occupied(mask,a+ivec2(0,2));
    case 9u: return occupied(mask,a+ivec2(-1,0)) || occupied(mask,a+ivec2(-1,1)) || occupied(mask,a+ivec2(0,1)) || occupied(mask,a+ivec2(0,2));
    default: return false;
    }
}

void main() {
    // This common outside point also keeps every rejected triangle degenerate.
    gl_Position = vec4(2.0, 2.0, 0.5, 1.0);
    uint token = uint(task.x), packed = uint(task.y);
    uint code = packed & 127u, column = (packed >> 7u) & 15u;
    uint row = (packed >> 11u) & 1u;
    bool atlas = (packed & 32768u) != 0u;
    uint atlas_row = (packed >> 12u) & 7u;
    uint segments = uint(task.z);
    if (segments < 3u || segments > 256u || (!atlas && column >= 10u) || (atlas && atlas_row >= 6u)) return;
    uvec3 mask = basic_seed_mask(code);
    float threshold = row == 0u ? 0.22 : 0.63;
    uint discs_end = 66u * (segments+1u);
    vec2 point;
    if (token < discs_end) {
        uint site = token / (segments+1u), ring = token % (segments+1u);
        ivec2 p = ivec2(int(site%6u), int(site/6u));
        if (!occupied(mask,p)) return;
        point = vec2(p)+vec2(0.5);
        if (ring != 0u) {
            float angle = TAU * float(ring-1u)/float(segments);
            float radius = SIGMA*sqrt(-2.0*log(threshold));
            point += radius*vec2(cos(angle),sin(angle));
        }
    } else {
        uint link_vertex = token-discs_end;
        if (link_vertex >= 66u*10u*4u) return;
        uint candidate = link_vertex/4u, corner = link_vertex%4u;
        uint site = candidate/10u;
        ivec2 a = ivec2(int(site%6u), int(site/6u));
        uint offset = candidate%10u;
        ivec2 delta = link_offset(offset), b = a+delta;
        if (!occupied(mask,a) || !occupied(mask,b)) return;
        int length2 = delta.x*delta.x+delta.y*delta.y;
        float reach = basic_seed_long_reach(code) ? 2.76 : 1.76;
        float length = sqrt(float(length2));
        if (length >= reach) return;
        bool blocked_link = (packed & 65536u) != 0u
            ? blocked_fast(mask,a,offset) : blocked(mask,a,b,length2);
        if (blocked_link) return;
        float t = clamp((reach-length)/0.30,0.0,1.0);
        float weight = t*t*(3.0-2.0*t);
        if (weight <= threshold) return;
        float radius = SIGMA*sqrt(-2.0*log(threshold/weight));
        vec2 normal = vec2(-delta.y,delta.x)*radius/length;
        // Endpoint discs cover the entire weighted link caps because weight
        // never exceeds one. Only this connecting rectangle is necessary.
        point = vec2(corner < 2u ? a : b)+vec2(0.5);
        point += (corner == 0u || corner == 3u) ? normal : -normal;
    }
    uint target_row = atlas ? atlas_row : row;
    vec2 target = atlas ? vec2(1536.0,1056.0) : vec2(960.0,352.0);
    vec2 pixel = vec2(float(column)*96.0,float(target_row)*176.0)+point*16.0;
    gl_Position = vec4(2.0*pixel.x/target.x-1.0,1.0-2.0*pixel.y/target.y,0.5,1.0);
}
