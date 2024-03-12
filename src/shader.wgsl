@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    var pos = array(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.5, 0.0),
        vec2<f32>(0.0, 0.5),
    );

    return vec4<f32>(pos[index], 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
