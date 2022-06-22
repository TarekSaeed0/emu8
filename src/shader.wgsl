struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    return out;
}

[[group(0), binding(0)]]
var texture: texture_1d<u32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let x = u32(in.tex_coords.x);
    let y = u32(in.tex_coords.y);
    if (extractBits(textureLoad(texture, i32(((y * 64u) + x) / 8u), 0).r, x % 8u, 1u) != 0u) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}