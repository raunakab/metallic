struct VertexIn {
    @location(0) vertex: vec2f,
    @location(1) color: vec4f,
}

struct VertexOut {
    @builtin(position) position: vec4f,
    @location(1) color: vec4f,
}

struct FragmentOut {
    @location(0) color: vec4f,
}

@vertex
fn vs(
    vertex_in: VertexIn,
) -> VertexOut {
    var vertex_out: VertexOut;
    vertex_out.position = vec4f(vertex_in.vertex.xy, 0.0, 1.0);
    vertex_out.color = vertex_in.color;
    return vertex_out;
}

@fragment
fn fs(
    vertex_out: VertexOut,
) -> FragmentOut {
    var fragment_out: FragmentOut;
    fragment_out.color = vertex_out.color;
    return fragment_out;
}
