struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

struct ViewportUniform {
    size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> viewport: ViewportUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = instance.position + model.position * instance.size;
    let ndc_x = (world_pos.x / (viewport.size.x / 2.0)) - 1.0;
    let ndc_y = 1.0 - (world_pos.y / (viewport.size.y / 2.0));
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
