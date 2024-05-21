struct Out {
    @builtin(position) position: vec4f,
    @location(1) color: vec4f,
}

@vertex
fn vs(
    @location(0) vertex: vec2f,
    @location(1) color: vec4f,
) -> Out {
    var out: Out;
    out.position = vec4f(vertex, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs(
    out: Out,
) -> @location(0) vec4f {
    return out.color;
}
