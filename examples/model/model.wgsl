struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> u_pvm: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) color: u32
) -> VertexOutput {
    var out: VertexOutput;
    out.proj_position = u_pvm * vec4<f32>(pos, 1.0);
    return out;
}

@fragment
fn fs_main(
    vtx: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}