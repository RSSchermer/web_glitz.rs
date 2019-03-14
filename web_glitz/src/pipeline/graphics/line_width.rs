use crate::runtime::Connection;
use std::convert::TryFrom;
use std::ops::Deref;

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
    type Error = NegativeWidth;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value == f32::NAN {
            Err(InvalidWidth::NaN)
        } else if value < 0 {
            Err(InvalidWidth::Negative)
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
