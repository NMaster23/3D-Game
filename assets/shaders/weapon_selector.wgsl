#import bevy_ui::ui_vertex_output::UiVertexOutput

struct WeaponSelectorUI {
    selected_weapon: u32,
};

@group(1) @binding(0) var<uniform> material: WeaponSelectorUI;
@group(1) @binding(1) var weapon1_tex: texture_2d<f32>;
@group(1) @binding(2) var weapon1_samp: sampler;
@group(1) @binding(3) var weapon2_tex: texture_2d<f32>;
@group(1) @binding(4) var weapon2_samp: sampler;
@group(1) @binding(5) var weapon3_tex: texture_2d<f32>;
@group(1) @binding(6) var weapon3_samp: sampler;

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
        
        // Aspect ratio correction (250px height / 1000px width = 0.25)
        let aspect = 0.25;
        
        // Apply aspect ratio to our local coordinates so the SDF isn't warped
        let p = (uv - center) * vec2<f32>(1.0, aspect);
        let box_size = vec2<f32>(slot_width * 0.4, 0.35 * aspect); 
        
        // Calculate distance and smooth the edges using aspect-corrected math
        let dist = sd_rounded_box(p, box_size, 0.05 * aspect);
        let alpha = 1.0 - smoothstep(0.0, 0.002, dist);
        
        var slot_color = vec4<f32>(0.2, 0.2, 0.2, 0.6); // Dark gray unselected
        
        // Highlight the currently selected weapon slot
        if i == material.selected_weapon {
            slot_color = vec4<f32>(1.0, 0.8, 0.1, 0.9); // Gold
            
            // Add a crisp white outline
            let border_alpha = 1.0 - smoothstep(0.0, 0.002, abs(dist));
            slot_color = mix(slot_color, vec4<f32>(1.0, 1.0, 1.0, 1.0), border_alpha);
        }
        
        // --- Draw the weapon image ---
        // Map local box boundaries to texture UV space (0.0 to 1.0)
        // Use the original un-aspect-corrected ratios here to avoid stretching the image
        let tex_uv = ((uv - center) / vec2<f32>(slot_width * 0.4, 0.35)) * 0.5 + vec2<f32>(0.5);
        
        // Check if we are inside the box boundaries to prevent texture stretching
        if tex_uv.x >= 0.0 && tex_uv.x <= 1.0 && tex_uv.y >= 0.0 && tex_uv.y <= 1.0 {
            var tex_color = vec4<f32>(0.0);
            if i == 1u {
                tex_color = textureSampleLevel(weapon1_tex, weapon1_samp, tex_uv, 0.0);
            } else if i == 2u {
                tex_color = textureSampleLevel(weapon2_tex, weapon2_samp, tex_uv, 0.0);
            } else if i == 3u {
                tex_color = textureSampleLevel(weapon3_tex, weapon3_samp, tex_uv, 0.0);
            }
            
            // Blend the image over the slot background (assuming the image has transparency)
            slot_color = mix(slot_color, tex_color, tex_color.a);
        }
        
        final_color = mix(final_color, slot_color, alpha);
    }
    
    return final_color;
}
