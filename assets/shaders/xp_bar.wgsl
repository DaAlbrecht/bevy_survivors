#import bevy_ui::ui_vertex_output::UiVertexOutput


struct XpBarMaterial {
    foreground_color: vec4<f32>,
    background_color: vec4<f32>,
    percent: f32,
};

@group(1) @binding(0) var<uniform> material: XpBarMaterial;


@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    var out: vec4<f32>;

    if in.uv.x <= material.percent {
        out = material.foreground_color;
    } else {
        out = material.background_color;
    }
    return out;
}
