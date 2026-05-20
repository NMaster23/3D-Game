#import bevy_ui::ui_vertex_output::UiVertexOutput

struct MuzzleFlashUniforms {
    power: f32,
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> uniforms: MuzzleFlashUniforms;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let centered_uv = vec2<f32>(0.5 - in.uv.x, in.uv.y - 0.5);
    let dist = length(centered_uv);
    let radius = uniforms.power * 0.5;
    let alpha = 1.0 - smoothstep(radius - 0.01, radius + 0.01, dist);
    return vec4<f32>(uniforms.color.rgb, uniforms.color.a * alpha);
}