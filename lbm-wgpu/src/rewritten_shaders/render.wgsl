struct Dimensions{
    row: u32,
    col: u32,
    total: u32,
}

struct VertexOutput{
    @builtin(position) pos: vec4<f32>,
    @location(0) @interpolate(flat) instance_index: u32,
}

fn calc_index(i: u32) -> vec2<f32>{
    return vec2<f32>(2.0*f32(i % dimensions.row)/f32(dimensions.row), -2.0*f32(i / dimensions.row)/f32(dimensions.col)) ;
}

@group(0) @binding(0) var<storage, read_write> colors: array<vec3<f32>>;
@group(1) @binding(0) var<uniform> dimensions: Dimensions;

@vertex
fn vs_main(@location(0) ver: vec2<f32>, @builtin(instance_index) ins: u32) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(calc_index(ins) + ver, 0.0, 1.0);
    out.instance_index = ins;
    return out;
}

struct FragmentInput{
    @location(0) @interpolate(flat) instance_index: u32,
}

@fragment
fn fs_main(f: FragmentInput) -> @location(0) vec4<f32> {
    return vec4<f32>(colors[f.instance_index], 1.0);
}