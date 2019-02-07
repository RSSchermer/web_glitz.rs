mod image_source;
pub use self::image_source::{Alignment, FromPixelsError, Image2DSource, Image3DSource};

pub mod format;
pub mod renderbuffer;
pub mod texture_2d;
pub mod texture_2d_array;
pub mod texture_3d;
pub mod texture_cube;

mod texture_object_dropper;
mod util;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Region2D {
    Fill,
    Area((u32, u32), u32, u32),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Region3D {
    Fill,
    Area((u32, u32, u32), u32, u32, u32),
}

impl Into<Region2D> for Region3D {
    fn into(self) -> Region2D {
        match self {
            Region3D::Fill => Region2D::Fill,
            Region3D::Area((offset_x, offset_y, _), width, height, _) => {
                Region2D::Area((offset_x, offset_y), width, height)
            }
        }
    }
}

pub enum MipmapLevels {
    Auto,
    Manual(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaxMipmapLevelsExceeded {
    pub given: usize,
    pub max: usize,
}
