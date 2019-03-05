pub struct PrimitiveAssembly {
    pub topology: Topology,
    pub winding_order: WindingOrder,
    pub face_culling: CullingMode
}


pub enum Topology {
    Point,
    Line,
    Triangle,
    LineStrip,
    LineLoop,
    TriangleStrip,
    TriangleFan
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
enum WindingOrder {
    ClockWise,
    CounterClockwise
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
enum CullingMode {
    None,
    Front,
    Back,
    Both
}
