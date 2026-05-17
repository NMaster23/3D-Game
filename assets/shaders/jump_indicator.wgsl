#import bevy_ui::ui_vertex_output::UiVertexOutput

struct JumpIndicatorUniforms {
    progress: f32,
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> uniforms: JumpIndicatorUniforms;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    if in.uv.x <= uniforms.progress {
        return uniforms.color;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}