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
