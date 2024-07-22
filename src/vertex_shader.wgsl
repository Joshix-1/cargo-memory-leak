struct PushConstants {{
    top_left_corner: vec2<f32>,
    cell_width: f32,
    cell_height: f32,
}}

var<push_constant> constants: PushConstants;

struct VertexInput {{
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
    var x: u32 = model.idx / 6u % {width}u;
    var y: u32 = model.idx / 6u / {width}u;

    var out: VertexOutput;
    out.texture_index = textureLoad(t, vec2<u32>(x, y), 0).x;

    var vri: u32 = model.idx % 6u;
    var dx: u32 = u32((vri & 4u) == 4u || vri == 0u);
    var dy: u32 = u32((vri & 2u) == 2u || vri == 5u);

    out.clip_position = vec4<f32>(
        (constants.top_left_corner.x + constants.cell_width * f32(x + dx)) - 1.0,
        -(constants.top_left_corner.y + constants.cell_height * f32(y + dy)) + 1.0,
        0.0,
        1.0
    );
    return out;
}}
