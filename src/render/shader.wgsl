struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) corner_radius: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) corner_radius: f32,
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
    out.local_pos = model.position * instance.size;
    out.rect_size = instance.size;
    out.corner_radius = instance.corner_radius;
    return out;
}

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(size.x, size.y) * 0.5);
    let q = abs(p - size * 0.5) - size * 0.5 + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.corner_radius > 0.5) {
        let dist = rounded_box_sdf(in.local_pos, in.rect_size, in.corner_radius);
        let alpha = 1.0 - smoothstep(-0.5, 0.5, dist);
        if (alpha <= 0.001) {
            discard;
        }
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    }
    return in.color;
}
