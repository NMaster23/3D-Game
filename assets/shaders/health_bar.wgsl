#import bevy_ui::ui_vertex_output::UiVertexOutput

struct HealthBarUIUniforms {
    progress: f32,
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> uniforms: HealthBarUIUniforms;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let centered_uv = in.uv - vec2<f32>(0.5, 0.5);
    let radius = length(centered_uv);
    let inner_radius = 0.40;
    let outer_radius = 0.45;
    let edge = 0.001;
    let ring_alpha = smoothstep(inner_radius - edge, inner_radius, radius) 
                   - smoothstep(outer_radius, outer_radius + edge, radius);
    let angle = atan2(centered_uv.y, centered_uv.x);
    let arc_start = 1.0; 
    let arc_end = -1.0;
    var color = vec4<f32>(0.0);
    if angle <= arc_start && angle >= arc_end {
        let target_angle = arc_start + (arc_end - arc_start) * uniforms.progress;
        let is_filled = smoothstep(target_angle - 0.02, target_angle + 0.02, angle);
        let bg_color = vec4<f32>(uniforms.color.rgb, uniforms.color.a * 0.2);
        color = mix(bg_color, uniforms.color, is_filled);
        color.a *= ring_alpha;
    }
    return color;
}