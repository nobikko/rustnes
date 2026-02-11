@group(0) @binding(0) var sampler1: sampler;
@group(0) @binding(1) var texture1: texture_2d<f32>;

struct Uniforms {
    resolution: vec2<f32>,
}

@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) Position: vec4<f32>,
    @location(0) texCoord: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
    var output: VertexOutput;

    // Triangle strip for a full-screen quad
    let positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
    );

    output.Position = vec4<f32>(positions[vertexIndex], 0.0, 1.0);
    output.texCoord = output.Position.xy * 0.5 + 0.5;

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.texCoord;
    let color = textureSample(texture1, sampler1, uv);
    return color;
}