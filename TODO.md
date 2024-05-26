# Todo
- [ ] Add support for glyph (text) rendering.
  - Look into using [`glyph_brush`](https://crates.io/crates/glyph_brush).
  - 1.2M downloads; seems reliable.
- [ ] Add support for matrix transformations.
  - E.g., rotations, translations, etc.
- [ ] Name all descriptors in the `wgpu` structs to something helpful.
  - Right now, just defaulting them to `None`.

# Done
- [x] Add rendering using index buffers.
  - Right now, everything is being rendered using plain vertex buffers, but that causes a lot of repetition of vertices.
- [x] Add support to render any arbitrary shape.
  - Should be tessellated via the [`lyon`](https://crates.io/crates/lyon) crate.
  - 1.5M downloads; seems reliable as well.
- [x] Add proper error-handling to `metallic`.
  - Currently, just passing an `anyhow!("...")` to propogate errors up.
