#import bevy_ui::ui_vertex_output::UiVertexOutput

struct SprintBarData {
    progress: f32,
    color: vec4<f32>,
}
@group(1) @binding(0) var<uniform> sprint_bar: SprintBarData;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let centered_uv = vec2<f32>(0.5 - in.uv.x, in.uv.y - 0.5);
    if uv.y < 0.48 || uv.y > 0.52 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return color;
    if uv.x < 0.002 || uv.x > 0.998 || uv.y < 0.482 || uv.y > 0.518 {
        return vec4<f32>(0.05, 0.05, 0.05, 0.9);
    }

    if uv.x <= sprint_bar.progress {
        let tip_glow = smoothstep(sprint_bar.progress - 0.05, sprint_bar.progress, uv.x);
        let color_boost = sprint_bar.color.xyz + vec3<f32>(tip_glow * 0.4);
        return vec4<f32>(color_boost, sprint_bar.color.a);
    } else {
        return vec4<f32>(0.2, 0.2, 0.2, 0.6);
    }
}