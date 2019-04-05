use std::sync::Arc;

use wasm_bindgen::JsCast;

use web_sys::WebGl2RenderingContext as Gl;

use crate::runtime::{Connection, Extensions, RenderingContext};
use crate::task::Progress;
use crate::task::{ContextId, GpuTask};
use crate::util::{arc_get_mut_unchecked, JsId};

/// Enumerates the filters available to a [Sampler] for magnification filtering.
///
/// Magnification filtering is used when a sampling a texture value for a fragment that is smaller
/// than the candidate texels.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MagnificationFilter {
    /// The sampled value is chosen to be the value of the texel whose coordinates are closest to
    /// the sampling coordinates.
    Nearest = Gl::NEAREST as isize,

    /// The sampled value is calculated by linearly interpolating between the 4 texels that are
    /// closest to the sampling coordinates.
    Linear = Gl::LINEAR as isize,
}

/// Enumerates the methods available to a [Sampler] for minification filtering.
///
/// Minification filtering is used when a sampling a texture value for a fragment that is larger
/// than the candidate texels.
///
/// # Minification Filtering and Mipmapping
///
/// Some of the filtering methods involve mipmapping. When a fragment is larger than the candidate
/// texels, the fragment surface might span multiple texels. The most appropriate sample value might
/// then be obtained by interpolating between these texels. However, doing this for each sampling
/// operation can be very expensive.
///
/// This is instead solved by using a mipmap, which produces similar results with much better
/// performance. A mipmap is a pre-calculated sequence of images, starting with the original image.
/// Each subsequent image is half the width and half the height of the previous image (rounded
/// down). The sequence ends when the width or height reaches 1. Each image in the mipmap sequence
/// is identified by a mipmap level: the base image has a mipmap level of 0, the subsequent image
/// has a mipmap level of 1, etc. For example, a mipmap of a base image of size 256 by 256 has 9
/// mipmap levels: 256x256 (level 0), 128x128 (level 1), 64x64 (level 2), 32x32 (level 3), 16x16
/// (level 4), 8x8 (level 5), 4x4 (level 6), 2x2 (level 7), 1x1 (level 8).
///
/// See the documentation for [NearestMipmapNearest], [NearestMipmapLinear], [LinearMipmapNearest]
/// and [LinearMipmapLinear] for details on how these filtering operations will make use of a
/// mipmap.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MinificationFilter {
    /// The sampled value is chosen to be the value of the texel whose coordinates are closest to
    /// the sampling coordinates.
    Nearest = Gl::NEAREST as isize,

    /// The sampled value is calculated by linearly interpolating between the 4 texels that are
    /// closest to the sampling coordinates.
    Linear = Gl::LINEAR as isize,

    /// First selects the mipmap level for which the texel size is closest to the fragment size,
    /// then the sampled value is chose to be the value of the texel whose coordinates are closest
    /// to the sampling coordinates.
    NearestMipmapNearest = Gl::NEAREST_MIPMAP_NEAREST as isize,

    /// First selects the mipmap level for which the texel size is closest to the fragment size,
    /// then the sampled value is calculated by linearly interpolating between the 4 texels that are
    /// closest to the sampling coordinates.
    NearestMipmapLinear = Gl::NEAREST_MIPMAP_LINEAR as isize,

    /// First selects both the nearest mipmap level for which the texel size is smaller than the
    /// fragment, as well as the nearest mipmap level for which the texel size is larger than the
    /// fragment; then samples a value from both mipmap levels by choosing the texel whose
    /// coordinates are closest to the sampling coordinates; finally, the sample value is calculated
    /// by linearly interpolating between these two values.
    LinearMipmapNearest = Gl::LINEAR_MIPMAP_NEAREST as isize,

    /// First selects both the nearest mipmap level for which the texel size is smaller than the
    /// fragment, as well as the nearest mipmap level for which the texel size is larger than the
    /// fragment; then samples a value from both mipmap levels by linearly interpolating between the
    /// 4 texels that are closest to the sampling coordinates; finally, the sample value is
    /// calculated by linearly interpolating between these two values.
    LinearMipmapLinear = Gl::LINEAR_MIPMAP_LINEAR as isize,
}

/// Enumerates the methods available to a [Sampler] for texture coordinate wrapping.
///
/// Texture coordinate wrapping concerns texture coordinate values outside of the range `0.0..=1.0`.
/// The extremes of this range correspond to the edges of the texture. A texture coordinate value
/// outside of this range therefore has to be mapped to a coordinate value on this range.
///
/// Separate wrapping methods can be used for each texture space coordinate component (typically
/// referred to as the `S`, `T`, `R` coordinates or "width", "height", "depth" respectively), see
/// [SamplerDescriptor] and [ShadowSamplerDescriptor].
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Wrap {
    /// If the coordinate value is smaller than `0.0`, then `0.0` is used as the coordinate value;
    /// if the coordinate value is greater than `1.0`, then `1.0` is used as the coordinate value.
    ///
    /// For example, `-3.15` maps to `0.0` and `2.85` maps to `1.0`.
    ClampToEdge = Gl::CLAMP_TO_EDGE as isize,

    /// The integer part of the coordinate value is ignored.
    ///
    /// For example, `3.15` maps to `0.15`.
    Repeat = Gl::REPEAT as isize,

    /// Similar to [Repeat], however, if the integer part is odd, then the decimal part is
    /// subtracted from `1`.
    ///
    /// For example, `2.15` maps to `0.15` and `3.15` maps to `0.85`.
    MirroredRepeat = Gl::MIRRORED_REPEAT as isize,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct LODRange {
    min: f32,
    max: f32,
}

impl Default for LODRange {
    fn default() -> Self {
        LODRange {
            min: -1000.0,
            max: 1000.0,
        }
    }
}

/// Provides the information necessary for the creation of a [Sampler].
///
/// See [RenderingContext::create_sampler] for details.
///
/// Can be instantiated with default values through [Default]:
///
/// ```rust
/// use web_glitz::sampler::{
///     SamplerDescriptor, MinificationFilter, MagnificationFilter, LODRange, Wrap
/// };
///
/// assert_eq!(SamplerDescriptor::default(), SamplerDescriptor {
///     minification_filter: MinificationFilter::NearestMipmapLinear,
///     magnification_filter: MagnificationFilter::Linear,
///     lod_range: LODRange::default(),
///     wrap_s: Wrap::Repeat,
///     wrap_t: Wrap::Repeat,
///     wrap_r: Wrap::Repeat,
/// });
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct SamplerDescriptor {
    /// The [MinificationFilter] that a sampler created from this descriptor will use.
    ///
    /// See [MinificationFilter] for details.
    pub minification_filter: MinificationFilter,

    /// The [MagnificationFilter] that a sampler created from this descriptor will use.
    ///
    /// See [MagnificationFilter] for details.
    pub magnification_filter: MagnificationFilter,

    /// The [LODRange] that a sampler created from this descriptor will use.
    ///
    /// See [LODRange] for details.
    pub lod_range: LODRange,

    /// The wrapping method that a sampler created from this descriptor will use when sampling a
    /// value at coordinates outside the range `0.0..=1.0` in the `S` ("width") direction in texture
    /// space.
    ///
    /// See [Wrap] for details.
    pub wrap_s: Wrap,

    /// The wrapping method that a sampler created from this descriptor will use when sampling a
    /// value at coordinates outside the range `0.0..=1.0` in the `T` ("height") direction in
    /// texture space.
    ///
    /// See [Wrap] for details.
    pub wrap_t: Wrap,

    /// The wrapping algorithm that a sampler created from this descriptor will use when sampling
    /// a value at coordinates outside the range `0.0..=1.0` in the `R` ("depth") direction in
    /// texture space.
    ///
    /// See [Wrap] for details.
    pub wrap_r: Wrap,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        SamplerDescriptor {
            minification_filter: MinificationFilter::NearestMipmapLinear,
            magnification_filter: MagnificationFilter::Linear,
            lod_range: LODRange::default(),
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            wrap_r: Wrap::Repeat,
        }
    }
}

/// Samples texture values given texture coordinates texture coordinates.
///
/// A [Sampler] attempts to obtain texture values by mapping texture coordinates onto texels
/// (texture pixels). However, a set of texture coordinates rarely corresponds to exactly 1 texel
/// unambiguously. Instead there are often several candidate texels. The [Sampler] performs texture
/// filtering and texture wrapping in order to obtain the most appropriate texture value.
///
/// See the documentation for [RenderingContext::create_sampler] for details on how to create a
/// [Sampler].
pub struct Sampler {
    data: Arc<SamplerData>,
    descriptor: SamplerDescriptor,
}

impl Sampler {
    pub(crate) fn new<Rc>(context: &Rc, descriptor: &SamplerDescriptor) -> Sampler
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(SamplerData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            extensions: context.extensions().clone(),
        });

        context.submit(SamplerAllocateCommand {
            data: data.clone(),
            descriptor: descriptor.clone(),
        });

        Sampler {
            data,
            descriptor: descriptor.clone(),
        }
    }

    pub(crate) fn data(&self) -> &Arc<SamplerData> {
        &self.data
    }

    /// The [MinificationFilter] used by this [Sampler].
    ///
    /// See [MinificationFilter] for details.
    pub fn minification_filter(&self) -> MinificationFilter {
        self.descriptor.minification_filter
    }

    /// The [MagnificationFilter] used by this [Sampler].
    ///
    /// See [MagnificationFilter] for details.
    pub fn magnification_filter(&self) -> MagnificationFilter {
        self.descriptor.magnification_filter
    }

    /// The [LODRange] used by this [Sampler].
    ///
    /// See [LODRange] for details.
    pub fn lod_range(&self) -> LODRange {
        self.descriptor.lod_range
    }

    /// The wrapping method that this [Sampler] uses when sampling a value at coordinates outside
    /// the range `0.0..=1.0` in the `S` ("width") direction in texture space.
    ///
    /// See [Wrap] for details.
    pub fn wrap_s(&self) -> Wrap {
        self.descriptor.wrap_s
    }

    /// The wrapping method that this [Sampler] uses when sampling a value at coordinates outside
    /// the range `0.0..=1.0` in the `T` ("height") direction in texture space.
    ///
    /// See [Wrap] for details.
    pub fn wrap_t(&self) -> Wrap {
        self.descriptor.wrap_t
    }

    /// The wrapping method that this [Sampler] uses when sampling a value at coordinates outside
    /// the range `0.0..=1.0` in the `R` ("depth") direction in texture space.
    ///
    /// See [Wrap] for details.
    pub fn wrap_r(&self) -> Wrap {
        self.descriptor.wrap_r
    }
}

/// Enumerates the compare functions available for a [ShadowSampler].
///
/// See [ShadowSampler] for details.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CompareFunction {
    /// The function passes if the texel value is equal to the reference value.
    Equal = Gl::EQUAL as isize,

    /// The function passes if the texel value is not equal to the reference value.
    NotEqual = Gl::NOTEQUAL as isize,

    /// The function passes if the texel value is strictly smaller than the reference value.
    Less = Gl::LESS as isize,

    /// The function passes if the texel value is strictly greater than the reference value.
    Greater = Gl::GREATER as isize,

    /// The function passes if the texel value is smaller than or equal to the reference value.
    LessOrEqual = Gl::LEQUAL as isize,

    /// The function passes if the texel value is greater than or equal to the reference value.
    GreaterOrEqual = Gl::GEQUAL as isize,

    /// The function always passes, regardless of how the texel value compares to the reference
    /// value.
    Always = Gl::ALWAYS as isize,

    /// The function never passes, regardless of how the texel value compares to the reference
    /// value.
    Never = Gl::NEVER as isize,
}

/// Provides the information necessary for the creation of a [Sampler].
///
/// See [RenderingContext::create_shadow_sampler] for details.
///
/// Can be instantiated with default values through [Default]:
///
/// ```rust
/// use web_glitz::sampler::{ShadowSamplerDescriptor, CompareFunction, Wrap};
///
/// assert_eq!(ShadowSamplerDescriptor::default(), ShadowSamplerDescriptor {
///     compare: CompareFunction::LessOrEqual,
///     wrap_s: Wrap::Repeat,
///     wrap_t: Wrap::Repeat,
///     wrap_r: Wrap::Repeat,
/// });
/// ```
#[derive(PartialEq, Clone, Debug)]
pub struct ShadowSamplerDescriptor {
    /// The [CompareFunction] that a [ShadowSampler] created from this descriptor will use.
    ///
    /// See [ShadowSampler] and [CompareFunction] for details.
    pub compare: CompareFunction,

    /// The wrapping method that a shadow sampler created from this descriptor will use when
    /// sampling a value at coordinates outside the range `0.0..=1.0` in the `S` ("width") direction
    /// in texture space.
    ///
    /// See [Wrap] for details.
    pub wrap_s: Wrap,

    /// The wrapping method that a shadow sampler created from this descriptor will use when
    /// sampling a value at coordinates outside the range `0.0..=1.0` in the `T` ("height")
    /// direction in texture space.
    ///
    /// See [Wrap] for details.
    pub wrap_t: Wrap,

    /// The wrapping method that a shadow sampler created from this descriptor will use when
    /// sampling a value at coordinates outside the range `0.0..=1.0` in the `R` ("depth") direction
    /// in texture space.
    ///
    /// See [Wrap] for details.
    pub wrap_r: Wrap,
}

impl Default for ShadowSamplerDescriptor {
    fn default() -> Self {
        ShadowSamplerDescriptor {
            compare: CompareFunction::LessOrEqual,
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            wrap_r: Wrap::Repeat,
        }
    }
}

/// Samples depth values and compares them to a reference value using a [CompareFunction].
///
/// A shadow sampler can only be used with a texture that uses a depth format or a depth stencil
/// stencil format. Rather than obtaining a (filtered) texel sample for certain texture coordinates
/// like a normal [Sampler], sampling with a [ShadowSampler] compares the depth texel values
/// closest to the sampled coordinates to a reference value using a [CompareFunction]. The value
/// returned is a floating point value in the range `0.0..=1.0` where the value signifies the
/// proportion of the texels that passed the [CompareFunction], for example: if no values passed, it
/// returns `0.0`; if all values passed it returns `1.0`; if 1 out of 4 samples passed it returns
/// `0.25`.
///
/// See the documentation for each of the [CompareFunction] variants for descriptions of how each
/// respective function decides whether or not a texel value passes.
///
/// See the documentation for [RenderingContext::create_shadow_sampler] for details on how to
/// create a [ShadowSampler].
pub struct ShadowSampler {
    data: Arc<SamplerData>,
    descriptor: ShadowSamplerDescriptor,
}

impl ShadowSampler {
    pub(crate) fn new<Rc>(context: &Rc, descriptor: &ShadowSamplerDescriptor) -> ShadowSampler
    where
        Rc: RenderingContext + Clone + 'static,
    {
        let data = Arc::new(SamplerData {
            id: None,
            context_id: context.id(),
            dropper: Box::new(context.clone()),
            extensions: context.extensions().clone(),
        });

        context.submit(ShadowSamplerAllocateCommand {
            data: data.clone(),
            descriptor: descriptor.clone(),
        });

        ShadowSampler {
            data,
            descriptor: descriptor.clone(),
        }
    }

    pub(crate) fn data(&self) -> &Arc<SamplerData> {
        &self.data
    }

    /// The [CompareFunction] used by this[ShadowSampler].
    ///
    /// See type documentation for [ShadowSampler] and the documentation for [CompareFunction] for
    /// details.
    pub fn compare(&self) -> CompareFunction {
        self.descriptor.compare
    }

    /// The wrapping method that this [ShadowSampler] uses when sampling a value at coordinates
    /// outside the range `0.0..=1.0` in the `S` ("width") direction in texture space.
    ///
    /// See [Wrap] for details.
    pub fn wrap_s(&self) -> Wrap {
        self.descriptor.wrap_s
    }

    /// The wrapping method that this [ShadowSampler] uses when sampling a value at coordinates
    /// outside the range `0.0..=1.0` in the `T` ("height") direction in texture space.
    ///
    /// See [Wrap] for details.
    pub fn wrap_t(&self) -> Wrap {
        self.descriptor.wrap_t
    }

    /// The wrapping method that this [ShadowSampler] uses when sampling a value at coordinates
    /// outside the range `0.0..=1.0` in the `R` ("depth") direction in texture space.
    ///
    /// See [Wrap] for details.
    pub fn wrap_r(&self) -> Wrap {
        self.descriptor.wrap_r
    }
}

trait SamplerObjectDropper {
    fn drop_sampler_object(&self, id: JsId);
}

impl<T> SamplerObjectDropper for T
where
    T: RenderingContext,
{
    fn drop_sampler_object(&self, id: JsId) {
        self.submit(SamplerDropCommand { id });
    }
}

pub(crate) struct SamplerData {
    id: Option<JsId>,
    context_id: usize,
    dropper: Box<SamplerObjectDropper>,
    extensions: Extensions,
}

impl SamplerData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }

    pub(crate) fn context_id(&self) -> usize {
        self.context_id
    }

    pub(crate) fn extensions(&self) -> &Extensions {
        &self.extensions
    }
}

impl Drop for SamplerData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.dropper.drop_sampler_object(id);
        }
    }
}

struct SamplerAllocateCommand {
    data: Arc<SamplerData>,
    descriptor: SamplerDescriptor,
}

unsafe impl GpuTask<Connection> for SamplerAllocateCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };
        let object = gl.create_sampler().unwrap();
        let descriptor = &self.descriptor;

        if descriptor.minification_filter != MinificationFilter::NearestMipmapLinear {
            gl.sampler_parameteri(
                &object,
                Gl::TEXTURE_MAG_FILTER,
                descriptor.minification_filter as i32,
            );
        }

        if descriptor.magnification_filter != MagnificationFilter::Linear {
            gl.sampler_parameteri(
                &object,
                Gl::TEXTURE_MAG_FILTER,
                descriptor.magnification_filter as i32,
            );
        }

        if descriptor.lod_range.min != -1000.0 {
            gl.sampler_parameterf(&object, Gl::TEXTURE_MIN_LOD, descriptor.lod_range.min);
        }

        if descriptor.lod_range.max != 1000.0 {
            gl.sampler_parameterf(&object, Gl::TEXTURE_MAX_LOD, descriptor.lod_range.max);
        }

        if descriptor.wrap_s != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::TEXTURE_WRAP_S, descriptor.wrap_s as i32);
        }

        if descriptor.wrap_t != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::TEXTURE_WRAP_T, descriptor.wrap_t as i32);
        }

        if descriptor.wrap_r != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::TEXTURE_WRAP_R, descriptor.wrap_r as i32);
        }

        data.id = Some(JsId::from_value(object.into()));

        Progress::Finished(())
    }
}

struct ShadowSamplerAllocateCommand {
    data: Arc<SamplerData>,
    descriptor: ShadowSamplerDescriptor,
}

unsafe impl GpuTask<Connection> for ShadowSamplerAllocateCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };
        let object = gl.create_sampler().unwrap();
        let descriptor = &self.descriptor;

        if descriptor.compare != CompareFunction::LessOrEqual {
            gl.sampler_parameteri(&object, Gl::TEXTURE_COMPARE_FUNC, descriptor.compare as i32);
        }

        if descriptor.wrap_s != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::TEXTURE_WRAP_S, descriptor.wrap_s as i32);
        }

        if descriptor.wrap_t != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::TEXTURE_WRAP_T, descriptor.wrap_t as i32);
        }

        if descriptor.wrap_r != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::TEXTURE_WRAP_R, descriptor.wrap_r as i32);
        }

        data.id = Some(JsId::from_value(object.into()));

        Progress::Finished(())
    }
}

struct SamplerDropCommand {
    id: JsId,
}

unsafe impl GpuTask<Connection> for SamplerDropCommand {
    type Output = ();

    fn context_id(&self) -> ContextId {
        ContextId::Any
    }

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_sampler(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}
