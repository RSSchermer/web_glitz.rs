use std::convert::TryFrom;
use std::ops::RangeInclusive;

use web_sys::WebGl2RenderingContext as Gl;

use crate::runtime::state::ContextUpdate;
use crate::runtime::Connection;

/// Enumerates the test functions that may be used with [DepthTest] and [StencilTest].
///
/// See the documentation for [DepthTest] and [StencilTest] for details.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TestFunction {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    NeverPass,
    AlwaysPass,
}

impl TestFunction {
    pub(crate) fn id(&self) -> u32 {
        match self {
            TestFunction::Equal => Gl::EQUAL,
            TestFunction::NotEqual => Gl::NOTEQUAL,
            TestFunction::Less => Gl::LESS,
            TestFunction::Greater => Gl::GREATER,
            TestFunction::LessOrEqual => Gl::LEQUAL,
            TestFunction::GreaterOrEqual => Gl::GEQUAL,
            TestFunction::NeverPass => Gl::NEVER,
            TestFunction::AlwaysPass => Gl::ALWAYS,
        }
    }
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
/// use web_glitz::pipeline::graphics::{DepthTest, TestFunction, DepthRange};
///
/// assert_eq!(DepthTest::default(), DepthTest {
///     test: TestFunction::Less,
///     write: true,
///     depth_range: DepthRange::default(),
///     polygon_offset: None
/// });
/// ```
#[derive(Clone, PartialEq, Debug)]
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
    pub polygon_offset: Option<PolygonOffset>,
}

impl DepthTest {
    pub(crate) fn apply(option: &Option<Self>, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        match option {
            Some(depth_test) => {
                state.set_depth_test_enabled(true).apply(gl).unwrap();
                state.set_depth_func(depth_test.test).apply(gl).unwrap();
                state
                    .set_depth_range(depth_test.depth_range.clone())
                    .apply(gl)
                    .unwrap();
                state.set_depth_mask(depth_test.write).apply(gl).unwrap();

                match &depth_test.polygon_offset {
                    Some(polygon_offset) => {
                        state
                            .set_polygon_offset_fill_enabled(true)
                            .apply(gl)
                            .unwrap();
                        state
                            .set_polygon_offset(polygon_offset.clone())
                            .apply(gl)
                            .unwrap();
                    }
                    _ => state
                        .set_polygon_offset_fill_enabled(false)
                        .apply(gl)
                        .unwrap(),
                }
            }
            _ => state.set_depth_test_enabled(false).apply(gl).unwrap(),
        }
    }
}

impl Default for DepthTest {
    fn default() -> Self {
        DepthTest {
            test: TestFunction::Less,
            write: true,
            depth_range: DepthRange::default(),
            polygon_offset: None,
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
/// use std::convert::TryFrom;
/// use web_glitz::pipeline::graphics::DepthRange;
///
/// let depth_range = DepthRange::try_from(0.2..=0.8).unwrap();
/// ```
///
/// The lower bound of the range must not be smaller than 0.0, the upper bound of the range must not
/// be greater than `1.0`, and the lower bound must be strictly smaller than the upper bound;
/// otherwise, an [InvalidDepthRange] error is returned.
///
/// A default depth range may be obtained through [Default]:
///
/// ```
/// use std::convert::TryFrom;
/// use web_glitz::pipeline::graphics::DepthRange;
///
/// assert_eq!(DepthRange::default(), DepthRange::try_from(0.0..=1.0).unwrap());
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct DepthRange {
    range_near: f32,
    range_far: f32,
}

impl DepthRange {
    /// The lower bound of this range that will be mapped onto the near plane at `0.0`.
    pub fn near(&self) -> f32 {
        self.range_near
    }

    /// The upper bound of this range that will be mapped onto the far plane at `1.0`.
    pub fn far(&self) -> f32 {
        self.range_far
    }
}

impl Default for DepthRange {
    fn default() -> Self {
        DepthRange {
            range_near: 0.0,
            range_far: 1.0,
        }
    }
}

impl TryFrom<RangeInclusive<f32>> for DepthRange {
    type Error = InvalidDepthRange;

    fn try_from(range: RangeInclusive<f32>) -> Result<Self, Self::Error> {
        let near = *range.start();
        let far = *range.end();

        if near < 0.0 || far > 1.0 || near >= far {
            Err(InvalidDepthRange(range))
        } else {
            Ok(DepthRange {
                range_near: near,
                range_far: far,
            })
        }
    }
}

/// Error returned when trying to construct a [DepthRange] instance from an invalid
/// [RangeInclusive<f32>].
///
/// The lower bound of the range must not be smaller than 0.0, the upper bound of the range must not
/// be greater than `1.0`, and the lower bound must be strictly smaller than the upper bound.
#[derive(Debug)]
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
/// A polygon offset is useful when rendering coplanar primitives, such as when rendering
/// hidden-line images, when applying decals to surfaces, and when rendering solids with highlighted
/// edges. Can be used to prevent what is called "stitching", "bleeding" or "Z-fighting", where
/// fragments with very similar z-values do not always produce predictable results.
///
/// Only applies to fragments from polygonal primitives (triangles); ignored for fragments from
/// other primitives (points, lines). If you are rendering e.g. a coplanar triangle and line,
/// specify a polygon offset to "push back" the triangle, rather than attempting to "push forward"
/// the line.
///
/// May be instantiated with default values through [Default]:
///
/// ```
/// use web_glitz::pipeline::graphics::PolygonOffset;
///
/// assert_eq!(PolygonOffset::default(), PolygonOffset {
///     factor: 0.0,
///     units: 0.0,
/// });
/// ```
#[derive(PartialEq, Debug, Clone, Default)]
pub struct PolygonOffset {
    /// Specifies a scale factor that is used to create a variable depth offset for each polygon.
    ///
    /// See the type documentation for [PolygonOffset] for details.
    ///
    /// Defaults to `0.0`.
    pub factor: f32,

    /// Is multiplied by an implementation-specific value to create a constant depth offset.
    ///
    /// See the type documentation for [PolygonOffset] for details.
    ///
    /// Defaults to `0.0`.
    pub units: f32,
}

/// Enumerates the operations that can be performed on a stencil fragment as a result of the
/// [StencilTest].
///
/// See the type documentation for [StencilTest] for details.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum StencilOperation {
    Keep,
    Zero,
    Replace,
    Increment,
    WrappingIncrement,
    Decrement,
    WrappingDecrement,
    Invert,
}

impl StencilOperation {
    pub(crate) fn id(&self) -> u32 {
        match self {
            StencilOperation::Keep => Gl::KEEP,
            StencilOperation::Zero => Gl::ZERO,
            StencilOperation::Replace => Gl::REPLACE,
            StencilOperation::Increment => Gl::INCR,
            StencilOperation::WrappingIncrement => Gl::INCR_WRAP,
            StencilOperation::Decrement => Gl::DECR,
            StencilOperation::WrappingDecrement => Gl::DECR_WRAP,
            StencilOperation::Invert => Gl::INVERT,
        }
    }
}

/// Provides instructions on how stencil testing should be performed.
///
/// In order to do a stencil test, a [RenderTarget] must have a stencil buffer. A stencil buffer
/// stores a stencil value for each fragment as an unsigned integer. This means that for an 8-bit
/// stencil buffer, stencil values can range from `0` (`00000000`) to 255 (`11111111`). When
/// rendering to a [RenderTarget] with no stencil buffer attached, the stencil test behaves as though
/// stencil testing is disabled.
///
/// The stencil test distinguishes between front-facing fragments obtained from the `Front` side of
/// a triangle and back-facing fragments obtained from the `Back` side of a triangle; see
/// [WindingOrder] for details on how the `Front` and `Back` faces are decided. Fragments obtained
/// from non-polygonal primitives (points, lines) are always considered front-facing. Separate
/// configurations are specified for the `Front` and `Back` cases respectively.
///
/// The stencil test is performed for each fragment. [test_function_front] specifies the function
/// that is used to test front-facing fragments; [test_function_back] specifies the function that is
/// used to test back-facing fragments. For each fragment sampled from a primitive (a point, line or
/// triangle) a reference value ([reference_value_front] or [reference_value_back], for front- and
/// back-facing fragments respectively) may be compared to the current `stencil_value` for this
/// fragment in the stencil buffer. A bitmask ([test_mask_front] or [test_mask_back], for front- and
/// back-facing fragments respectively) will be applied to both the reference value and the
/// `stencil_value` with the bitwise `AND` operation (`&`), before the test function is evaluated;
/// this allows masking off certain bits, reserving them for different conditional tests. The
/// following test functions are available:
///
/// - [TestFunction::Equal]: the test passes if `reference_value & test_mask` is equal to
///   `stencil_value & test_mask`.
/// - [TestFunction::NotEqual]: the test passes if `reference_value & test_mask` is not equal to
///   `stencil_value & test_mask`.
/// - [TestFunction::Less]: the test passes if `reference_value & test_mask` is smaller than
///   `stencil_value & test_mask`.
/// - [TestFunction::Greater]: the test passes if `reference_value & test_mask` is greater than
///   `stencil_value & test_mask`.
/// - [TestFunction::LessOrEqual]: the test passes if `reference_value & test_mask` is smaller than
///   or equal to `stencil_value & test_mask`.
/// - [TestFunction::GreaterOrEqual]: the test passes if `reference_value & test_mask` is greater
///   than or equal to `stencil_value & test_mask`.
/// - [TestFunction::NeverPass]: the test never passes, regardless of how
///   `reference_value & test_mask` compares to `stencil_value & test_mask`.
/// - [TestFunction::AlwaysPass]: the test always passes, regardless of how
///   `reference_value & test_mask` compares to `stencil_value & test_mask`.
///
/// Typically, if either the [DepthTest] or [StencilTest] fails, a fragment will be discarded and a
/// [RenderTarget]'s output buffers will not be updated. However, the stencil buffer is exceptional
/// in that it may be updated even if the [DepthTest] or the [StencilTest] fails, or both tests
/// fail. There are 3 relevant cases:
///
/// 1.  The [StencilTest] fails. The [StencilTest] is performed before the [DepthTest]. If the
///     [StencilTest] fails the [DepthTest] will not be performed.
/// 2.  The [StencilTest] passes, but the [DepthTest] fails. If depth testing is disabled, then it
///     is always assumed to pass and this case will never occur.
/// 3.  The [StencilTest] passes and the [DepthTest] passes. If depth testing is disabled, then it
///     is always assumed to pass.
///
/// For each of these 3 cases the update operation that is performed on the stencil buffer can be
/// controlled separately. For the `Front` stencil value this controlled by [fail_operation_front]
/// (case 1), [pass_depth_fail_operation_front] (case 2), and [pass_operation_front] (case 3) for
/// each case respectively. For the `Back` stencil value this is controlled by [fail_operation_back]
/// (case 1), [pass_depth_fail_operation_back] (case 2), and [pass_operation_back] (case 3) for each
/// case respectively. The following stencil update operations are available:
///
/// - [StencilOperation::Keep]: the stencil value in the stencil buffer is not updated.
/// - [StencilOperation::Zero]: the stencil value in the stencil buffer is set to `0`.
/// - [StencilOperation::Replace]: the stencil value in the stencil buffer is replaced with the
///   masked reference value `reference_value & test_mask`.
/// - [StencilOperation::Increment]: the stencil value in the stencil buffer is incremented. If the
///   current stencil value is the maximum value, don't do anything.
/// - [StencilOperation::WrappingIncrement]: the stencil value in the stencil buffer is incremented.
///   If the current stencil value is the maximum value (as determined by the stencil buffer's
///   bit-depth, e.g. `255` for an 8-bit stencil buffer), set the value to `0`.
/// - [StencilOperation::Decrement]: the stencil value in the stencil buffer is decremented. If the
///   current stencil value is `0`, don't do anything.
/// - [StencilOperation::WrappingDecrement]: the stencil value in the stencil buffer is decremented.
///   If the current stencil value is `0`, set the value to the maximum stencil value (as determined
///   by the stencil buffer's bit-depth, e.g. `255` for an 8-bit stencil buffer).
/// - [StencilOperation::Invert]: inverts the bits of the stencil value in the stencil buffer (for
///   example: `10110011` becomes `01001100`).
///
/// Finally, the bitmask specified by [write_mask_front] (when testing a front-facing fragment) or
/// [write_mask_back] (when testing a back-facing fragment) controls which individual bits can be
/// written too. Suppose an 8-bit stencil buffer and a write mask of `0000011`, then only the final
/// two bits will be updated; where `0` appears, the bits are write-protected.
///
/// Note that reference values, test masks and write masks are declared here as `u32` values,
/// but when testing only the bottom `X` values are used, where `X` is the bit-depth of the stencil
/// buffer.
///
/// A [StencilTest] may be instantiated with the default configuration via [Default]:
///
/// ```
/// use web_glitz::pipeline::graphics::{StencilTest, TestFunction, StencilOperation};
///
/// assert_eq!(StencilTest::default(), StencilTest {
///     test_function_front: TestFunction::AlwaysPass,
///     fail_operation_front: StencilOperation::Keep,
///     pass_depth_fail_operation_front: StencilOperation::Keep,
///     pass_operation_front: StencilOperation::Keep,
///     test_function_back: TestFunction::AlwaysPass,
///     fail_operation_back: StencilOperation::Keep,
///     pass_depth_fail_operation_back: StencilOperation::Keep,
///     pass_operation_back: StencilOperation::Keep,
///     reference_value_front: 0,
///     test_mask_front: 0xffffffff,
///     reference_value_back: 0,
///     test_mask_back: 0xffffffff,
///     write_mask_front: 0xffffffff,
///     write_mask_back: 0xffffffff,
/// });
/// ```
#[derive(PartialEq, Clone, Debug)]
pub struct StencilTest {
    /// The [TestFunction] used by this [StencilTest] for front-facing fragments.
    ///
    /// Only applies to fragments that originate from front-facing triangles, lines or points. For
    /// fragments originating from back-facing triangles [test_function_back] is used instead.
    ///
    /// See the type documentation for [StencilTest] for details on how the available functions are
    /// evaluated.
    pub test_function_front: TestFunction,

    /// The [StencilOperation] that will be used to update the stencil buffer when a front-facing
    /// fragment passes neither the [DepthTest], nor [test_function_front].
    ///
    /// Only applies to fragments that originate from front-facing triangles, lines or points. For
    /// fragments originating from back-facing triangles [fail_operation_back] is used instead.
    pub fail_operation_front: StencilOperation,

    /// The [StencilOperation] that will be used to update the stencil buffer when a front-facing
    /// fragment passes the [DepthTest], but fails [test_function_front].
    ///
    /// Only applies to fragments that originate from front-facing triangles, lines or points. For
    /// fragments originating from back-facing triangles [fail_operation_back] is used instead.
    pub pass_depth_fail_operation_front: StencilOperation,

    /// The [StencilOperation] that will be used to update the stencil buffer when a front-facing
    /// fragment passes both the [DepthTest] and [test_function_front].
    ///
    /// Only applies to fragments that originate from front-facing triangles, lines or points. For
    /// fragments originating from back-facing triangles [fail_operation_back] is used instead.
    pub pass_operation_front: StencilOperation,

    /// The [TestFunction] used by this [StencilTest] for back-facing fragments.
    ///
    /// Only applies to fragments that originate from back-facing triangles. For fragments
    /// originating from back-facing triangles, lines or points, [test_function_front] is used
    /// instead.
    ///
    /// See the type documentation for [StencilTest] for details on how the available functions are
    /// evaluated.
    pub test_function_back: TestFunction,

    /// The [StencilOperation] that will be used to update the stencil buffer when a back-facing
    /// fragment passes neither the [DepthTest], nor [test_function_back].
    ///
    /// Only applies to fragments that originate from back-facing triangles. For fragments
    /// originating from front-facing triangles, lines or points, [fail_operation_front] is used
    /// instead.
    pub fail_operation_back: StencilOperation,

    /// The [StencilOperation] that will be used to update the stencil buffer when a back-facing
    /// fragment passes the [DepthTest], but fails [test_function_back].
    ///
    /// Only applies to fragments that originate from back-facing triangles. For fragments
    /// originating from front-facing triangles, lines or points, [fail_operation_front] is used
    /// instead.
    pub pass_depth_fail_operation_back: StencilOperation,

    /// The [StencilOperation] that will be used to update the stencil buffer when a back-facing
    /// fragment passes both the [DepthTest] and [test_function_back].
    ///
    /// Only applies to fragments that originate from back-facing triangles. For fragments
    /// originating from front-facing triangles, lines or points, [fail_operation_front] is used
    /// instead.
    pub pass_operation_back: StencilOperation,

    /// The value that is compared to a front-facing fragment's stencil value in the stencil buffer,
    /// in order to decide if the fragment passes the stencil test.
    ///
    /// See the type documentation for [StencilTest] for details on how this value is used when
    /// stencil testing.
    pub reference_value_front: u32,

    /// The bitmask that is applied to both the [reference_value_front] and the stencil value in the
    /// stencil buffer to which it is compared, before the stencil test is evaluated on a
    /// front-facing fragment.
    ///
    /// Can be used to mask off certain bits, reserving them for different conditional tests.
    ///
    /// See the type documentation for [StencilTest] for details on how the [test_mask] is applied.
    pub test_mask_front: u32,

    /// The value that is compared to a back-facing fragment's stencil value in the stencil buffer,
    /// in order to decide if the fragment passes the stencil test.
    ///
    /// See the type documentation for [StencilTest] for details on how this value is used when
    /// stencil testing.
    pub reference_value_back: u32,

    /// The bitmask that is applied to both the [reference_value_back] and the stencil value in the
    /// stencil buffer to which it is compared, before the stencil test is evaluated on a
    /// back-facing fragment.
    ///
    /// Can be used to mask off certain bits, reserving them for different conditional tests.
    ///
    /// See the type documentation for [StencilTest] for details on how the [test_mask_back] is
    /// applied.
    pub test_mask_back: u32,

    /// The bitmask that controls which of a stencil value's individual bits may be updated in the
    /// stencil buffer when testing a front-facing fragment.
    ///
    /// Suppose an 8-bit stencil buffer and a [write_mask] of `0000011`, then only the final two
    /// bits will be updated; where `0` appears, the bits are write-protected.
    pub write_mask_front: u32,

    /// The bitmask that controls which of a stencil value's individual bits may be updated in the
    /// stencil buffer when testing a back-facing fragment.
    ///
    /// Suppose an 8-bit stencil buffer and a [write_mask] of `0000011`, then only the final two
    /// bits will be updated; where `0` appears, the bits are write-protected.
    pub write_mask_back: u32,
}

impl StencilTest {
    pub(crate) fn apply(option: &Option<Self>, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        match option {
            Some(stencil_test) => {
                state.set_stencil_test_enabled(true).apply(gl).unwrap();

                state
                    .set_stencil_func_front(
                        stencil_test.test_function_front,
                        stencil_test.reference_value_front as i32,
                        stencil_test.test_mask_front,
                    )
                    .apply(gl)
                    .unwrap();

                state
                    .set_stencil_op_front(
                        stencil_test.fail_operation_front,
                        stencil_test.pass_depth_fail_operation_front,
                        stencil_test.pass_operation_front,
                    )
                    .apply(gl)
                    .unwrap();

                state
                    .set_stencil_write_mask_front(stencil_test.write_mask_front)
                    .apply(gl)
                    .unwrap();

                state
                    .set_stencil_func_back(
                        stencil_test.test_function_back,
                        stencil_test.reference_value_back as i32,
                        stencil_test.test_mask_back,
                    )
                    .apply(gl)
                    .unwrap();

                state
                    .set_stencil_op_back(
                        stencil_test.fail_operation_back,
                        stencil_test.pass_depth_fail_operation_back,
                        stencil_test.pass_operation_back,
                    )
                    .apply(gl)
                    .unwrap();

                state
                    .set_stencil_write_mask_back(stencil_test.write_mask_back)
                    .apply(gl)
                    .unwrap();
            }
            _ => state.set_stencil_test_enabled(false).apply(gl).unwrap(),
        }
    }
}

impl Default for StencilTest {
    fn default() -> Self {
        StencilTest {
            test_function_front: TestFunction::AlwaysPass,
            fail_operation_front: StencilOperation::Keep,
            pass_depth_fail_operation_front: StencilOperation::Keep,
            pass_operation_front: StencilOperation::Keep,
            test_function_back: TestFunction::AlwaysPass,
            fail_operation_back: StencilOperation::Keep,
            pass_depth_fail_operation_back: StencilOperation::Keep,
            pass_operation_back: StencilOperation::Keep,
            reference_value_front: 0,
            test_mask_front: 0xffffffff,
            reference_value_back: 0,
            test_mask_back: 0xffffffff,
            write_mask_front: 0xffffffff,
            write_mask_back: 0xffffffff,
        }
    }
}
