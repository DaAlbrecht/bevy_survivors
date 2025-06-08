#import bevy_ui::ui_vertex_output::UiVertexOutput


struct XpBarMaterial {
    filled_color: vec4<f32>,
    background_color: vec4<f32>,
    factor: f32,
    border_color: vec4<f32>,
    border_radius: vec4<f32>,
    offset: vec4<f32>,
};

@group(1) @binding(0) var<uniform> input: XpBarMaterial;

// https://gist.github.com/munrocket/30e645d584b5300ee69295e54674b3e4
fn sdf_rounded_rect(p: vec2f, b: vec2f, r: vec4f) -> f32 {
    var x = r.x;
    var y = r.y;
    x = select(r.z, r.x, p.x > 0.);
    y = select(r.w, r.y, p.x > 0.);
    x = select(y, x, p.y > 0.);
    let q = abs(p) - b + x;
    return min(max(q.x, q.y), 0.) + length(max(q, vec2f(0.))) - x;
}


@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * in.size * 2.0 - in.size;

    // position offset to account for border
    let border_offset = vec2<f32>(
        input.offset.w - input.offset.y, // right - left
        input.offset.z - input.offset.x, // bottom - top
    );

    // SDF distance in the inner button area
    // The inner button size is equal to actual size - offset size
    let size = in.size - vec2<f32>(
        input.offset.y + input.offset.w, // left + right
        input.offset.x + input.offset.z, // top + bottom
    );
    let d_shape = sdf_rounded_rect(
        uv + border_offset,
        size,
        input.border_radius,
    );

    // SDF distance in border area
    let d_border = sdf_rounded_rect(uv, in.size, input.border_radius);

    // define the alpha value. Opaque if within the button or border area,
    // transparent otherwise.
    let alpha = select(1., 0., (d_shape > 0. && d_border > 0.));
    // define the final color. Use `input.border_color` if within border
    // radius, otherwise `input.background_color`.

    var xp_color: vec4<f32>;
    if in.uv.x <= input.factor {
        xp_color = input.filled_color;
    } else {
        xp_color = input.background_color;
    }

    let color = select(
        xp_color,
        input.border_color,
        (d_shape > 0. && d_border <= 0.),
    );


    return vec4<f32>(color.rgb, alpha * color.a);
}
