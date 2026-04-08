#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

struct EnergyTrailParams {
    edge_color: vec4<f32>,
    core_color: vec4<f32>,
    pulse_speed: f32,
    edge_sharpness: f32,
    _padding: vec2<f32>,
}

@group(2) @binding(100)
var<uniform> energy: EnergyTrailParams;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // Use vertex UV.y (0 at left edge, 1 at right edge) for cross-section gradient.
    // Remap to center: 0 at edges, 1 at center.
    let cross_t = 1.0 - abs(in.uv.y * 2.0 - 1.0);

    // Sharp core with soft edges
    let core_mask = pow(cross_t, energy.edge_sharpness);

    // Mix edge and core colors
    let trail_color = mix(energy.edge_color, energy.core_color, core_mask);

    // Apply vertex color alpha (from the trail system's curve evaluation)
    let vertex_alpha = pbr_input.material.base_color.a;
    let final_alpha = trail_color.a * vertex_alpha;

    // Emissive glow based on core intensity
    let emissive_strength = core_mask * 2.0;

    var out_color = vec4<f32>(
        trail_color.rgb * (1.0 + emissive_strength),
        final_alpha,
    );

    out_color = alpha_discard(pbr_input.material, out_color);
    return out_color;
}
