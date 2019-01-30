pub struct Rasterizer {
    pub depth_test: Option<DepthTest>,
    pub stencil_test: Option<StencilTest>,
    pub viewport: Viewport,
    pub scissor: Option<Region2D>,
    pub front_face: WindingOrder,
    pub face_culling: CullingMode,
    pub line_width: f32
}

impl Default for Rasterizer {
    fn default() -> Rasterizer {
        Rasterizer {
            depth_test: None,
            stencil_test: None,
            viewport: Viewport::default(),
            scissor: None,
            front_face: WindingOrder::default(),
            face_culling: CullingMode::default(),
            line_width: 1.0
        }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
enum WindingOrder {
    ClockWise,
    CounterClockwise
}

impl Default for WindingOrder {
    fn default() -> WindingOrder {
        WindingOrder::CounterClockwise
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
enum CullingMode {
    None,
    Front,
    Back,
    Both
}

impl Default for CullingMode {
    fn default() -> CullingMode {
        CullingMode::None
    }
}

