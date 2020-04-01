use std::borrow::Borrow;
use std::marker;
use std::mem;

use wasm_bindgen::JsCast;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

/// Encapsulates data that may be uploaded to a 2D texture (sub-)image.
///
/// # Example
///
/// ```rust
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
/// use web_glitz::image::{Image2DSource, MipmapLevels};
/// use web_glitz::image::format::RGB8;
/// use web_glitz::image::texture_2d::Texture2DDescriptor;
///
/// let texture = context.try_create_texture_2d(&Texture2DDescriptor {
///     format: RGB8,
///     width: 256,
///     height: 256,
///     levels: MipmapLevels::Complete
/// }).unwrap();
///
/// let data: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
/// let image_source = Image2DSource::from_pixels(data, 256, 256).unwrap();
///
/// context.submit(texture.base_level().upload_command(image_source));
/// # }
/// ```
///
/// Note that the pixel data type (`[u8; 3]` in the example) must implement [ClientFormat] for the
/// texture's [InternalFormat] (in this case that means `ClientFormat<RGB8>` must be implemented
/// for `[u8; 3]`, which it is).
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
    /// Creates a new [Image2DSource] from the `pixels` for an image with the given `width` and the
    /// given `height`.
    ///
    /// Returns [FromPixelsError::NotEnoughPixels] if the `pixels` does not contain enough data for
    /// at least `width * height` pixels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use web_glitz::image::Image2DSource;
    ///
    /// let data: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256];
    /// let image_source = Image2DSource::from_pixels(data, 256, 256).unwrap();
    /// ```
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

impl Image2DSource<Vec<[u8; 4]>, [u8; 4]> {
    /// Creates a new [Image2DSource] for the `image_element`.
    ///
    /// The width will be equal to the [HtmlImageElement::natural_width] of the image element and
    /// the height will be equal the [HtmlImageElement::natural_height] of the image element.
    ///
    /// # Panics
    ///
    /// Panics if the image element is not yet "complete" (see [HtmlImageElement::complete]).
    pub fn from_image_element(image_element: &HtmlImageElement) -> Self {
        // Current implementation is very conservative and wasteful, copying the image data into a
        // new vector. WebGL support initializing textures from image elements directly which would
        // avoid the copy and may even avoid an upload as the browser may have already uploaded the
        // pixel data previously. However, it is currently unclear to me how sending
        // HtmlImageElements to secondary workers/threads would work.

        if !image_element.complete() {
            panic!("Incomplete image.");
        }

        let document = window().unwrap().document().unwrap();

        let width = image_element.natural_width();
        let height = image_element.natural_height();

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_width(width);
        canvas.set_height(height);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        context
            .draw_image_with_html_image_element(&image_element, 0.0, 0.0)
            .unwrap();

        let mut image_data = context
            .get_image_data(0.0, 0.0, width as f64, height as f64)
            .unwrap()
            .data();

        let len = image_data.len();
        let capacity = image_data.capacity();
        let ptr = image_data.as_mut_ptr();

        mem::forget(image_data);

        let pixels = unsafe { Vec::from_raw_parts(mem::transmute(ptr), len / 4, capacity / 4) };

        Image2DSource {
            internal: Image2DSourceInternal::PixelData {
                data: pixels,
                row_length: width,
                image_height: height,
                alignment: Alignment::Byte4,
            },
            _marker: marker::PhantomData,
        }
    }
}

/// Encapsulates data that may be uploaded to a layered texture (sub-)image.
///
/// # Example
///
/// ```rust
/// # use web_glitz::runtime::RenderingContext;
/// # fn wrapper<Rc>(context: &Rc) where Rc: RenderingContext + Clone + 'static {
/// use web_glitz::image::{LayeredImageSource, MipmapLevels};
/// use web_glitz::image::format::RGB8;
/// use web_glitz::image::texture_3d::Texture3DDescriptor;
///
/// let texture = context.try_create_texture_3d(&Texture3DDescriptor {
///     format: RGB8,
///     width: 256,
///     height: 256,
///     depth: 256,
///     levels: MipmapLevels::Complete
/// }).unwrap();
///
/// let data: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256 * 256];
/// let image_source = LayeredImageSource::from_pixels(data, 256, 256, 256).unwrap();
///
/// context.submit(texture.base_level().upload_command(image_source));
/// # }
/// ```
///
/// Note that the pixel data type (`[u8; 3]` in the example) must implement [ClientFormat] for the
/// texture's [InternalFormat] (in this case that means `ClientFormat<RGB8>` must be implemented
/// for `[u8; 3]`, which it is).
pub struct LayeredImageSource<D, T> {
    pub(crate) internal: LayeredImageSourceInternal<D>,
    _marker: marker::PhantomData<[T]>,
}

pub(crate) enum LayeredImageSourceInternal<D> {
    PixelData {
        data: D,
        row_length: u32,
        image_height: u32,
        image_count: u32,
        alignment: Alignment,
    },
}

impl<D, T> LayeredImageSource<D, T>
where
    D: Borrow<[T]>,
{
    /// Creates a new [LayeredImageSource] from the `pixels` for an image with the given `width`,
    /// the given `height` and the given `depth`.
    ///
    /// In this context the `depth` of the image corresponds to its number of layers.
    ///
    /// Returns [FromPixelsError::NotEnoughPixels] if the `pixels` does not contain enough data for
    /// at least `width * height * depth` pixels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use web_glitz::image::LayeredImageSource;
    ///
    /// let data: Vec<[u8; 3]> = vec![[255, 0, 0]; 256 * 256 * 256];
    /// let image_source = LayeredImageSource::from_pixels(data, 256, 256, 256).unwrap();
    /// ```
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

        Ok(LayeredImageSource {
            internal: LayeredImageSourceInternal::PixelData {
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

/// Error returned by [Image2DSource::from_pixels] or [Image3DSource::from_pixels].
///
/// See [Image2DSource::from_pixels] and [Image3DSource::from_pixels] for details.
#[derive(Debug)]
pub enum FromPixelsError {
    /// Variant returned when the data does not contain enough pixels to describe an image of the
    /// required dimensions.
    NotEnoughPixels(usize, u32),

    /// Variant returned when the pixel data type has an unsupported alignment.
    UnsupportedAlignment(usize),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Alignment {
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
