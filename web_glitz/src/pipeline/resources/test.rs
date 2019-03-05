use std::borrow::Borrow;
use std::sync::Arc;
use crate::buffer::BufferData;
use crate::sampler::{SamplerData, FloatSampledTexture2DArray, FloatSampledTexture3D, FloatSampledTextureCube, IntegerSampledTexture2D, IntegerSampledTexture2DArray, IntegerSampledTexture3D, IntegerSampledTextureCube, UnsignedIntegerSampledTexture2D, UnsignedIntegerSampledTexture2DArray, UnsignedIntegerSampledTexture3D, UnsignedIntegerSampledTextureCube, ShadowSampledTexture2D, ShadowSampledTexture2DArray, ShadowSampledTextureCube};
use crate::image::texture_2d::Texture2DData;
use crate::image::texture_2d_array::Texture2DArrayData;
use crate::image::texture_3d::Texture3DData;
use crate::image::texture_cube::TextureCubeData;
use crate::pipeline::resources::pipeline_resource_descriptor::{ResourceSlot, PipelineResourceDescriptor};
use crate::pipeline::interface_block;
use crate::buffer::BufferView;
use crate::pipeline::interface_block::InterfaceBlock;
use crate::buffer::Buffer;
use crate::sampler::FloatSampledTexture2D;
use crate::image::format::TextureFormat;
use crate::image::format::FloatSamplable;
use crate::pipeline::resources::pipeline_resource_descriptor::SamplerKind;
use std::mem;


