struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@group(1) @binding(1)
var palette: texture_2d<f32>;
@group(1) @binding(2)
var palette_sampler: sampler;

let light_direction = vec3<f32>(0.0, 0.917, 0.4);

fn color(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(palette, palette_sampler, uv);
}

fn lighting(normal: vec3<f32>) -> f32 {
    return 0.8 + abs(dot(normal, light_direction)) * 0.2;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return color(input.uv) * lighting(input.world_normal);
}
