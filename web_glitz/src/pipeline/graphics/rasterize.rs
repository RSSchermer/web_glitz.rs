/// Enumerates the test functions that may be used with [DepthTest] and [StencilTest].
pub enum TestFunction {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    NeverPass,
    AlwaysPass
}

/// Provides instructions on how depth testing should be performed.
///
/// In order to do a depth test, the [RenderTarget] that is being rendered to must include a depth
/// buffer. A depth buffer stores a depth value between `0.0` (close) and `1.0` (far) for each
/// fragment. Initially the depth value for each fragment is set to `1.0`. When rendering to a
/// [RenderTarget] with no depth buffer attached, the depth test behaves as though depth testing is
/// disabled.
///
/// The depth test is performed for each fragment. The fragment's depth output will be mapped onto
/// the range defined by [depth_range]:
///
/// - Values smaller than the lower bound will be mapped to `0.0`.
/// - Values greater than the upper bound will be mapped to `1.0`.
/// - Values contained on the range will be mapped to the range `0.0..1.0`.
///
/// The lower bound of the [depth_range] must not be smaller than `0.0`, the upper bound of the
/// [depth_range] must not be greater than `1.0`, and the lower bound must be strictly smaller than
/// the upper bound.
///
/// The resulting depth value may then be compared to the depth buffer's current depth value for
/// this fragment using the [test]. The [test] can be one of the following functions:
///
/// - [TestFunction::Equal]: the test passes if the fragment's depth value is equal to the depth
///   buffer's current depth value for this fragment.
/// - [TestFunction::NotEqual]: the test passes if the fragment's depth value is not equal to the
///   depth buffer's current depth value for this fragment.
/// - [TestFunction::Less]: the test passes if the fragment's depth value is smaller than the depth
///   buffer's current depth value for this fragment.
/// - [TestFunction::Greater]: the test passes if the fragment's depth value is greater than the
///   depth buffer's current depth value for this fragment.
/// - [TestFunction::LessOrEqual]: the test passes if the fragment's depth value is smaller than or
///   equal to the depth buffer's current depth value for this fragment.
/// - [TestFunction::GreaterOrEqual]: the test passes if the fragment's depth value is greater than
///   or equal to the depth buffer's current depth value for this fragment.
/// - [TestFunction::NeverPass]: the test never passes, regardless of how the fragment's depth value
///   compares to the depth buffer's current depth value for this fragment.
/// - [TestFunction::AlwaysPass]: the test always passes, regardless of how the fragment's depth
///   value compares to depth buffer's current depth value for this fragment.
///
/// If the test fails, the fragment will be discarded: none of the [RenderTarget]'s output buffers
/// will be updated. If the test passes and [write] is `true`, then the depth buffer's depth value
/// for this fragment will be replaced with the new depth value; if [write] is `false`, then the
/// depth buffer will not be updated. If the test passes, the fragment will not be discarded by the
/// depth test, but note that other stages of the pipeline (such as front/back-face culling, stencil
/// testing, the fragment shader) may still discard the fragment.
///
/// An instance of for the default depth test options may be obtained via [Default]:
///
/// ```
/// assert_eq!(DepthTest::default(), DepthTest {
///     test: TestFunction::Less,
///     write: true,
///     depth_range: 0.0..=1.0,
///     polygon_offset: None
/// });
/// ```
pub struct DepthTest {
    /// The [TestFunction] used to decide if the [DepthTest] passes or fails.
    ///
    /// Defaults to [TestFunction::Less].
    pub test: TestFunction,

    /// Whether or not the depth buffer will be updated when the depth test passes.
    ///
    /// When set to `false`, the depth buffer will not be updated when the depth test passes.
    ///
    /// Defaults to `true`.
    pub write: bool,

    /// Defines how a fragment's depth output will be mapped onto the range `0.0..1.0` from the near
    /// plane at `0.0` to the far plane at `1.0`.
    ///
    /// See [DepthRange] for details.
    pub depth_range: DepthRange,

    /// Defines an optional polygon-offset to be applied to depth fragments optioned from polygonal
    /// primitives (triangles).
    ///
    /// See [PolygonOffset] for details.
    ///
    /// Defaults to `None`, in which case no polygon offset will be applied.
    pub polygon_offset: Option<PolygonOffset>
}

impl Default for DepthTest {
    fn default() -> Self {
        DepthTest {
            test: TestFunction::Less,
            write: true,
            depth_range: DepthRange::default(),
            polygon_offset: None
        }
    }
}

/// Defines how a fragment's depth output will be mapped onto the range `0.0..1.0` from the near
/// plane at `0.0` to the far plane at `1.0`.
///
/// - Values smaller than the lower bound will be mapped to `0.0`.
/// - Values greater than the upper bound will be mapped to `1.0`.
/// - Values contained on the range will be mapped to the range `0.0..1.0`.
///
/// Can be constructed from a [RangeInclusive<f32>] via [TryFrom]:
///
/// ```
/// let depth_range = DepthRange::try_from(0.2..=0.8)?;
/// ```
///
/// The lower bound of the range must not be smaller than 0.0, the upper bound of the range must not
/// be greater than `1.0`, and the lower bound must be strictly smaller than the upper bound;
/// otherwise, an [InvalidDepthRange] error is returned.
///
/// A default depth range may be obtained through [Default]:
///
/// ```
/// assert_eq!(DepthRange::default(), DepthRange::try_from(0.0..=1.0).unwrap());
/// ```
pub struct DepthRange {
    range_near: f32,
    range_far: f32
}

impl DepthRange {
    /// The lower bound of this range that will be mapped onto the near plane at `0.0`.
    fn range_near(&self) -> f32 {
        self.range_near
    }

    /// The upper bound of this range that will be mapped onto the far plane at `1.0`.
    fn range_far(&self) -> f32 {
        self.range_far
    }
}

impl Default for DepthRange {
    fn default() -> Self {
        DepthRange {
            range_near: 0.0,
            range_far: 1.0
        }
    }
}

/// Error returned when trying to construct a [DepthRange] instance from an invalid
/// [RangeInclusive<f32>].
///
/// The lower bound of the range must not be smaller than 0.0, the upper bound of the range must not
/// be greater than `1.0`, and the lower bound must be strictly smaller than the upper bound.
pub struct InvalidDepthRange(RangeInclusive<f32>);

/// Specifies an offset modifier for for depth values from polygonal fragments.
///
/// When a [DepthTest] specifies a polygon offset, then an offset is applied to depth values for
/// fragments from polygonal primitives (triangles) before they are tested and before the depth
/// value is written to the depth buffer. This offset is computed as `factor * DZ + r * units`,
/// where `factor` is the [factor] specified by the [PolygonOffset] instance, `DZ` is a measurement
/// of the fragment's rate of change in depth relative to the screen area of the polygon, `units` is
/// the [units] value specified by the [PolygonOffset] instance, and `r` is the smallest value that
/// is guaranteed to produce a resolvable offset for a given implementation.
///
/// A polygon offset is useful when rendering coplanar primitives. Can be used to prevent what is
/// called "stitching", "bleeding" or "Z-fighting", where fragments with very similar z-values do
/// not always produce predictable results.
///
/// Only applies to fragments from polygonal primitives (triangles); ignored for fragments from
/// other primitives (points, lines). If you are rendering e.g. a coplanar triangle and line,
/// specify a polygon offset to "push back" the triangle, rather than attempting to "push forward"
/// the line.
#[derive()]
pub struct PolygonOffset {
    /// A scaling factor applied as part of a [PolygonOffset] modifier.
    ///
    /// See the type documentation for [PolygonOffset] for details.
    pub factor: f32,

    /// A constant offset in units applied as part of a [PolygonOffset] modifier.
    ///
    /// See the type documentation for [PolygonOffset] for details.
    pub units: f32,
}
