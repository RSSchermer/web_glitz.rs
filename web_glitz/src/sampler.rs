use std::sync::Arc;

use crate::image::format::Filterable;
use crate::image::format::InvalidMagnificationFilter;
use crate::image::format::InvalidMinificationFilter;
use crate::image::format::{
    FloatSamplable, IntegerSamplable, ShadowSamplable, TextureFormat, UnsignedIntegerSamplable,
};
use crate::image::texture_2d::Texture2D;
use crate::image::texture_2d::Texture2DData;
use crate::image::texture_2d_array::Texture2DArray;
use crate::image::texture_2d_array::Texture2DArrayData;
use crate::image::texture_3d::Texture3D;
use crate::image::texture_3d::Texture3DData;
use crate::image::texture_cube::TextureCube;
use crate::image::texture_cube::TextureCubeData;
use crate::runtime::state::ContextUpdate;
use crate::runtime::{Connection, RenderingContext};
use crate::task::GpuTask;
use crate::task::Progress;
use crate::util::{arc_get_mut_unchecked, identical, JsId};
use std::convert::TryFrom;
use std::marker;
use std::ops::RangeInclusive;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MagnificationFilter {
    Nearest = Gl::NEAREST,
    Linear = Gl::LINEAR,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MinificationFilter {
    Nearest = Gl::NEAREST,
    Linear = Gl::LINEAR,
    NearestMipmapNearest = Gl::NEAREST_MIPMAP_NEAREST,
    NearestMipmapLinear = Gl::NEAREST_MIPMAP_LINEAR,
    LinearMipmapNearest = Gl::LINEAR_MIPMAP_NEAREST,
    LinearMipmapLinear = Gl::LINEAR_MIPMAP_LINEAR,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Wrap {
    ClampToEdge = Gl::CLAMP_TO_EDGE,
    MirroredRepeat = Gl::MIRRORED_REPEAT,
    Repeat = Gl::REPEAT,
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

pub struct SamplerDescriptor {
    pub minification_filter: MinificationFilter,
    pub magnification_filter: MagnificationFilter,
    pub lod_range: LODRange,
    pub wrap_s: Wrap,
    pub wrap_t: Wrap,
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
            context: Box::new(context.clone()),
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

    pub(crate) fn context(&self) -> impl RenderingContext {
        &self.data.context
    }

    pub fn minification_filter(&self) -> MinificationFilter {
        self.descriptor.minification_filter
    }

    pub fn magnification_filter(&self) -> MagnificationFilter {
        self.descriptor.magnification_filter
    }

    pub fn lod_range(&self) -> LODRange {
        self.descriptor.lod_range
    }

    pub fn wrap_s(&self) -> Wrap {
        self.descriptor.wrap_s
    }

    pub fn wrap_t(&self) -> Wrap {
        self.descriptor.wrap_t
    }

    pub fn wrap_r(&self) -> Wrap {
        self.descriptor.wrap_r
    }
}

pub enum IncompatibleSampler {
    ContextMismatch,
    InvalidMagnificationFilter(InvalidMagnificationFilter),
    InvalidMinificationFilter(InvalidMinificationFilter),
}

impl From<InvalidMagnificationFilter> for IncompatibleSampler {
    fn from(err: InvalidMagnificationFilter) -> Self {
        IncompatibleSampler::InvalidMagnificationFilter(err)
    }
}

impl From<InvalidMinificationFilter> for IncompatibleSampler {
    fn from(err: InvalidMinificationFilter) -> Self {
        IncompatibleSampler::InvalidMinificationFilter(err)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CompareFunction {
    Equal = Gl::EQUAL,
    NotEqual = GL::NOTEQUAL,
    Less = Gl::LESS,
    Greater = Gl::GREATER,
    LessOrEqual = Gl::LEQUAL,
    GreaterOrEqual = Gl::GEQUAL,
    Always = Gl::ALWAYS,
    Never = Gl::NEVER,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ShadowSamplerDescriptor {
    pub compare: CompareFunction,
    pub wrap_s: Wrap,
    pub wrap_t: Wrap,
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
            context: Box::new(context.clone()),
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

    pub(crate) fn context(&self) -> impl RenderingContext {
        &self.data.context
    }

    pub fn compare(&self) -> CompareFunction {
        self.descriptor.compare
    }

    pub fn wrap_s(&self) -> Wrap {
        self.descriptor.wrap_s
    }

    pub fn wrap_t(&self) -> Wrap {
        self.descriptor.wrap_t
    }

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
    context: Box<RenderingContext>,
}

impl SamplerData {
    pub(crate) fn id(&self) -> Option<JsId> {
        self.id
    }
}

impl Drop for SamplerData {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            self.context.drop_sampler_object(id);
        }
    }
}

struct SamplerAllocateCommand {
    data: Arc<SamplerData>,
    descriptor: SamplerDescriptor,
}

impl GpuTask<Connection> for SamplerAllocateCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
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
            gl.sampler_parameteri(&object, Gl::WRAP_S, descriptor.wrap_s as i32);
        }

        if descriptor.wrap_t != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::WRAP_T, descriptor.wrap_t as i32);
        }

        if descriptor.wrap_r != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::WRAP_R, descriptor.wrap_r as i32);
        }

        data.id = Some(JsId::from_value(object.into()));

        Progress::Finished(())
    }
}

struct ShadowSamplerAllocateCommand {
    data: Arc<SamplerData>,
    descriptor: ShadowSamplerDescriptor,
}

impl GpuTask<Connection> for ShadowSamplerAllocateCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, state) = unsafe { connection.unpack_mut() };
        let data = unsafe { arc_get_mut_unchecked(&mut self.data) };
        let object = gl.create_sampler().unwrap();
        let descriptor = &self.descriptor;

        if descriptor.compare != CompareFunction::LessOrEqual {
            gl.sampler_parameteri(&object, Gl::TEXTURE_COMPARE_FUNC, descriptor.compare as i32);
        }

        if descriptor.wrap_s != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::WRAP_S, descriptor.wrap_s as i32);
        }

        if descriptor.wrap_t != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::WRAP_T, descriptor.wrap_t as i32);
        }

        if descriptor.wrap_r != Wrap::Repeat {
            gl.sampler_parameteri(&object, Gl::WRAP_R, descriptor.wrap_r as i32);
        }

        data.id = Some(JsId::from_value(object.into()));

        Progress::Finished(())
    }
}

struct SamplerDropCommand {
    id: JsId,
}

impl GpuTask<Connection> for SamplerDropCommand {
    type Output = ();

    fn progress(&mut self, connection: &mut Connection) -> Progress<Self::Output> {
        let (gl, _) = unsafe { connection.unpack() };
        let value = unsafe { JsId::into_value(self.id) };

        gl.delete_sampler(Some(&value.unchecked_into()));

        Progress::Finished(())
    }
}

#[derive(Clone)]
pub struct FloatSampledTexture2D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> FloatSampledTexture2D<'a> {
    pub fn new<F>(
        texture: &'a Texture2D<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + FloatSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(FloatSampledTexture2D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct IntegerSampledTexture2D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> IntegerSampledTexture2D<'a> {
    pub fn new<F>(
        texture: &'a Texture2D<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + IntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(IntegerSampledTexture2D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct UnsignedIntegerSampledTexture2D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> UnsignedIntegerSampledTexture2D<'a> {
    pub fn new<F>(
        texture: &'a Texture2D<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + UnsignedIntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(UnsignedIntegerSampledTexture2D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct ShadowSampledTexture2D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> ShadowSampledTexture2D<'a> {
    pub fn new<F>(
        texture: &'a Texture2D<F>,
        sampler: &'a ShadowSampler,
    ) -> Result<Self, ContextMismatch>
    where
        F: TextureFormat + ShadowSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        Ok(ShadowSampledTexture2D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct FloatSampledTexture2DArray<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DArrayData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> FloatSampledTexture2DArray<'a> {
    pub fn new<F>(
        texture: &'a Texture2DArray<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + FloatSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(FloatSampledTexture2DArray {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct IntegerSampledTexture2DArray<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DArrayData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> IntegerSampledTexture2DArray<'a> {
    pub fn new<F>(
        texture: &'a Texture2DArray<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + IntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(IntegerSampledTexture2DArray {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct UnsignedIntegerSampledTexture2DArray<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DArrayData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> UnsignedIntegerSampledTexture2DArray<'a> {
    pub fn new<F>(
        texture: &'a Texture2DArray<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + UnsignedIntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(UnsignedIntegerSampledTexture2DArray {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct ShadowSampledTexture2DArray<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture2DArrayData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> ShadowSampledTexture2DArray<'a> {
    pub fn new<F>(
        texture: &'a Texture2DArray<F>,
        sampler: &'a ShadowSampler,
    ) -> Result<Self, ContextMismatch>
    where
        F: TextureFormat + ShadowSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        Ok(ShadowSampledTexture2DArray {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct FloatSampledTexture3D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture3DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> FloatSampledTexture3D<'a> {
    pub fn new<F>(
        texture: &'a Texture3D<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + FloatSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(FloatSampledTexture3D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct IntegerSampledTexture3D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture3DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> IntegerSampledTexture3D<'a> {
    pub fn new<F>(
        texture: &'a Texture3D<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + IntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(IntegerSampledTexture3D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct UnsignedIntegerSampledTexture3D<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<Texture3DData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> UnsignedIntegerSampledTexture3D<'a> {
    pub fn new<F>(
        texture: &'a Texture3D<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + UnsignedIntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(UnsignedIntegerSampledTexture3D {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct FloatSampledTextureCube<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> FloatSampledTextureCube<'a> {
    pub fn new<F>(
        texture: &'a TextureCube<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + FloatSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(FloatSampledTextureCube {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct IntegerSampledTextureCube<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> IntegerSampledTextureCube<'a> {
    pub fn new<F>(
        texture: &'a TextureCube<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + IntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(IntegerSampledTextureCube {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct UnsignedIntegerSampledTextureCube<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> UnsignedIntegerSampledTextureCube<'a> {
    pub fn new<F>(
        texture: &'a TextureCube<F>,
        sampler: &'a Sampler,
    ) -> Result<Self, IncompatibleSampler>
    where
        F: TextureFormat + UnsignedIntegerSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        F::validate_minification_filter(context, sampler.minification_filter())?;
        F::validate_magnification_filter(context, sampler.magnification_filter())?;

        Ok(UnsignedIntegerSampledTextureCube {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}

#[derive(Clone)]
pub struct ShadowSampledTextureCube<'a> {
    pub sampler_data: Arc<SamplerData>,
    pub texture_data: Arc<TextureCubeData>,
    _marker: marker::PhantomData<&'a ()>,
}

impl<'a> ShadowSampledTextureCube<'a> {
    pub fn new<F>(
        texture: &'a TextureCube<F>,
        sampler: &'a ShadowSampler,
    ) -> Result<Self, ContextMismatch>
    where
        F: TextureFormat + ShadowSamplable + 'static,
    {
        let context = sampler.context();

        if texture.data().context_id() != sampler.context().id() {
            return Err(IncompatibleSampler::ContextMismatch);
        }

        Ok(ShadowSampledTextureCube {
            sampler_data: sampler.data().clone(),
            texture_data: texture.data().clone(),
            _marker: marker::PhantomData,
        })
    }
}
