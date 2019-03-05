/// Enumerates the possible blending factors that can be applied to color
/// values during [Blending].
///
/// See the documentation for [Blending] for details on how these blending
/// factors are used and what their effects are.
pub enum BlendingFactor {
    Zero,
    One,
    SourceColor,
    OneMinusSourceColor,
    DestinationColor,
    OneMinusDestinationColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    SourceAlphaSaturate
}

/// Enumerates the available functions that can be employed to perform
/// [Blending].
///
/// See the documentation for [Blending] for details on how these functions act.
pub enum BlendingFunction {
    Addition,
    Subtraction,
    ReverseSubtraction
}