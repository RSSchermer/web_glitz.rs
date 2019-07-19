use std::cmp;

use crate::image::{Region2D, Region3D};

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
