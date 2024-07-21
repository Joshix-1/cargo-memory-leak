struct VertexInput {{
    @location(0) position: vec2<f32>,
    @builtin(vertex_index) idx : u32,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_index: u32,
}};

@group(1) @binding(0)
var t: texture_2d<u32>;

@vertex
fn main(
    model: VertexInput,
) -> VertexOutput {{
    var out: VertexOutput;
    var cell_idx: u32 = model.idx / 4u;
    out.texture_index = textureLoad(
        t,
        vec2<u32>(
            cell_idx % {width}u,
            cell_idx / {height}u
        ),
        0
    ).x;
    out.clip_position = vec4<f32>(
        model.position.x - 1.0,
        -model.position.y + 1.0,
        0.0,
        1.0
    );
    return out;
}}