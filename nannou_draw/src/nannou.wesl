import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput, decompress_vertex},
    mesh_view_bindings::view,
};

const FLAGS_TEXTURE_BIT: u32 = 1u;

struct ShaderModel {
    color: vec4<f32>,
    flags: u32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> model: ShaderModel;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var texture_sampler: sampler;

@vertex
fn vertex(vertex_in: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Use vertex_in.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    let vertex = decompress_vertex(vertex_in, vertex_in.instance_index);
    var world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);

@if(VERTEX_NORMALS)
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_in.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_in.instance_index
    );

@if(VERTEX_POSITIONS) {
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = view.clip_from_world * vec4(out.world_position.xyz, 1.0);
}

@if(VERTEX_UVS_A)
    out.uv = vertex.uv;
@if(VERTEX_UVS_B)
    out.uv_b = vertex.uv_b;

@if(VERTEX_TANGENTS)
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_in.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_in.instance_index
    );

@if(VERTEX_COLORS)
    out.color = vertex.color;

@if(VERTEX_OUTPUT_INSTANCE_INDEX)
    // Use vertex_in.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_in.instance_index;

    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var output_color: vec4<f32> = model.color;

@if(VERTEX_COLORS)
    output_color = output_color * mesh.color;

    if ((model.flags & FLAGS_TEXTURE_BIT) != 0u) {
        output_color = output_color * textureSample(texture, texture_sampler, mesh.uv);
    }

    return output_color;
}
