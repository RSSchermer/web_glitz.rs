use std::convert::TryFrom;
use std::ops::Deref;

use web_sys::WebGl2RenderingContext as Gl;

use crate::runtime::state::ContextUpdate;
use crate::runtime::Connection;

/// Enumerates the algorithms available for assembling a stream of vertices into a stream of
/// primitives.
#[derive(Clone, PartialEq, Debug)]
pub enum PrimitiveAssembly {
    /// Assembles the vertices into point primitives, where every vertex defined 1 point.
    Points,

    /// The stream of vertices is assembled into lines.
    ///
    /// The stream of vertices is assembled into lines by partitioning the vertex sequence into
    /// pairs: the first line is described by vertices `0` and `1`; the second line is described by
    /// vertices `2` and `3`; etc.
    ///
    /// The width of the line is defined by the given [LineWidth].
    Lines(LineWidth),

    /// The stream of vertices is assembled into lines.
    ///
    /// Suppose `v0` is the first vertex in the sequence, `v1` is the second vertex in the sequence,
    /// `v2` is the third vertex in the sequence, etc. The first line is then described by `v0` and
    /// `v1`, the second line is described by `v1` and `v2`, the third line is described by `v2` and
    /// `v3`, etc.
    ///
    /// Note that `v1` is both the "end" vertex for the first line, as well the "start" vertex for
    /// the second line; `v2` is both the "end" vertex for the second line, as well the "start"
    /// vertex for the third line. In general: for each line in a line-strip the "start" vertex is
    /// equal to the preceding line's "end" vertex. The first line is an exception as it has no
    /// preceding line segment.
    ///
    /// The width of the line is defined by the given [LineWidth].
    LineStrip(LineWidth),

    /// The stream of vertices is assembled into lines.
    ///
    /// Suppose `v0` is the first vertex in the sequence, `v1` is the second vertex in the sequence,
    /// ..., `vn` is the last vertex in the sequence. The first line is then described by `v0` and
    /// `v1`, the second line is described by `v1` and `v2`, the third line is described by `v2` and
    /// `v3`, ..., the last line is described by `vn` and `v0`.
    ///
    /// Note that `v1` is both the "end" vertex for the first line, as well the "start" vertex for
    /// the second line; `v2` is both the "end" vertex for the second line, as well the "start"
    /// vertex for the third line. `v0` is special: it is both the "start" vertex of the first line,
    /// as well as the "end" vertex of the last line, thus closing the loop.
    ///
    /// The width of the line is defined by the given [LineWidth].
    LineLoop(LineWidth),

    /// The stream of vertices is assembled into triangles.
    ///
    /// The stream of vertices is assembled into triangles by partitioning the vertex sequence into
    /// groups of 3: the first triangle is described by vertices `0`, `1` and `1`; the second
    /// triangle is described by vertices `3`, `4` and `5`; etc.
    Triangles {
        /// The winding order used to assemble the triangles.
        ///
        /// See [WindingOrder] for details.
        winding_order: WindingOrder,

        /// The culling mode applied to the assembled triangles.
        ///
        /// See [CullingMode] for details.
        face_culling: CullingMode,
    },

    /// The stream of vertices is combined into triangless.
    ///
    /// Suppose `v0` is the first vertex in the sequence, `v1` is the second vertex in the sequence,
    /// `v2` is the third vertex in the sequence, etc. A triangle strip will then combine this
    /// vertex sequence into the following triangle sequence:
    ///
    /// - Triangle `0`: `v0` -> `v1` -> `v2`
    /// - Triangle `1`: `v2` -> `v1` -> `v3`
    /// - Triangle `2`: `v2` -> `v3` -> `v4`
    /// - Triangle `3`: `v4` -> `v3` -> `v5`
    /// - Triangle `4`: `v4` -> `v5` -> `v6`
    /// - Triangle `5`: `v6` -> `v5` -> `v7`
    /// - Triangle `6`: ...
    ///
    /// The following diagram shows the connectedness of the vertices in a triangle strip:
    ///
    /// ```
    /// // v1---v3---v5---v7
    /// // |\   |\   |\   |
    /// // | \  | \  | \  |
    /// // |  \ |  \ |  \ |
    /// // |   \|   \|   \|
    /// // v0---v2---v4---v6
    /// ```
    TriangleStrip {
        /// The winding order used to assemble the triangles.
        ///
        /// See [WindingOrder] for details.
        winding_order: WindingOrder,

        /// The culling mode applied to the assembled triangles.
        ///
        /// See [CullingMode] for details.
        face_culling: CullingMode,
    },

    /// The stream of vertices is combined into triangless.
    ///
    /// Suppose `v0` is the first vertex in the sequence, `v1` is the second vertex in the sequence,
    /// `v2` is the third vertex in the sequence, etc. A triangle strip will then combine this
    /// vertex sequence into the following triangle sequence:
    ///
    /// - Triangle `0`: `v0` -> `v1` -> `v2`
    /// - Triangle `1`: `v0` -> `v2` -> `v3`
    /// - Triangle `2`: `v0` -> `v3` -> `v4`
    /// - Triangle `3`: `v0` -> `v4` -> `v5`
    /// - Triangle `4`: `v0` -> `v5` -> `v6`
    /// - Triangle `5`: ...
    ///
    /// This diagram shows the connectedness of the vertices a triangle fan:
    ///
    /// ```
    /// //      v4--------v3
    /// //     / \       / \
    /// //    /   \     /   \
    /// //   /     \   /     \
    /// //  /       \ /       \
    /// // v5--------v0-------v2
    /// //  \       / \       /
    /// //   \     /   \     /
    /// //    \   /     \   /
    /// //     \ /       \ /
    /// //      v6        v1
    /// ```
    TriangleFan {
        /// The winding order used to assemble the triangles.
        ///
        /// See [WindingOrder] for details.
        winding_order: WindingOrder,

        /// The culling mode applied to the assembled triangles.
        ///
        /// See [CullingMode] for details.
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
/// use std::convert::TryFrom;
/// use web_glitz::pipeline::graphics::LineWidth;
///
/// let line_width = LineWidth::try_from(2.0).unwrap();
/// ```
///
/// The value must not be negative or [f32::NAN], otherwise [InvalidLineWidth] is returned.
///
/// A [LineWidth] may be instantiated with the default value through [Default]:
///
/// ```
/// use std::convert::TryFrom;
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
#[derive(Debug)]
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
