pub struct Rasterizer {
    pub viewport: Viewport,
    pub scissor: Option<Scissor>,
    pub front_face: WindingOrder,
    pub face_culling: CullingMode,
    pub line_width: f32
}

impl Default for Rasterizer {
    fn default() -> Rasterizer {
        Rasterizer {
            viewport: Viewport::Auto,
            scissor: Scissor::Auto,
            front_face: WindingOrder::CounterClockwise,
            face_culling: CullingMode::None,
            line_width: 1.0
        }
    }
}