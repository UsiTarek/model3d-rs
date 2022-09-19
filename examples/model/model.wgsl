struct VertexOut {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal: vec3<f32>,
};
type FragmentIn = VertexOut;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>
};

@group(0)
@binding(0)
var<uniform> u_pvm: mat4x4<f32>;

@vertex
fn vs_main(
    in: VertexIn
) -> VertexOut {
    var out: VertexOut;

    out.proj_position = u_pvm * vec4<f32>(in.pos, 1.0);
    out.color = in.color;
    out.normal = in.normal;

    return out;
}

@fragment
fn fs_main(
    in: FragmentIn
) -> @location(0) vec4<f32> {

    if (length(in.color) != 0.0) {
        return vec4(in.color.xyz, 1.0);
    } else {
        return vec4(in.normal, 1.0);
    }
}