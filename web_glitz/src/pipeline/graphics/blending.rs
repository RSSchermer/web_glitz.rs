use web_sys::WebGl2RenderingContext as Gl;

use crate::runtime::Connection;
use crate::runtime::state::ContextUpdate;

/// Enumerates the possible blending factors that can be applied to color values during [Blending].
///
/// See the documentation for [Blending] for details on how these blending factors are used and what
/// their effects are.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlendFactor {
    Zero,
    One,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SourceColor,
    OneMinusSourceColor,
    DestinationColor,
    OneMinusDestinationColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    SourceAlphaSaturate,
}

impl BlendFactor {
    pub(crate) fn id(&self) -> u32 {
        match self {
            BlendFactor::Zero => Gl::ZERO,
            BlendFactor::One => Gl::ONE,
            BlendFactor::ConstantColor => Gl::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => Gl::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha => Gl::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => Gl::ONE_MINUS_CONSTANT_ALPHA,
            BlendFactor::SourceColor => Gl::SRC_COLOR,
            BlendFactor::OneMinusSourceColor => Gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::DestinationColor => Gl::DST_COLOR,
            BlendFactor::OneMinusDestinationColor => Gl::ONE_MINUS_DST_COLOR,
            BlendFactor::SourceAlpha => Gl::SRC_ALPHA,
            BlendFactor::OneMinusSourceAlpha => Gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DestinationAlpha => Gl::DST_ALPHA,
            BlendFactor::OneMinusDestinationAlpha => Gl::ONE_MINUS_DST_ALPHA,
            BlendFactor::SourceAlphaSaturate => Gl::SRC_ALPHA_SATURATE,
        }
    }
}

/// Enumerates the available blend equations that can be employed to perform [Blending].
///
/// See the documentation for [Blending] for details on how these equations act.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlendEquation {
    Addition,
    Subtraction,
    ReverseSubtraction,
    Min,
    Max,
}

impl BlendEquation {
    pub(crate) fn id(&self) -> u32 {
        match self {
            BlendEquation::Addition => Gl::FUNC_ADD,
            BlendEquation::Subtraction => Gl::FUNC_SUBTRACT,
            BlendEquation::ReverseSubtraction => Gl::FUNC_REVERSE_SUBTRACT,
            BlendEquation::Min => Gl::MIN,
            BlendEquation::Max => Gl::MAX,
        }
    }
}

/// Provides instructions on how blending should be performed.
///
/// When [Blending] is enabled, a fragment's color output does not merely overwrite the color
/// buffer's current color value for this fragment, but instead these two values are combined using
/// a [BlendEquation]. The new color output for the fragment is referred to as the source color,
/// the value already in the color buffer is referred to as the destination color. Separate blending
/// equations may be used for the RGB portion and for the alpha portion of the color value; the
/// respective blending equations are specified by [color_equation] and [alpha_equation]. The
/// following blending equations are available:
///
/// - [BlendEquation::Addition]: the output of blending `O` is calculated as
///   `O = F_s * S + F_d * D`.
/// - [BlendEquation::Subtraction]: the output of blending `O` is calculated as
///   `O = F_s * S - F_d * D`.
/// - [BlendEquation::ReverseSubtraction]: the output of blending `O` is calculated as
///   `O = F_d * D - F_s * S`.
/// - [BlendEquation::Min]: the output of blending `O` is calculated as `O = min(F_d * D, F_s * S)`.
/// - [BlendEquation::Max]: the output of blending `O` is calculated as `O = max(F_d * D, F_s * S)`.
///
/// Here `S` is the relevant portion of the source value: the [color_equation] will use the red,
/// green and blue components of the source color as `S`, the [alpha_equation] will use the alpha
/// components of the source color as `S`. `D` is the relevant portion of the destination value: the
/// [color_function] will use the red, green and blue components of the destination color as `D`
/// and the [alpha_function] will use the alpha component of the destination color as `D`. `F_s` and
/// `F_d` are [BlendFactor]s for `S` and `D` respectively. The following blend factors are
/// available:
///
/// - [BlendFactor::Zero]: all color components are multiplied by `0`.
/// - [BlendFactor::One]: all color components are multiplied by `1`.
/// - [BlendFactor::ConstantColor]: the value of each color component is multiplied by the value
///   or the corresponding component of the color specified by [constant_color].
/// - [BlendFactor::OneMinusConstantColor]: the value of each color component is multiplied by
///   the value of the corresponding component of the color specified by [constant_color],
///   subtracted from `1`. For example, the red component is multiplied by `1 - R_c`, where `R_c` is
///   the value of the red component of the [constant_color].
/// - [BlendFactor::OneMinusConstantAlpha]: all color components are multiplied by `1 - A_c`,
///   where `A_c` is the value of the alpha component of the color specified by [constant_color].
/// - [BlendFactor::SourceColor]: the value of each color component is multiplied by the value of
///   the corresponding component of the source color.
/// - [BlendFactor::OneMinusSourceColor]: the value of each color component is multiplied by the
///   value of the corresponding component of the source color subtracted from `1`. For example, the
///   red component is multiplied by `1 - R_s`, where `R_s` is the value of the red component of the
///   source color.
/// - [BlendFactor::DestinationColor]: the value of each color component is multiplied by the
///   value of the corresponding component of the destination color.
/// - [BlendFactor::OneMinusDestinationColor]: the value of each color component is multiplied by
///   the value of the corresponding component of the destination color subtracted from `1`. For
///   example, the red component is multiplied by `1 - R_d`, where `R_d` is the value of the red
///   component of the destination color.
/// - [BlendFactor::SourceAlpha]: all color components are multiplied by the value of the alpha
///   component of the source color.
/// - [BlendFactor::OneMinusSourceAlpha]: all color components are multiplied by `1 - A_s`, where
///   `A_s` is the value of the alpha component of the source color.
/// - [BlendFactor::DestinationAlpha]: all color components are multiplied by the value of the
///   alpha component of the destination color.
/// - [BlendFactor::OneMinusDestinationAlpha]: all color components are multiplied by `1 - A_s`,
///   where `A_s` is the value of the alpha component of the destination color.
/// - [BlendFactor::SourceAlphaSaturate]: all color components are multiplied by the smaller of
///   either `A_s` or `1 - A_d`, where `A_s` is the value of the alpha component of the source color
///   and `A_d` is the value of the alpha component of the destination color.
///
/// [Blending] may be instantiated with default values through [Default]:
///
/// ```
/// use web_glitz::pipeline::graphics::{Blending, BlendEquation, BlendFactor};
///
/// assert_eq!(Blending::default(), Blending {
///     constant_color: [0.0, 0.0, 0.0, 0.0],
///     source_color_factor: BlendFactor::One,
///     source_alpha_factor: BlendFactor::One,
///     destination_color_factor: BlendFactor::Zero,
///     destination_alpha_factor: BlendFactor::Zero,
///     color_equation: BlendEquation::Addition,
///     alpha_equation: BlendEquation::Addition
/// });
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct Blending {
    /// The color used as the constant color when [BlendFactor::ConstantColor],
    /// [BlendFactor::OneMinusConstantColor], [BlendFactor::ConstantAlpha] or
    /// [BlendFactor::OneMinusConstantAlpha] is used as a blending factor.
    ///
    /// Component values are clamped to `[0, 1]`.
    ///
    /// This value is ignored when other [BlendFactor]s are used.
    pub constant_color: [f32; 4],

    /// The [BlendFactor] that the [color_equation] applies to the source value.
    pub source_color_factor: BlendFactor,

    /// The [BlendFactor] that the [alpha_equation] applies to the source value.
    pub source_alpha_factor: BlendFactor,

    /// The [BlendFactor] that the [color_equation] applies to the destination value.
    pub destination_color_factor: BlendFactor,

    /// The [BlendFactor] that the [alpha_equation] applies to the destination value.
    pub destination_alpha_factor: BlendFactor,

    /// The [BlendFactor] used to combine the red, green and blue components of the source and
    /// destination colors.
    pub color_equation: BlendEquation,

    /// The [BlendEquation] used to combine the alpha components of the source and destination
    /// colors.
    pub alpha_equation: BlendEquation,
}

impl Blending {
    pub(crate) fn apply(option: &Option<Self>, connection: &mut Connection) {
        let (gl, state) = unsafe { connection.unpack_mut() };

        match option {
            Some(blend) => {
                state.set_blend_enabled(true).apply(gl).unwrap();
                state
                    .set_blend_color(blend.constant_color)
                    .apply(gl)
                    .unwrap();
                state
                    .set_blend_equations(blend.color_equation, blend.alpha_equation)
                    .apply(gl)
                    .unwrap();
                state
                    .set_blend_func(
                        blend.source_color_factor,
                        blend.destination_color_factor,
                        blend.source_alpha_factor,
                        blend.destination_alpha_factor,
                    )
                    .apply(gl)
                    .unwrap();
            }
            _ => state.set_blend_enabled(false).apply(gl).unwrap(),
        }
    }
}

impl Default for Blending {
    fn default() -> Self {
        Blending {
            constant_color: [0.0; 4],
            source_color_factor: BlendFactor::One,
            source_alpha_factor: BlendFactor::One,
            destination_color_factor: BlendFactor::Zero,
            destination_alpha_factor: BlendFactor::Zero,
            color_equation: BlendEquation::Addition,
            alpha_equation: BlendEquation::Addition,
        }
    }
}
