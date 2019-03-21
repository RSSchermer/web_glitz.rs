use crate::runtime::Connection;

/// Describes the viewport used by by a [GraphicsPipeline].
///
/// The viewport defines the affine transformation of `X` and `Y` from normalized device coordinates
/// to window coordinates.
///
/// Given a viewport with origin `(X_vp, Y_vp)`, width `width` and height `height`; let `X_nd` and
/// `Y_nd` be normalized device coordinates, then the corresponding window coordinates `X_w` and
/// `Y_w` are computed as:
///
/// `X_w = (X_nd + 1) * 0.5 * width + X_vp`
///
/// `Y_w = (X_nd + 1) * 0.5 * height + Y_vp`
///
/// There are two ways to define a [Viewport]:
///
/// 1. With explicit values:
///    ```
///    let viewport = Viewport::Region((X_vp, Y_vp), width, height);
///    ```
///    Where `X_vp`, `Y_vp`, `width` and `height` correspond to the values used in the description
///    above.
///
/// 2. With automatic values:
///    ```
///    let viewport = Viewport::Auto;
///    ```
///    In this case, the origin will always be `(0, 0)`; determination of the `width` and `height`
///    is deferred until draw time, where they taken to be the width and height of the
///    [RenderTarget] that is being drawn to, such that the viewport will cover the [RenderTarget]
///    exactly. Note that the width and height of the [RenderTarget] are determined by the attached
///    images with the smallest width and height respectively.
///
#[derive(Clone, PartialEq, Debug)]
pub enum Viewport {
    Region((i32, i32), u32, u32),
    Auto,
}

impl Viewport {
    pub(crate) fn apply(&self, connection: &mut Connection, auto_dimensions: (u32, u32)) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        let (x, y, width, height) = match self {
            Viewport::Region((x, y), width, height) => (x, y, width, height),
            Viewport::Auto => {
                let (width, height) = auto_dimensions;

                (0, 0, width, height)
            }
        };

        state.set_viewport(*x, *y, *width as i32, *height as i32).apply(gl).unwrap();
    }
}