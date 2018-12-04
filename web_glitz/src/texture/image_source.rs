use std::borrow::Borrow;
use std::marker;
use std::mem;

pub struct Image2DSource<D, T> {
    pub(crate) internal: Image2DSourceInternal<D>,
    _marker: marker::PhantomData<[T]>,
}

pub(crate) enum Image2DSourceInternal<D> {
    PixelData {
        data: D,
        row_length: u32,
        image_height: u32,
        alignment: Alignment,
    },
}

impl<D, T> Image2DSource<D, T>
where
    D: Borrow<[T]>,
{
    pub fn from_pixels(pixels: D, width: u32, height: u32) -> Result<Self, FromPixelsError> {
        let len = pixels.borrow().len();
        let expected_len = width * height;

        if len < expected_len as usize {
            return Err(FromPixelsError::NotEnoughPixels(len, expected_len));
        }

        let alignment = match mem::align_of::<T>() {
            1 => Alignment::Byte,
            2 => Alignment::Byte2,
            4 => Alignment::Byte4,
            8 => Alignment::Byte8,
            a => return Err(FromPixelsError::UnsupportedAlignment(a)),
        };

        Ok(Image2DSource {
            internal: Image2DSourceInternal::PixelData {
                data: pixels,
                row_length: width,
                image_height: height,
                alignment,
            },
            _marker: marker::PhantomData,
        })
    }
}

pub struct Image3DSource<D, T> {
    pub(crate) internal: Image3DSourceInternal<D>,
    _marker: marker::PhantomData<[T]>,
}

pub(crate) enum Image3DSourceInternal<D> {
    PixelData {
        data: D,
        row_length: u32,
        image_height: u32,
        image_count: u32,
        alignment: Alignment,
    },
}

impl<D, T> Image3DSource<D, T>
where
    D: Borrow<[T]>,
{
    pub fn from_pixels(
        pixels: D,
        width: u32,
        height: u32,
        depth: u32,
    ) -> Result<Self, FromPixelsError> {
        let len = pixels.borrow().len();
        let expected_len = width * height * depth;

        if len < expected_len as usize {
            return Err(FromPixelsError::NotEnoughPixels(len, expected_len));
        }

        let alignment = match mem::align_of::<T>() {
            1 => Alignment::Byte,
            2 => Alignment::Byte2,
            4 => Alignment::Byte4,
            8 => Alignment::Byte8,
            a => return Err(FromPixelsError::UnsupportedAlignment(a)),
        };

        Ok(Image3DSource {
            internal: Image3DSourceInternal::PixelData {
                data: pixels,
                row_length: width,
                image_height: height,
                image_count: depth,
                alignment,
            },
            _marker: marker::PhantomData,
        })
    }
}

pub enum FromPixelsError {
    NotEnoughPixels(usize, u32),
    UnsupportedAlignment(usize),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Alignment {
    Byte,
    Byte2,
    Byte4,
    Byte8,
}

impl Into<i32> for Alignment {
    fn into(self) -> i32 {
        match self {
            Alignment::Byte => 1,
            Alignment::Byte2 => 2,
            Alignment::Byte4 => 4,
            Alignment::Byte8 => 8,
        }
    }
}
