
pub struct PipelineState {
    vertex_shader: Shader,
    primitive_assembly: PrimitiveAssembly,
    transform_feedback: Option<TransformFeedback>,
    rasterizer: Option<Rasterizer>,
    scissor_test: Option<ScissorTest>,
    depth_test: Option<DepthTest>,
    stencil_test: Option<StencilTest>,
    fragment_shader: Option<Shader>,
    blending: Option<Blending>,
    output: ColorMask
}

impl PipelineState {

}

pub struct RasterizerState {
    viewport: Viewport,
    scissor: Option<Scissor>,
    front_face: WindingOrder,
    face_culling: CullingMode,
    line_width: f32
}

pub struct Viewport {
    x: i32,
    y: i32,
    width: u32,
    height: u32
}

pub struct Scissor {
    x: i32,
    y: i32,
    width: u32,
    height: u32
}

struct PipelineStateBuilder {

}

struct VertexShaderStage {

}

struct TransformFeedbackStage {

}

struct RasterizerStage {

}

struct DepthTestStage {

}

struct StencilTestStage {

}

struct FragmentShaderStage {

}

struct BlendingStage {

}