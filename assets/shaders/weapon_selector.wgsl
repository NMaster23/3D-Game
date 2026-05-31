#import bevy_ui::ui_vertex_output::UiVertexOutput

struct WeaponSelectorUI {
    selected_weapon: u32,
};

@group(1) @binding(0) var<uniform> material: WeaponSelectorUI;

// A simple signed distance function for a rounded box
fn sd_rounded_box(p: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(p) - size + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    var final_color = vec4<f32>(0.0, 0.0, 0.0, 0.0); // Transparent background

    // Divide the width of 1.0 into 3 segments
    for (var i: u32 = 1u; i <= 3u; i++) {
        let slot_width = 1.0 / 3.0;
        let center_x = (f32(i) - 0.5) * slot_width;
        let center = vec2<f32>(center_x, 0.5);
        
        let p = uv - center;
        let box_size = vec2<f32>(slot_width * 0.4, 0.35); // Shape of the slots
        
        // Calculate distance and smooth the edges
        let dist = sd_rounded_box(p, box_size, 0.05);
        let alpha = 1.0 - smoothstep(0.0, 0.015, dist);
        
        var slot_color = vec4<f32>(0.2, 0.2, 0.2, 0.6); // Dark gray unselected
        
        // Highlight the currently selected weapon slot
        if i == material.selected_weapon {
            slot_color = vec4<f32>(1.0, 0.8, 0.1, 0.9); // Gold
            
            // Add a white outline
            let border_alpha = 1.0 - smoothstep(0.0, 0.015, abs(dist));
            slot_color = mix(slot_color, vec4<f32>(1.0, 1.0, 1.0, 1.0), border_alpha);
        }
        
        final_color = mix(final_color, slot_color, alpha);
    }
    
    return final_color;
}
