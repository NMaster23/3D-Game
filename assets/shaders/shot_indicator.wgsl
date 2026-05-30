#import bevy_ui::ui_vertex_output::UiVertexOutput

struct ShotIndicator {
    health_or: u32,
    color: LinearRgba,
    shot: f32,
    magazine: f32,
};

@group(1) @binding(0)
var<uniform> uniforms: ShotIndicatorUIUniforms;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    
}