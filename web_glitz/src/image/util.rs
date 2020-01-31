use std::{cmp, mem, slice};

use js_sys::{Object, Uint8Array, Uint16Array, Uint32Array, Int8Array, Int16Array, Int32Array, Float32Array};
use web_sys::WebGl2RenderingContext as Gl;

use crate::image::{Region2D, Region3D};
use crate::image::format::{PixelUnpack, InternalFormat};

pub(crate) fn max_mipmap_levels(width: u32, height: u32) -> usize {
    (cmp::max(width, height) as f64).log2() as usize + 1
}

pub(crate) fn mipmap_size(base_size: u32, level: usize) -> u32 {
    let level_size = base_size / 2u32.pow(level as u32);

    if level_size < 1 {
        1
    } else {
        level_size
    }
}

pub(crate) fn region_2d_overlap_width(base_width: u32, level: usize, region: &Region2D) -> u32 {
    let level_width = mipmap_size(base_width, level);

    match *region {
        Region2D::Area((offset_x, _), width, _) => {
            if offset_x >= level_width {
                0
            } else {
                let max_width = level_width - offset_x;

                cmp::min(max_width, width)
            }
        }
        Region2D::Fill => level_width,
    }
}

pub(crate) fn region_2d_overlap_height(base_height: u32, level: usize, region: &Region2D) -> u32 {
    let level_height = mipmap_size(base_height, level);

    match *region {
        Region2D::Area((_, offset_y), _, height) => {
            if offset_y >= level_height {
                0
            } else {
                let max_height = level_height - offset_y;

                cmp::min(max_height, height)
            }
        }
        Region2D::Fill => level_height,
    }
}

pub(crate) fn region_3d_overlap_width(base_width: u32, level: usize, region: &Region3D) -> u32 {
    let level_width = mipmap_size(base_width, level);

    match *region {
        Region3D::Area((offset_x, _, _), width, ..) => {
            if offset_x >= level_width {
                0
            } else {
                let max_width = level_width - offset_x;

                cmp::min(max_width, width)
            }
        }
        Region3D::Fill => level_width,
    }
}

pub(crate) fn region_3d_overlap_height(base_height: u32, level: usize, region: &Region3D) -> u32 {
    let level_height = mipmap_size(base_height, level);

    match *region {
        Region3D::Area((_, offset_y, _), _, height, _) => {
            if offset_y >= level_height {
                0
            } else {
                let max_height = level_height - offset_y;

                cmp::min(max_height, height)
            }
        }
        Region3D::Fill => level_height,
    }
}

pub(crate) fn region_3d_overlap_depth(base_depth: u32, region: &Region3D) -> u32 {
    match *region {
        Region3D::Area((_, _, offset_z), _, _, depth) => {
            if offset_z >= base_depth {
                0
            } else {
                let max_depth = base_depth - offset_z;

                cmp::min(max_depth, depth)
            }
        }
        Region3D::Fill => base_depth,
    }
}

pub(crate) fn region_2d_sub_image(region_a: Region2D, region_b: Region2D) -> Region2D {
    match region_b {
        Region2D::Fill => region_a,
        Region2D::Area((b_offset_x, b_offset_y), width, height) => match region_a {
            Region2D::Fill => region_b,
            Region2D::Area((a_offset_x, a_offset_y), ..) => Region2D::Area(
                (a_offset_x + b_offset_x, a_offset_y + b_offset_y),
                width,
                height,
            ),
        },
    }
}

pub(crate) fn region_3d_sub_image(region_a: Region3D, region_b: Region3D) -> Region3D {
    match region_b {
        Region3D::Fill => region_a,
        Region3D::Area((b_offset_x, b_offset_y, b_offset_z), width, height, depth) => {
            match region_a {
                Region3D::Fill => region_b,
                Region3D::Area((a_offset_x, a_offset_y, a_offset_z), ..) => {
                    let offset_x = a_offset_x + b_offset_x;
                    let offset_y = a_offset_y + b_offset_y;
                    let offset_z = a_offset_z + b_offset_z;

                    Region3D::Area((offset_x, offset_y, offset_z), width, height, depth)
                }
            }
        }
    }
}

enum TextureBufferType {
    Float32,
    Uint8,
    Uint16,
    Uint32,
    Int8,
    Int16,
    Int32,
}

impl TextureBufferType {
    fn from_type_id(id: u32) -> Self {
        match id {
            Gl::BYTE => TextureBufferType::Int8,
            Gl::SHORT => TextureBufferType::Int16,
            Gl::INT => TextureBufferType::Int32,
            Gl::UNSIGNED_BYTE => TextureBufferType::Uint8,
            Gl::UNSIGNED_SHORT => TextureBufferType::Uint16,
            Gl::UNSIGNED_INT => TextureBufferType::Uint32,
            Gl::FLOAT => TextureBufferType::Float32,
            Gl::UNSIGNED_SHORT_5_6_5 => TextureBufferType::Uint16,
            Gl::UNSIGNED_INT_10F_11F_11F_REV => TextureBufferType::Uint32,
            Gl::UNSIGNED_INT_5_9_9_9_REV => TextureBufferType::Uint32,
            Gl::UNSIGNED_SHORT_5_5_5_1 => TextureBufferType::Uint16,
            Gl::UNSIGNED_INT_2_10_10_10_REV => TextureBufferType::Uint32,
            Gl::UNSIGNED_SHORT_4_4_4_4 => TextureBufferType::Uint16,
            Gl::UNSIGNED_INT_24_8 => TextureBufferType::Uint32,
            _ => panic!("Unsupported texture data type.")
        }
    }
}

pub(crate) fn texture_data_as_js_buffer<F, T>(data: &[F], elements: usize) -> Object where F: PixelUnpack<T>, T: InternalFormat {
    let element_size = mem::size_of::<F>();
    let len_in_bytes = data.len() * element_size;
    let max_len_in_bytes = element_size * elements;

    let buffer_type = TextureBufferType::from_type_id(F::TYPE_ID);

    match buffer_type {
        TextureBufferType::Float32 => {
            let len = len_in_bytes / 4;
            let max_len = max_len_in_bytes / 4;

            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const f32, len as usize);

                if max_len < len {
                    data = &data[0..max_len];
                }

                let js_buffer = Float32Array::from(data);

                js_buffer.into()
            }
        }
        TextureBufferType::Uint8 => {
            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const u8, len_in_bytes as usize);

                if max_len_in_bytes < len_in_bytes {
                    data = &data[0..max_len_in_bytes];
                }

                let js_buffer = Uint8Array::from(data);

                js_buffer.into()
            }
        }
        TextureBufferType::Uint16 => {
            let len = len_in_bytes / 2;
            let max_len = max_len_in_bytes / 2;

            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const u16, len as usize);

                if max_len < len {
                    data = &data[0..max_len];
                }

                let js_buffer = Uint16Array::from(data);

                js_buffer.into()
            }
        }
        TextureBufferType::Uint32 => {
            let len = len_in_bytes / 4;
            let max_len = max_len_in_bytes / 4;

            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const u32, len as usize);

                if max_len < len {
                    data = &data[0..max_len];
                }

                let js_buffer = Uint32Array::from(data);

                js_buffer.into()
            }
        }
        TextureBufferType::Int8 => {
            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const i8, len_in_bytes as usize);

                if max_len_in_bytes < len_in_bytes {
                    data = &data[0..max_len_in_bytes];
                }

                let js_buffer = Int8Array::from(data);

                js_buffer.into()
            }
        }
        TextureBufferType::Int16 => {
            let len = len_in_bytes / 2;
            let max_len = max_len_in_bytes / 2;

            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const i16, len as usize);

                if max_len < len {
                    data = &data[0..max_len];
                }

                let js_buffer = Int16Array::from(data);

                js_buffer.into()
            }
        }
        TextureBufferType::Int32 => {
            let len = len_in_bytes / 4;
            let max_len = max_len_in_bytes / 4;

            unsafe {
                let mut data =
                    slice::from_raw_parts(data as *const _ as *const i32, len as usize);

                if max_len < len {
                    data = &data[0..max_len];
                }

                let js_buffer = Int32Array::from(data);

                js_buffer.into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mipmap_size() {
        assert_eq!(mipmap_size(256, 0), 256);
        assert_eq!(mipmap_size(256, 1), 128);
        assert_eq!(mipmap_size(256, 2), 64);
        assert_eq!(mipmap_size(256, 3), 32);
        assert_eq!(mipmap_size(256, 4), 16);
        assert_eq!(mipmap_size(256, 5), 8);
        assert_eq!(mipmap_size(256, 6), 4);
        assert_eq!(mipmap_size(256, 7), 2);
        assert_eq!(mipmap_size(256, 8), 1);
    }
}
