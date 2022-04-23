// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_view_bind_group
[[group(0), binding(0)]]
var<uniform> view: View;
#import bevy_sprite::mesh2d_struct
[[group(1), binding(0)]]
var<uniform> mesh: Mesh2d;

[[group(2), binding(0)]]
var d_texture: texture_2d<f32>;
[[group(2), binding(1)]]
var d_sampler: sampler;

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
    [[location(2)]] weight: f32;
    [[location(3)]] i_tail_0: vec4<f32>;
    [[location(4)]] i_tail_1: vec4<f32>;
    [[location(5)]] i_tail_2: vec4<f32>;
    [[location(6)]] i_tail_3: vec4<f32>;

    [[location(7)]] i_head_0: vec4<f32>;
    [[location(8)]] i_head_1: vec4<f32>;
    [[location(9)]] i_head_2: vec4<f32>;
    [[location(10)]] i_head_3: vec4<f32>;
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};


/// HACK: This works around naga not supporting matrix addition in SPIR-V 
// translations. See https://github.com/gfx-rs/naga/issues/1527
fn add_matrix(
    a: mat4x4<f32>,
    b: mat4x4<f32>,
) -> mat4x4<f32> {
    return mat4x4<f32>(
        a.x + b.x,
        a.y + b.y,
        a.z + b.z,
        a.w + b.w,
    );
}


/// Entry point for the vertex shader
[[stage(vertex)]]
fn vs_main(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position

    var i_head: mat4x4<f32> = (1.0 - vertex.weight) * mat4x4<f32>(vertex.i_head_0, vertex.i_head_1, vertex.i_head_2, vertex.i_head_3);
    var i_tail: mat4x4<f32> = vertex.weight * mat4x4<f32>(vertex.i_tail_0, vertex.i_tail_1, vertex.i_tail_2, vertex.i_tail_3);

    var interp_model: mat4x4<f32> = add_matrix(i_head, i_tail);

    // var interp_model: mat4x4<f32> = vertex.weight * i_tail + (1.0 - vertex.weight) * i_head;
    out.clip_position = view.view_proj * mesh.model * interp_model * vec4<f32>(vertex.position, 1.0);
    out.uv = vertex.uv;
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    [[location(0)]] uv: vec2<f32>;
};
/// Entry point for the fragment shader
[[stage(fragment)]]
fn fs_main(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    return textureSample(d_texture, d_sampler, in.uv);
}