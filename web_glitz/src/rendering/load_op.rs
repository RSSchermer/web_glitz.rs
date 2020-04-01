use web_sys::WebGl2RenderingContext as Gl;

#[derive(Clone, Copy, PartialEq)]
pub enum LoadOp<T> {
    Load,
    Clear(T),
}

impl LoadOp<[f32; 4]> {
    pub(crate) fn as_load_float_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorFloat(index, *value),
        }
    }
}

impl LoadOp<[i32; 4]> {
    pub(crate) fn as_load_integer_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorInteger(index, *value),
        }
    }
}

impl LoadOp<[u32; 4]> {
    pub(crate) fn as_load_unsigned_integer_action(&self, index: i32) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(value) => LoadAction::ClearColorUnsignedInteger(index, *value),
        }
    }
}

impl LoadOp<(f32, i32)> {
    pub(crate) fn as_load_depth_stencil_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear((depth, stencil)) => LoadAction::ClearDepthStencil(*depth, *stencil),
        }
    }
}

impl LoadOp<f32> {
    pub(crate) fn as_load_depth_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(depth) => LoadAction::ClearDepth(*depth),
        }
    }
}

impl LoadOp<i32> {
    pub(crate) fn as_load_stencil_action(&self) -> LoadAction {
        match self {
            LoadOp::Load => LoadAction::Load,
            LoadOp::Clear(stencil) => LoadAction::ClearStencil(*stencil),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum LoadAction {
    Load,
    ClearColorFloat(i32, [f32; 4]),
    ClearColorInteger(i32, [i32; 4]),
    ClearColorUnsignedInteger(i32, [u32; 4]),
    ClearDepthStencil(f32, i32),
    ClearDepth(f32),
    ClearStencil(i32),
}

impl LoadAction {
    pub(crate) fn perform(&self, gl: &Gl) {
        match self {
            LoadAction::Load => (),
            LoadAction::ClearColorFloat(index, value) => {
                gl.clear_bufferfv_with_f32_array(Gl::COLOR, *index, value)
            }
            LoadAction::ClearColorInteger(index, value) => {
                gl.clear_bufferiv_with_i32_array(Gl::COLOR, *index, value)
            }
            LoadAction::ClearColorUnsignedInteger(index, value) => {
                gl.clear_bufferuiv_with_u32_array(Gl::COLOR, *index, value)
            }
            LoadAction::ClearDepthStencil(depth, stencil) => {
                gl.clear_bufferfi(Gl::DEPTH_STENCIL, 0, *depth, *stencil)
            }
            LoadAction::ClearDepth(value) => {
                gl.clear_bufferfv_with_f32_array(Gl::DEPTH, 0, &mut [*value])
            }
            LoadAction::ClearStencil(value) => {
                gl.clear_bufferiv_with_i32_array(Gl::STENCIL, 0, &mut [*value])
            }
        }
    }
}
