use std::convert::TryFrom;
use std::ops::Deref;

use web_sys::WebGl2RenderingContext as Gl;

use crate::runtime::state::ContextUpdate;
use crate::runtime::Connection;

#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveAssembly {
    Points,
    Lines(LineWidth),
    LineStrip(LineWidth),
    LineLoop(LineWidth),
    Triangles {
        winding_order: WindingOrder,
        face_culling: CullingMode,
    },
    TriangleStrip {
        winding_order: WindingOrder,
        face_culling: CullingMode,
    },
    TriangleFan {
        winding_order: WindingOrder,
        face_culling: CullingMode,
    },
}

impl PrimitiveAssembly {
    pub(crate) fn topology(&self) -> Topology {
        match self {
            PrimitiveAssembly::Points => Topology::Point,
            PrimitiveAssembly::Lines(_) => Topology::Line,
            PrimitiveAssembly::LineStrip(_) => Topology::LineStrip,
            PrimitiveAssembly::LineLoop(_) => Topology::LineLoop,
            PrimitiveAssembly::Triangles { .. } => Topology::Triangle,
            PrimitiveAssembly::TriangleStrip { .. } => Topology::TriangleStrip,
            PrimitiveAssembly::TriangleFan { .. } => Topology::TriangleFan,
        }
    }

    pub(crate) fn line_width(&self) -> Option<LineWidth> {
        match self {
            PrimitiveAssembly::Lines(line_width) => Some(*line_width),
            PrimitiveAssembly::LineStrip(line_width) => Some(*line_width),
            PrimitiveAssembly::LineLoop(line_width) => Some(*line_width),
            _ => None,
        }
    }

    pub(crate) fn face_culling(&self) -> Option<CullingMode> {
        match self {
            PrimitiveAssembly::Triangles { face_culling, .. } => Some(*face_culling),
            PrimitiveAssembly::TriangleStrip { face_culling, .. } => Some(*face_culling),
            PrimitiveAssembly::TriangleFan { face_culling, .. } => Some(*face_culling),
            _ => None,
        }
    }

    pub(crate) fn winding_order(&self) -> Option<WindingOrder> {
        match self {
            PrimitiveAssembly::Triangles { winding_order, .. } => Some(*winding_order),
            PrimitiveAssembly::TriangleStrip { winding_order, .. } => Some(*winding_order),
            PrimitiveAssembly::TriangleFan { winding_order, .. } => Some(*winding_order),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Topology {
    Point,
    Line,
    Triangle,
    LineStrip,
    LineLoop,
    TriangleStrip,
    TriangleFan,
}

impl Topology {
    pub(crate) fn id(&self) -> u32 {
        match self {
            Topology::Point => Gl::POINTS,
            Topology::Line => Gl::LINES,
            Topology::Triangle => Gl::TRIANGLES,
            Topology::LineStrip => Gl::LINE_STRIP,
            Topology::LineLoop => Gl::LINE_LOOP,
            Topology::TriangleStrip => Gl::TRIANGLE_STRIP,
            Topology::TriangleFan => Gl::TRIANGLE_FAN,
        }
    }
}

/// Defines the line width used by a [Rasterizer].
///
/// Can be constructed from an `f32` via [TryFrom]:
///
/// ```
/// use web_glitz::pipeline::graphics::LineWidth;
///
/// let line_width = LineWidth::try_from(2.0)?;
/// ```
///
/// The value must not be negative or [f32::NAN], otherwise [InvalidLineWidth] is returned.
///
/// A [LineWidth] may be instantiated with the default value through [Default]:
///
/// ```
/// use web_glitz::pipeline::graphics::LineWidth;
///
/// assert_eq!(LineWidth::default(), LineWidth::try_from(1.0).unwrap());
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LineWidth {
    value: f32,
}

impl LineWidth {
    pub(crate) fn apply(&self, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state.set_line_width(self.value).apply(gl).unwrap();
    }
}

impl TryFrom<f32> for LineWidth {
    type Error = InvalidLineWidth;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value == std::f32::NAN {
            Err(InvalidLineWidth::NaN)
        } else if value < 0.0 {
            Err(InvalidLineWidth::Negative)
        } else {
            Ok(LineWidth { value })
        }
    }
}

impl Default for LineWidth {
    fn default() -> Self {
        LineWidth { value: 1.0 }
    }
}

impl Deref for LineWidth {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Error returned when trying to construct a [LineWidth] from an invalid value.
pub enum InvalidLineWidth {
    NaN,
    Negative,
}

/// Enumerates the possible winding orders for triangles that may be used by a [Rasterizer].
///
/// A triangle is considered to have 2 sides or 'faces': a front-face and a back-face. The winding
/// order determines which face is considered the front-face, and thus by extension which face is
/// considered the back-face.
///
/// Each triangle is defined by 3 points: a first point (point `a`), a second point (point `b`), and
/// a third point (point `c`):
///
/// ```
/// //        a
/// //       /\
/// //      /  \
/// //     /    \
/// //    /      \
/// //   /________\
/// //  c          b
/// ```
///
/// In the above example we are looking at just one face of a triangle: the face that is facing
/// toward us. If we trace the outline of this triangle from `a -> b -> c -> a`, we'll notice that
/// we've followed a clockwise path. If the winding order is defined to be
/// [WindingOrder::Clockwise], then we are looking at the front-face of this triangle. If the
/// winding order is defined to be [WindingOrder::CounterClockwise], then we are looking at the
/// back-face of this triangle.
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

impl WindingOrder {
    pub(crate) fn apply(&self, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state.set_front_face(*self).apply(gl).unwrap();
    }
}

/// Enumerates the face-culling modes that may be used by a [Rasterizer].
///
/// A triangle is considered to have 2 sides or 'faces': a front-face and a back-face. Which face is
/// considered to be the front-face and which face is considered to be the back-face, is determined
/// by the [WindingOrder] (see the documentation for [WindingOrder] for details).
///
/// Triangles may be discarded based on their facing in a process known as face-culling. A triangle
/// is considered front-facing if it is oriented such that the front-face is facing the 'camera'. A
/// triangle is considered back-facing if it is oriented such that the back-face is facing the
/// camera.
///
/// There are 4 possible culling modes:
///
/// - [CullingMode::None]: no faces will be culled, regardless or their facing.
/// - [CullingMode::Both]: all triangles will be culled, regardless of their facing.
/// - [CullingMode::Front]: front-facing triangles will be culled.
/// - [CullingMode::Back]: back-facing triangles will be culled.
///
/// Face culling is an optimization typically used when rendering closed surfaces. It allows the
/// rasterizer to discard triangles that would not have been visible anyway, before the expensive
/// rasterization and fragment shader operations are performed.
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub enum CullingMode {
    None,
    Front,
    Back,
    Both,
}

impl CullingMode {
    pub(crate) fn apply(&self, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        state.set_cull_face(*self).apply(gl).unwrap();
    }
}
