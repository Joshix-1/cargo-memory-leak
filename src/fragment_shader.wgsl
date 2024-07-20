struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_index: u32,
};

@group(0) @binding(0)
var t: texture_1d<f32>;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureLoad(t, in.texture_index, 0);
}
