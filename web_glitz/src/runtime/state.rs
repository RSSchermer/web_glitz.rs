use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};

use fnv::{FnvHashMap, FnvHasher};

use js_sys::Uint32Array;

use wasm_bindgen::JsValue;

use web_sys::{
    WebGl2RenderingContext as Gl, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlRenderbuffer,
    WebGlSampler, WebGlTexture, WebGlTransformFeedback, WebGlVertexArrayObject,
};

use crate::pipeline::graphics::transform_feedback::layout_descriptor::TransformFeedbackVaryings;

use crate::pipeline::graphics::util::BufferDescriptor;
use crate::pipeline::graphics::vertex::index_buffer::IndexDataDescriptor;
use crate::pipeline::graphics::vertex::layout_descriptor::VertexAttributeSlotDescriptor;
use crate::pipeline::graphics::{
    BlendEquation, BlendFactor, CullingMode, DepthRange, PolygonOffset, StencilOperation,
    TestFunction, TransformFeedbackLayoutDescriptor, VertexAttributeType,
    VertexInputLayoutDescriptor, WindingOrder,
};
use crate::pipeline::resources::resource_slot::{
    ShaderResourceSlotDescriptor, TextureSamplerSlot, UniformBlockSlot,
};
use crate::pipeline::resources::{ResourceSlotIdentifier, SampledTextureType};
use crate::rendering::attachment::AttachmentData;
use crate::runtime::index_lru::IndexLRU;
use crate::util::{identical, JsId};
use std::ops::Deref;
use wasm_bindgen::convert::{IntoWasmAbi, RefFromWasmAbi};

pub struct DynamicState {
    framebuffer_cache: FnvHashMap<u64, (Framebuffer, [Option<JsId>; 17])>,
    vertex_array_cache: FnvHashMap<u64, (WebGlVertexArrayObject, [Option<JsId>; 17])>,
    program_cache: FnvHashMap<ProgramKey, Program>,
    read_framebuffer: WebGlFramebuffer,
    max_draw_buffers: usize,
    active_program: Option<u32>,
    bound_array_buffer: Option<u32>,
    bound_element_array_buffer: Option<u32>,
    bound_copy_read_buffer: Option<u32>,
    bound_copy_write_buffer: Option<u32>,
    bound_pixel_pack_buffer: Option<u32>,
    bound_pixel_unpack_buffer: Option<u32>,
    bound_transform_feedback_buffers: Vec<BufferRange<u32>>,
    active_uniform_buffer_index: u32,
    bound_uniform_buffers: Vec<BufferRange<u32>>,
    uniform_buffer_index_lru: IndexLRU,
    bound_draw_framebuffer: Option<u32>,
    bound_read_framebuffer: Option<u32>,
    bound_renderbuffer: Option<u32>,
    bound_texture_2d: Option<u32>,
    bound_texture_cube_map: Option<u32>,
    bound_texture_3d: Option<u32>,
    bound_texture_2d_array: Option<u32>,
    bound_samplers: Vec<Option<u32>>,
    texture_units_lru: IndexLRU,
    texture_units_textures: Vec<Option<u32>>,
    bound_vertex_array: Option<u32>,
    bound_transform_feedback: Option<u32>,
    active_texture: u32,
    clear_color: [f32; 4],
    clear_depth: f32,
    clear_stencil: i32,
    depth_test_enabled: bool,
    stencil_test_enabled: bool,
    scissor_test_enabled: bool,
    blend_enabled: bool,
    dither_enabled: bool,
    polygon_offset_fill_enabled: bool,
    sample_alpha_to_coverage_enabled: bool,
    sample_coverage_enabled: bool,
    rasterizer_discard_enabled: bool,
    //    read_buffer: ReadBuffer,
    depth_func: TestFunction,
    depth_mask: bool,
    depth_range: DepthRange,
    polygon_offset: PolygonOffset,
    stencil_func_front: TestFunction,
    stencil_ref_front: i32,
    stencil_test_mask_front: u32,
    stencil_func_back: TestFunction,
    stencil_ref_back: i32,
    stencil_test_mask_back: u32,
    stencil_op_fail_front: StencilOperation,
    stencil_op_zfail_front: StencilOperation,
    stencil_op_zpass_front: StencilOperation,
    stencil_op_fail_back: StencilOperation,
    stencil_op_zfail_back: StencilOperation,
    stencil_op_zpass_back: StencilOperation,
    stencil_write_mask_front: u32,
    stencil_write_mask_back: u32,
    blend_color: [f32; 4],
    blend_equation_rgb: BlendEquation,
    blend_equation_alpha: BlendEquation,
    blend_func_source_rgb: BlendFactor,
    blend_func_source_alpha: BlendFactor,
    blend_func_destination_rgb: BlendFactor,
    blend_func_destination_alpha: BlendFactor,
    //    color_mask: [bool;4],
    //    cull_face: CullFace,
    //    front_face: FrontFace,
    line_width: f32,
    //    pixel_pack_alignment: u32,
    pixel_unpack_alignment: i32,
    //    pixel_unpack_flip_y: bool,
    //    pixel_unpack_premultiply_alpha: bool,
    //    pixel_unpack_colorspace_conversion: ColorspaceConversion,
    //    pixel_pack_row_length: u32,
    //    pixel_pack_skip_pixels: u32,
    //    pixel_pack_skip_rows: u32,
    pixel_unpack_row_length: i32,
    pixel_unpack_image_height: i32,
    //    pixel_unpack_skip_pixels: u32,
    //    pixel_unpack_skip_rows: u32,
    //    pixel_unpack_skip_images: u32,
    //    sample_coverage: SampleCoverage,
    scissor: (i32, i32, u32, u32),
    viewport: (i32, i32, i32, i32),
    front_face: WindingOrder,
    cull_face: CullingMode,
}

impl DynamicState {
    pub(crate) fn framebuffer_cache_mut(&mut self) -> FramebufferCache {
        FramebufferCache { state: self }
    }

    pub(crate) fn vertex_array_cache_mut(&mut self) -> VertexArrayCache {
        VertexArrayCache { state: self }
    }

    pub(crate) fn program_cache_mut(&mut self) -> ProgramCache {
        ProgramCache { state: self }
    }

    pub(crate) fn bind_default_read_framebuffer(&mut self, gl: &Gl) {
        let current = unsafe {
            self.bound_read_framebuffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(
            current.as_ref().map(|v| v.deref()),
            Some(&self.read_framebuffer),
        ) {
            gl.bind_framebuffer(Gl::READ_FRAMEBUFFER, Some(&self.read_framebuffer));

            self.bound_read_framebuffer = Some((&self.read_framebuffer).into_abi());
        }
    }

    pub fn max_draw_buffers(&self) -> usize {
        self.max_draw_buffers
    }

    pub fn use_program<'a>(
        &mut self,
        program: Option<&'a WebGlProgram>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe { self.active_program.map(|abi| JsValue::ref_from_abi(abi)) };

        if !identical(program, current.as_ref().map(|v| v.deref())) {
            self.active_program = program.map(|p| p.into_abi());

            Some(move |context: &Gl| {
                context.use_program(program);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_array_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_array_buffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(buffer, current.as_ref().map(|v| v.deref())) {
            self.bound_array_buffer = buffer.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::ARRAY_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_element_array_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let do_update = if self.bound_vertex_array.is_some() {
            // If a vertex array is currently bound, then we don't really know what element array
            // buffer is currently active, so always update.

            true
        } else {
            let current = unsafe {
                self.bound_element_array_buffer
                    .map(|abi| JsValue::ref_from_abi(abi))
            };

            if !identical(buffer, current.as_ref().map(|v| v.deref())) {
                self.bound_element_array_buffer = buffer.map(|b| b.into_abi());

                true
            } else {
                false
            }
        };

        if do_update {
            Some(move |context: &Gl| {
                context.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_copy_read_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_copy_read_buffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(buffer, current.as_ref().map(|v| v.deref())) {
            self.bound_copy_read_buffer = buffer.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::COPY_READ_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_copy_write_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_copy_write_buffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(buffer, current.as_ref().map(|v| v.deref())) {
            self.bound_copy_write_buffer = buffer.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::COPY_WRITE_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_pixel_pack_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_pixel_pack_buffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(buffer, current.as_ref().map(|v| v.deref())) {
            self.bound_pixel_pack_buffer = buffer.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::PIXEL_PACK_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_pixel_unpack_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_pixel_unpack_buffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(buffer, current.as_ref().map(|v| v.deref())) {
            self.bound_pixel_unpack_buffer = buffer.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::PIXEL_UNPACK_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_transform_feedback_buffer_range<'a>(
        &mut self,
        index: u32,
        buffer_range: BufferRange<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_transform_feedback_buffers[index as usize]
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !buffer_range.identical(&current.as_ref().map(|b| b.deref())) {
            self.bound_transform_feedback_buffers[index as usize] =
                buffer_range.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                match buffer_range {
                    BufferRange::None => {
                        context.bind_buffer_base(Gl::TRANSFORM_FEEDBACK_BUFFER, index, None)
                    }
                    BufferRange::Full(buffer) => {
                        context.bind_buffer_base(Gl::TRANSFORM_FEEDBACK_BUFFER, index, Some(buffer))
                    }
                    BufferRange::OffsetSize(buffer, offset, size) => context
                        .bind_buffer_range_with_i32_and_i32(
                            Gl::TRANSFORM_FEEDBACK_BUFFER,
                            index,
                            Some(buffer),
                            offset as i32,
                            size as i32,
                        ),
                };

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn active_uniform_buffer_binding(&self) -> u32 {
        self.active_uniform_buffer_index
    }

    pub fn set_active_uniform_buffer_index(&mut self, index: u32) {
        self.uniform_buffer_index_lru.use_index(index as usize);
        self.active_uniform_buffer_index = index;
    }

    pub fn set_active_uniform_buffer_binding_lru(&mut self) {
        self.active_uniform_buffer_index = self.uniform_buffer_index_lru.use_lru_index() as u32;
    }

    pub fn bind_uniform_buffer_range<'a>(
        &mut self,
        buffer_range: BufferRange<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let index = self.active_uniform_buffer_index;

        let current = unsafe {
            self.bound_uniform_buffers[index as usize].map(|abi| JsValue::ref_from_abi(abi))
        };

        if !buffer_range.identical(&current.as_ref().map(|b| b.deref())) {
            self.bound_uniform_buffers[index as usize] = buffer_range.map(|b| b.into_abi());

            Some(move |context: &Gl| {
                match buffer_range {
                    BufferRange::None => context.bind_buffer_base(Gl::UNIFORM_BUFFER, index, None),
                    BufferRange::Full(buffer) => {
                        context.bind_buffer_base(Gl::UNIFORM_BUFFER, index, Some(buffer))
                    }
                    BufferRange::OffsetSize(buffer, offset, size) => context
                        .bind_buffer_range_with_i32_and_i32(
                            Gl::UNIFORM_BUFFER,
                            index,
                            Some(buffer),
                            offset as i32,
                            size as i32,
                        ),
                };

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_draw_framebuffer<'a>(
        &mut self,
        framebuffer: Option<&'a WebGlFramebuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_draw_framebuffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(framebuffer, current.as_ref().map(|v| v.deref())) {
            self.bound_draw_framebuffer = framebuffer.map(|f| f.into_abi());

            Some(move |context: &Gl| {
                context.bind_framebuffer(Gl::DRAW_FRAMEBUFFER, framebuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_read_framebuffer<'a>(
        &mut self,
        framebuffer: Option<&'a WebGlFramebuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_read_framebuffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(framebuffer, current.as_ref().map(|v| v.deref())) {
            self.bound_read_framebuffer = framebuffer.map(|f| f.into_abi());

            Some(move |context: &Gl| {
                context.bind_framebuffer(Gl::READ_FRAMEBUFFER, framebuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_renderbuffer<'a>(
        &mut self,
        renderbuffer: Option<&'a WebGlRenderbuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_renderbuffer
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(renderbuffer, current.as_ref().map(|v| v.deref())) {
            self.bound_renderbuffer = renderbuffer.map(|r| r.into_abi());

            Some(move |context: &Gl| {
                context.bind_renderbuffer(Gl::RENDERBUFFER, renderbuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_texture_2d<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe { self.bound_texture_2d.map(|abi| JsValue::ref_from_abi(abi)) };

        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        let unit_current = unsafe { active_unit_texture.map(|abi| JsValue::ref_from_abi(abi)) };

        if !identical(texture, current.as_ref().map(|v| v.deref()))
            || !identical(texture, unit_current.as_ref().map(|v| v.deref()))
        {
            self.bound_texture_2d = texture.map(|t| t.into_abi());
            *active_unit_texture = texture.map(|t| t.into_abi());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_2D, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_texture_2d_array<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_texture_2d_array
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        let unit_current = unsafe { active_unit_texture.map(|abi| JsValue::ref_from_abi(abi)) };

        if !identical(texture, current.as_ref().map(|v| v.deref()))
            || !identical(texture, unit_current.as_ref().map(|v| v.deref()))
        {
            self.bound_texture_2d_array = texture.map(|t| t.into_abi());
            *active_unit_texture = texture.map(|t| t.into_abi());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_2D_ARRAY, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_texture_3d<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe { self.bound_texture_3d.map(|abi| JsValue::ref_from_abi(abi)) };

        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        let unit_current = unsafe { active_unit_texture.map(|abi| JsValue::ref_from_abi(abi)) };

        if !identical(texture, current.as_ref().map(|v| v.deref()))
            || !identical(texture, unit_current.as_ref().map(|v| v.deref()))
        {
            self.bound_texture_3d = texture.map(|t| t.into_abi());
            *active_unit_texture = texture.map(|t| t.into_abi());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_3D, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_texture_cube_map<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_texture_cube_map
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        let unit_current = unsafe { active_unit_texture.map(|abi| JsValue::ref_from_abi(abi)) };

        if !identical(texture, current.as_ref().map(|v| v.deref()))
            || !identical(texture, unit_current.as_ref().map(|v| v.deref()))
        {
            self.bound_texture_cube_map = texture.map(|t| t.into_abi());
            *active_unit_texture = texture.map(|t| t.into_abi());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_CUBE_MAP, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_sampler<'a>(
        &mut self,
        texture_unit: u32,
        sampler: Option<&'a WebGlSampler>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_samplers[texture_unit as usize].map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(sampler, current.as_ref().map(|v| v.deref())) {
            self.bound_samplers[texture_unit as usize] = sampler.map(|v| v.into_abi());

            Some(move |context: &Gl| {
                context.bind_sampler(texture_unit, sampler);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_vertex_array<'a>(
        &mut self,
        vertex_array: Option<&'a WebGlVertexArrayObject>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_vertex_array
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(vertex_array, current.as_ref().map(|v| v.deref())) {
            self.bound_vertex_array = vertex_array.map(|v| v.into_abi());

            Some(move |context: &Gl| {
                context.bind_vertex_array(vertex_array);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bind_transform_feedback<'a>(
        &mut self,
        transform_feedback: Option<&'a WebGlTransformFeedback>,
    ) -> impl ContextUpdate<'a, ()> {
        let current = unsafe {
            self.bound_transform_feedback
                .map(|abi| JsValue::ref_from_abi(abi))
        };

        if !identical(transform_feedback, current.as_ref().map(|v| v.deref())) {
            self.bound_transform_feedback = transform_feedback.map(|tf| tf.into_abi());

            Some(move |context: &Gl| {
                context.bind_transform_feedback(Gl::TRANSFORM_FEEDBACK, transform_feedback);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn active_texture(&self) -> u32 {
        self.active_texture
    }

    pub fn set_active_texture(&mut self, texture_unit: u32) -> impl ContextUpdate<'static, ()> {
        if texture_unit != self.active_texture {
            self.active_texture = texture_unit;
            self.texture_units_lru.use_index(texture_unit as usize);

            Some(move |context: &Gl| {
                context.active_texture(Gl::TEXTURE0 + texture_unit);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn set_active_texture_lru(&mut self) -> impl ContextUpdate<'static, ()> {
        let texture_unit = self.texture_units_lru.use_lru_index();
        self.active_texture = texture_unit as u32;

        Some(move |context: &Gl| {
            context.active_texture(Gl::TEXTURE0 + texture_unit as u32);

            Ok(())
        })
    }

    pub fn clear_color(&self) -> [f32; 4] {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, color: [f32; 4]) -> impl ContextUpdate<'static, ()> {
        if color != self.clear_color {
            self.clear_color = color;

            Some(move |context: &Gl| {
                context.clear_color(color[0], color[1], color[2], color[3]);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn clear_depth(&self) -> f32 {
        self.clear_depth
    }

    pub fn set_clear_depth(&mut self, depth: f32) -> impl ContextUpdate<'static, ()> {
        if depth != self.clear_depth {
            self.clear_depth = depth;

            Some(move |context: &Gl| {
                context.clear_depth(depth);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn clear_stencil(&self) -> i32 {
        self.clear_stencil
    }

    pub fn set_clear_stencil(&mut self, stencil: i32) -> impl ContextUpdate<'static, ()> {
        if stencil != self.clear_stencil {
            self.clear_stencil = stencil;

            Some(move |context: &Gl| {
                context.clear_stencil(stencil);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn pixel_unpack_alignment(&self) -> i32 {
        self.pixel_unpack_alignment
    }

    pub fn set_pixel_unpack_alignment(
        &mut self,
        pixel_unpack_alignment: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if pixel_unpack_alignment != self.pixel_unpack_alignment {
            self.pixel_unpack_alignment = pixel_unpack_alignment;

            Some(move |context: &Gl| {
                context.pixel_storei(Gl::UNPACK_ALIGNMENT, pixel_unpack_alignment);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn pixel_unpack_row_length(&self) -> i32 {
        self.pixel_unpack_row_length
    }

    pub fn set_pixel_unpack_row_length(
        &mut self,
        pixel_unpack_row_length: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if pixel_unpack_row_length != self.pixel_unpack_row_length {
            self.pixel_unpack_row_length = pixel_unpack_row_length;

            Some(move |context: &Gl| {
                context.pixel_storei(Gl::UNPACK_ROW_LENGTH, pixel_unpack_row_length);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn pixel_unpack_image_height(&self) -> i32 {
        self.pixel_unpack_image_height
    }

    pub fn set_pixel_unpack_image_height(
        &mut self,
        pixel_unpack_image_height: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if pixel_unpack_image_height != self.pixel_unpack_image_height {
            self.pixel_unpack_image_height = pixel_unpack_image_height;

            Some(move |context: &Gl| {
                context.pixel_storei(Gl::UNPACK_IMAGE_HEIGHT, pixel_unpack_image_height);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn line_width(&self) -> f32 {
        self.line_width
    }

    pub fn set_line_width(&mut self, line_width: f32) -> impl ContextUpdate<'static, ()> {
        if line_width != self.line_width {
            self.line_width = line_width;

            Some(move |context: &Gl| {
                context.line_width(line_width);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn depth_test_enabled(&self) -> bool {
        self.depth_test_enabled
    }

    pub fn set_depth_test_enabled(
        &mut self,
        depth_test_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if depth_test_enabled != self.depth_test_enabled {
            self.depth_test_enabled = depth_test_enabled;

            Some(move |context: &Gl| {
                if depth_test_enabled {
                    context.enable(Gl::DEPTH_TEST);
                } else {
                    context.disable(Gl::DEPTH_TEST);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_test_enabled(&self) -> bool {
        self.stencil_test_enabled
    }

    pub fn set_stencil_test_enabled(
        &mut self,
        stencil_test_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if stencil_test_enabled != self.stencil_test_enabled {
            self.stencil_test_enabled = stencil_test_enabled;

            Some(move |context: &Gl| {
                if stencil_test_enabled {
                    context.enable(Gl::STENCIL_TEST);
                } else {
                    context.disable(Gl::STENCIL_TEST);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn scissor_test_enabled(&self) -> bool {
        self.scissor_test_enabled
    }

    pub fn set_scissor_test_enabled(
        &mut self,
        scissor_test_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if scissor_test_enabled != self.scissor_test_enabled {
            self.scissor_test_enabled = scissor_test_enabled;

            Some(move |context: &Gl| {
                if scissor_test_enabled {
                    context.enable(Gl::SCISSOR_TEST);
                } else {
                    context.disable(Gl::SCISSOR_TEST);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn blend_enabled(&self) -> bool {
        self.blend_enabled
    }

    pub fn set_blend_enabled(&mut self, blend_enabled: bool) -> impl ContextUpdate<'static, ()> {
        if blend_enabled != self.blend_enabled {
            self.blend_enabled = blend_enabled;

            Some(move |context: &Gl| {
                if blend_enabled {
                    context.enable(Gl::BLEND);
                } else {
                    context.disable(Gl::BLEND);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn dither_enabled(&self) -> bool {
        self.dither_enabled
    }

    pub fn set_dither_enabled(&mut self, dither_enabled: bool) -> impl ContextUpdate<'static, ()> {
        if dither_enabled != self.dither_enabled {
            self.dither_enabled = dither_enabled;

            Some(move |context: &Gl| {
                if dither_enabled {
                    context.enable(Gl::DITHER);
                } else {
                    context.disable(Gl::DITHER);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn polygon_offset_fill_enabled(&self) -> bool {
        self.polygon_offset_fill_enabled
    }

    pub fn set_polygon_offset_fill_enabled(
        &mut self,
        polygon_offset_fill_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if polygon_offset_fill_enabled != self.polygon_offset_fill_enabled {
            self.polygon_offset_fill_enabled = polygon_offset_fill_enabled;

            Some(move |context: &Gl| {
                if polygon_offset_fill_enabled {
                    context.enable(Gl::POLYGON_OFFSET_FILL);
                } else {
                    context.disable(Gl::POLYGON_OFFSET_FILL);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn sample_aplha_to_coverage_enabled(&self) -> bool {
        self.sample_alpha_to_coverage_enabled
    }

    pub fn set_sample_aplha_to_coverage_enabled(
        &mut self,
        sample_aplha_to_coverage_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if sample_aplha_to_coverage_enabled != self.sample_alpha_to_coverage_enabled {
            self.sample_alpha_to_coverage_enabled = sample_aplha_to_coverage_enabled;

            Some(move |context: &Gl| {
                if sample_aplha_to_coverage_enabled {
                    context.enable(Gl::SAMPLE_ALPHA_TO_COVERAGE);
                } else {
                    context.disable(Gl::SAMPLE_ALPHA_TO_COVERAGE);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn sample_coverage_enabled(&self) -> bool {
        self.sample_coverage_enabled
    }

    pub fn set_sample_coverage_enabled(
        &mut self,
        sample_coverage_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if sample_coverage_enabled != self.sample_coverage_enabled {
            self.sample_coverage_enabled = sample_coverage_enabled;

            Some(move |context: &Gl| {
                if sample_coverage_enabled {
                    context.enable(Gl::SAMPLE_COVERAGE);
                } else {
                    context.disable(Gl::SAMPLE_COVERAGE);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn rasterizer_discard_enabled(&self) -> bool {
        self.rasterizer_discard_enabled
    }

    pub fn set_rasterizer_discard_enabled(
        &mut self,
        rasterizer_discard_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if rasterizer_discard_enabled != self.rasterizer_discard_enabled {
            self.rasterizer_discard_enabled = rasterizer_discard_enabled;

            Some(move |context: &Gl| {
                if rasterizer_discard_enabled {
                    context.enable(Gl::RASTERIZER_DISCARD);
                } else {
                    context.disable(Gl::RASTERIZER_DISCARD);
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn set_scissor_rect(
        &mut self,
        value: (i32, i32, u32, u32),
    ) -> impl ContextUpdate<'static, ()> {
        if self.scissor != value {
            self.scissor = value;

            Some(move |context: &Gl| {
                let (x, y, width, height) = value;

                context.scissor(x, y, width as i32, height as i32);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn depth_func(&self) -> TestFunction {
        self.depth_func
    }

    pub fn set_depth_func(&mut self, depth_func: TestFunction) -> impl ContextUpdate<'static, ()> {
        if self.depth_func != depth_func {
            self.depth_func = depth_func;

            Some(move |context: &Gl| {
                context.depth_func(depth_func.id());

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn depth_mask(&self) -> bool {
        self.depth_mask
    }

    pub fn set_depth_mask(&mut self, depth_mask: bool) -> impl ContextUpdate<'static, ()> {
        if self.depth_mask != depth_mask {
            self.depth_mask = depth_mask;

            Some(move |context: &Gl| {
                context.depth_mask(depth_mask);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn depth_range(&self) -> &DepthRange {
        &self.depth_range
    }

    pub fn set_depth_range(&mut self, depth_range: DepthRange) -> impl ContextUpdate<'static, ()> {
        if &self.depth_range != &depth_range {
            let near = depth_range.near();
            let far = depth_range.far();

            self.depth_range = depth_range;

            Some(move |context: &Gl| {
                context.depth_range(near, far);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn polygon_offset(&self) -> &PolygonOffset {
        &self.polygon_offset
    }

    pub fn set_polygon_offset(
        &mut self,
        polygon_offset: PolygonOffset,
    ) -> impl ContextUpdate<'static, ()> {
        if &self.polygon_offset != &polygon_offset {
            let factor = polygon_offset.factor;
            let units = polygon_offset.units;

            self.polygon_offset = polygon_offset;

            Some(move |context: &Gl| {
                context.polygon_offset(factor, units);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_func_front(&self) -> TestFunction {
        self.stencil_func_front
    }

    pub fn stencil_ref_front(&self) -> i32 {
        self.stencil_ref_front
    }

    pub fn stencil_test_mask_front(&self) -> u32 {
        self.stencil_test_mask_front
    }

    pub fn set_stencil_func_front(
        &mut self,
        func: TestFunction,
        reference: i32,
        mask: u32,
    ) -> impl ContextUpdate<'static, ()> {
        if self.stencil_func_front != func
            || self.stencil_ref_front != reference
            || self.stencil_test_mask_front != mask
        {
            self.stencil_func_front = func;
            self.stencil_ref_front = reference;
            self.stencil_test_mask_front = mask;

            Some(move |context: &Gl| {
                context.stencil_func_separate(Gl::FRONT, func.id(), reference, mask);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_func_back(&self) -> TestFunction {
        self.stencil_func_back
    }

    pub fn stencil_ref_back(&self) -> i32 {
        self.stencil_ref_back
    }

    pub fn stencil_test_mask_back(&self) -> u32 {
        self.stencil_test_mask_back
    }

    pub fn set_stencil_func_back(
        &mut self,
        func: TestFunction,
        reference: i32,
        mask: u32,
    ) -> impl ContextUpdate<'static, ()> {
        if self.stencil_func_back != func
            || self.stencil_ref_back != reference
            || self.stencil_test_mask_back != mask
        {
            self.stencil_func_back = func;
            self.stencil_ref_back = reference;
            self.stencil_test_mask_back = mask;

            Some(move |context: &Gl| {
                context.stencil_func_separate(Gl::BACK, func.id(), reference, mask);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_op_fail_front(&self) -> StencilOperation {
        self.stencil_op_fail_front
    }

    pub fn stencil_op_zfail_front(&self) -> StencilOperation {
        self.stencil_op_zfail_front
    }

    pub fn stencil_op_zpass_front(&self) -> StencilOperation {
        self.stencil_op_zpass_front
    }

    pub fn set_stencil_op_front(
        &mut self,
        fail: StencilOperation,
        zfail: StencilOperation,
        zpass: StencilOperation,
    ) -> impl ContextUpdate<'static, ()> {
        if self.stencil_op_fail_front != fail
            || self.stencil_op_zfail_front != zfail
            || self.stencil_op_zpass_front != zpass
        {
            self.stencil_op_fail_front = fail;
            self.stencil_op_zfail_front = zfail;
            self.stencil_op_zpass_front = zpass;

            Some(move |context: &Gl| {
                context.stencil_op_separate(Gl::FRONT, fail.id(), zfail.id(), zpass.id());

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_op_fail_back(&self) -> StencilOperation {
        self.stencil_op_fail_back
    }

    pub fn stencil_op_zfail_back(&self) -> StencilOperation {
        self.stencil_op_zfail_back
    }

    pub fn stencil_op_zpass_back(&self) -> StencilOperation {
        self.stencil_op_zpass_back
    }

    pub fn set_stencil_op_back(
        &mut self,
        fail: StencilOperation,
        zfail: StencilOperation,
        zpass: StencilOperation,
    ) -> impl ContextUpdate<'static, ()> {
        if self.stencil_op_fail_back != fail
            || self.stencil_op_zfail_back != zfail
            || self.stencil_op_zpass_back != zpass
        {
            self.stencil_op_fail_back = fail;
            self.stencil_op_zfail_back = zfail;
            self.stencil_op_zpass_back = zpass;

            Some(move |context: &Gl| {
                context.stencil_op_separate(Gl::BACK, fail.id(), zfail.id(), zpass.id());

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_write_mask_front(&self) -> u32 {
        self.stencil_write_mask_front
    }

    pub fn set_stencil_write_mask_front(&mut self, mask: u32) -> impl ContextUpdate<'static, ()> {
        if self.stencil_write_mask_front != mask {
            self.stencil_write_mask_front = mask;

            Some(move |context: &Gl| {
                context.stencil_mask_separate(Gl::FRONT, mask);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn stencil_write_mask_back(&self) -> u32 {
        self.stencil_write_mask_back
    }

    pub fn set_stencil_write_mask_back(&mut self, mask: u32) -> impl ContextUpdate<'static, ()> {
        if self.stencil_write_mask_back != mask {
            self.stencil_write_mask_back = mask;

            Some(move |context: &Gl| {
                context.stencil_mask_separate(Gl::BACK, mask);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn blend_color(&self) -> [f32; 4] {
        self.blend_color
    }

    pub fn set_blend_color(&mut self, blend_color: [f32; 4]) -> impl ContextUpdate<'static, ()> {
        if self.blend_color != blend_color {
            self.blend_color = blend_color;

            Some(move |context: &Gl| {
                let [r, g, b, a] = blend_color;

                context.blend_color(r, g, b, a);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn blend_equation_rgb(&self) -> BlendEquation {
        self.blend_equation_rgb
    }

    pub fn blend_equation_alpha(&self) -> BlendEquation {
        self.blend_equation_alpha
    }

    pub fn set_blend_equations(
        &mut self,
        rgb: BlendEquation,
        alpha: BlendEquation,
    ) -> impl ContextUpdate<'static, ()> {
        if self.blend_equation_rgb != rgb || self.blend_equation_alpha != alpha {
            self.blend_equation_rgb = rgb;
            self.blend_equation_alpha = alpha;

            Some(move |context: &Gl| {
                context.blend_equation_separate(rgb.id(), alpha.id());

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn blend_func_source_rgb(&self) -> BlendFactor {
        self.blend_func_source_rgb
    }

    pub fn blend_func_source_alpha(&self) -> BlendFactor {
        self.blend_func_source_alpha
    }

    pub fn blend_func_destination_rgb(&self) -> BlendFactor {
        self.blend_func_destination_rgb
    }

    pub fn blend_func_destination_alpha(&self) -> BlendFactor {
        self.blend_func_destination_alpha
    }

    pub fn set_blend_func(
        &mut self,
        source_rgb: BlendFactor,
        destination_rgb: BlendFactor,
        source_alpha: BlendFactor,
        destination_alpha: BlendFactor,
    ) -> impl ContextUpdate<'static, ()> {
        if self.blend_func_source_rgb != source_rgb
            || self.blend_func_source_alpha != source_alpha
            || self.blend_func_destination_rgb != destination_rgb
            || self.blend_func_destination_alpha != destination_alpha
        {
            self.blend_func_source_rgb = source_rgb;
            self.blend_func_source_alpha = source_alpha;
            self.blend_func_destination_rgb = destination_rgb;
            self.blend_func_destination_alpha = destination_alpha;

            Some(move |context: &Gl| {
                context.blend_func_separate(
                    source_rgb.id(),
                    destination_rgb.id(),
                    source_alpha.id(),
                    destination_alpha.id(),
                );

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn viewport(&self) -> &(i32, i32, i32, i32) {
        &self.viewport
    }

    pub fn set_viewport(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> impl ContextUpdate<'static, ()> {
        if self.viewport != (x, y, width, height) {
            self.viewport = (x, y, width, height);

            Some(move |context: &Gl| {
                context.viewport(x, y, width, height);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn front_face(&self) -> WindingOrder {
        self.front_face
    }

    pub fn set_front_face(&mut self, front_face: WindingOrder) -> impl ContextUpdate<'static, ()> {
        if self.front_face != front_face {
            self.front_face = front_face;

            Some(move |context: &Gl| {
                match front_face {
                    WindingOrder::CounterClockwise => context.front_face(Gl::CCW),
                    WindingOrder::Clockwise => context.front_face(Gl::CW),
                }

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn cull_face(&self) -> CullingMode {
        self.cull_face
    }

    pub fn set_cull_face(&mut self, cull_face: CullingMode) -> impl ContextUpdate<'static, ()> {
        if self.cull_face != cull_face {
            let is_enabled = self.cull_face != CullingMode::None;

            self.cull_face = cull_face;

            Some(move |context: &Gl| {
                match cull_face {
                    CullingMode::None => context.disable(Gl::CULL_FACE),
                    CullingMode::Both => {
                        if !is_enabled {
                            context.enable(Gl::CULL_FACE)
                        }

                        context.cull_face(Gl::FRONT_AND_BACK)
                    }
                    CullingMode::Front => {
                        if !is_enabled {
                            context.enable(Gl::CULL_FACE)
                        }

                        context.cull_face(Gl::FRONT)
                    }
                    CullingMode::Back => {
                        if !is_enabled {
                            context.enable(Gl::CULL_FACE)
                        }

                        context.cull_face(Gl::BACK)
                    }
                }

                Ok(())
            })
        } else {
            None
        }
    }
}

impl DynamicState {
    pub fn initial(context: &Gl) -> Self {
        let max_combined_texture_image_units = context
            .get_parameter(Gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            .unwrap()
            .as_f64()
            .unwrap() as usize;

        DynamicState {
            framebuffer_cache: FnvHashMap::default(),
            vertex_array_cache: FnvHashMap::default(),
            program_cache: FnvHashMap::default(),
            read_framebuffer: context.create_framebuffer().unwrap(),
            max_draw_buffers: context
                .get_parameter(Gl::MAX_DRAW_BUFFERS)
                .unwrap()
                .as_f64()
                .unwrap() as usize,
            active_program: None,
            bound_array_buffer: None,
            bound_element_array_buffer: None,
            bound_copy_read_buffer: None,
            bound_copy_write_buffer: None,
            bound_pixel_pack_buffer: None,
            bound_pixel_unpack_buffer: None,
            bound_transform_feedback_buffers: vec![
                BufferRange::None;
                context
                    .get_parameter(Gl::MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS)
                    .unwrap()
                    .as_f64()
                    .unwrap() as usize
            ],
            bound_uniform_buffers: vec![
                BufferRange::None;
                context
                    .get_parameter(Gl::MAX_UNIFORM_BUFFER_BINDINGS)
                    .unwrap()
                    .as_f64()
                    .unwrap() as usize
            ],
            active_uniform_buffer_index: 0,
            uniform_buffer_index_lru: IndexLRU::new(
                context
                    .get_parameter(Gl::MAX_UNIFORM_BUFFER_BINDINGS)
                    .unwrap()
                    .as_f64()
                    .unwrap() as usize,
            ),
            bound_draw_framebuffer: None,
            bound_read_framebuffer: None,
            bound_renderbuffer: None,
            bound_texture_2d: None,
            bound_texture_cube_map: None,
            bound_texture_3d: None,
            bound_texture_2d_array: None,
            bound_samplers: vec![None; max_combined_texture_image_units],
            texture_units_lru: IndexLRU::new(max_combined_texture_image_units),
            texture_units_textures: vec![None; max_combined_texture_image_units],
            bound_vertex_array: None,
            bound_transform_feedback: None,
            active_texture: 0,
            clear_color: [0.0, 0.0, 0.0, 0.0],
            clear_depth: 1.0,
            clear_stencil: 0,
            pixel_unpack_alignment: 4,
            pixel_unpack_row_length: 0,
            pixel_unpack_image_height: 0,
            depth_test_enabled: false,
            stencil_test_enabled: false,
            scissor_test_enabled: false,
            blend_enabled: false,
            dither_enabled: true,
            polygon_offset_fill_enabled: false,
            sample_alpha_to_coverage_enabled: false,
            sample_coverage_enabled: false,
            rasterizer_discard_enabled: false,
            scissor: (0, 0, 0, 0),
            viewport: (
                0,
                0,
                context.drawing_buffer_width(),
                context.drawing_buffer_height(),
            ),
            depth_func: TestFunction::Less,
            depth_mask: true,
            depth_range: DepthRange::default(),
            polygon_offset: PolygonOffset::default(),
            stencil_func_front: TestFunction::AlwaysPass,
            stencil_ref_front: 0,
            stencil_test_mask_front: 0xffffffff,
            stencil_func_back: TestFunction::AlwaysPass,
            stencil_ref_back: 0,
            stencil_test_mask_back: 0xffffffff,
            stencil_op_fail_front: StencilOperation::Keep,
            stencil_op_zfail_front: StencilOperation::Keep,
            stencil_op_zpass_front: StencilOperation::Keep,
            stencil_op_fail_back: StencilOperation::Keep,
            stencil_op_zfail_back: StencilOperation::Keep,
            stencil_op_zpass_back: StencilOperation::Keep,
            stencil_write_mask_front: 0xffffffff,
            stencil_write_mask_back: 0xffffffff,
            blend_color: [0.0; 4],
            blend_equation_rgb: BlendEquation::Addition,
            blend_equation_alpha: BlendEquation::Addition,
            blend_func_source_rgb: BlendFactor::One,
            blend_func_source_alpha: BlendFactor::One,
            blend_func_destination_rgb: BlendFactor::Zero,
            blend_func_destination_alpha: BlendFactor::Zero,
            line_width: 1.0,
            front_face: WindingOrder::CounterClockwise,
            cull_face: CullingMode::None,
        }
    }
}

pub trait ContextUpdate<'a, E> {
    fn apply(self, context: &Gl) -> Result<(), E>;
}

impl<'a, F, E> ContextUpdate<'a, E> for Option<F>
where
    F: FnOnce(&Gl) -> Result<(), E> + 'a,
{
    fn apply(self, context: &Gl) -> Result<(), E> {
        self.map(|f| f(context)).unwrap_or(Ok(()))
    }
}

#[derive(Clone, Copy)]
pub enum BufferRange<T> {
    None,
    Full(T),
    OffsetSize(T, u32, u32),
}

impl<T> BufferRange<T> {
    pub fn as_ref(&self) -> BufferRange<&T> {
        match *self {
            BufferRange::None => BufferRange::None,
            BufferRange::Full(ref buffer) => BufferRange::Full(buffer),
            BufferRange::OffsetSize(ref buffer, offset, size) => {
                BufferRange::OffsetSize(buffer, offset, size)
            }
        }
    }

    fn map<U, F>(self, f: F) -> BufferRange<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            BufferRange::None => BufferRange::None,
            BufferRange::Full(buffer) => BufferRange::Full(f(buffer)),
            BufferRange::OffsetSize(buffer, offset, size) => {
                BufferRange::OffsetSize(f(buffer), offset, size)
            }
        }
    }

    fn identical<U>(&self, other: &BufferRange<U>) -> bool
    where
        T: AsRef<JsValue>,
        U: AsRef<JsValue>,
    {
        match (self, other) {
            (BufferRange::None, BufferRange::None) => true,
            (BufferRange::Full(a), BufferRange::Full(b)) => a.as_ref() == b.as_ref(),
            (
                BufferRange::OffsetSize(a, offset_a, size_a),
                BufferRange::OffsetSize(b, offset_b, size_b),
            ) => offset_a == offset_b && size_a == size_b && a.as_ref() == b.as_ref(),
            _ => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DrawBuffer {
    Color0 = 0,
    Color1 = 1,
    Color2 = 2,
    Color3 = 3,
    Color4 = 4,
    Color5 = 5,
    Color6 = 6,
    Color7 = 7,
    Color8 = 8,
    Color9 = 9,
    Color10 = 10,
    Color11 = 11,
    Color12 = 12,
    Color13 = 13,
    Color14 = 14,
    Color15 = 15,
    None = 16,
}

impl DrawBuffer {
    fn id(&self) -> u32 {
        match self {
            DrawBuffer::None => Gl::NONE,
            _ => Gl::COLOR_ATTACHMENT0 + *self as u32,
        }
    }
}

pub(crate) struct Framebuffer {
    fbo: WebGlFramebuffer,
    draw_buffers: [DrawBuffer; 16],
}

pub(crate) struct CachedFramebuffer<'a> {
    framebuffer: &'a mut Framebuffer,
    max_draw_buffers: usize,
    gl: &'a Gl,
}

impl<'a> CachedFramebuffer<'a> {
    pub(crate) fn set_draw_buffers<I, B>(&mut self, draw_buffers: I)
    where
        I: IntoIterator<Item = B>,
        B: Borrow<DrawBuffer>,
    {
        let framebuffer = &mut self.framebuffer;

        let mut needs_update = false;
        let mut buffer_count = 0;

        for buffer in draw_buffers {
            if buffer_count >= self.max_draw_buffers {
                panic!(
                    "Cannot bind more than {} draw buffers",
                    self.max_draw_buffers
                );
            }

            let buffer = *buffer.borrow();

            if buffer != framebuffer.draw_buffers[buffer_count] {
                framebuffer.draw_buffers[buffer_count] = buffer;

                needs_update = true;
            }

            buffer_count += 1;
        }

        for i in buffer_count..self.max_draw_buffers {
            if DrawBuffer::None != framebuffer.draw_buffers[i] {
                framebuffer.draw_buffers[i] = DrawBuffer::None;

                needs_update = true;
            }
        }

        if needs_update {
            let mut buffer_ids = [0; 16];

            for (i, buffer) in framebuffer.draw_buffers[0..self.max_draw_buffers]
                .iter()
                .enumerate()
            {
                buffer_ids[i] = buffer.id();
            }

            let array = unsafe { Uint32Array::view(&buffer_ids[0..self.max_draw_buffers]) };

            self.gl.draw_buffers(array.as_ref());
        }
    }
}

pub(crate) struct FramebufferCache<'a> {
    state: &'a mut DynamicState,
}

impl<'a> FramebufferCache<'a> {
    pub(crate) fn bind_or_create<'b: 'a, A>(
        &'b mut self,
        attachment_set: &A,
        gl: &'b Gl,
    ) -> CachedFramebuffer<'b>
    where
        A: AttachmentSet,
    {
        let mut hasher = FnvHasher::default();

        attachment_set.hash(&mut hasher);

        let key = hasher.finish();
        let max_draw_buffers = self.state.max_draw_buffers;
        let DynamicState {
            framebuffer_cache,
            bound_draw_framebuffer,
            ..
        } = &mut self.state;
        let target = Gl::DRAW_FRAMEBUFFER;

        let (framebuffer, _) = framebuffer_cache
            .entry(key)
            .and_modify(|(framebuffer, _)| {
                let current =
                    unsafe { bound_draw_framebuffer.map(|abi| JsValue::ref_from_abi(abi)) };

                if !identical(
                    Some(&framebuffer.fbo),
                    current.as_ref().map(|fb| fb.deref()),
                ) {
                    gl.bind_framebuffer(target, Some(&framebuffer.fbo));

                    *bound_draw_framebuffer = Some((&framebuffer.fbo).into_abi());
                }
            })
            .or_insert_with(|| {
                let fbo = gl.create_framebuffer().unwrap();

                gl.bind_framebuffer(target, Some(&fbo));

                *bound_draw_framebuffer = Some((&fbo).into_abi());

                let mut attachment_ids = [None; 17];

                for (i, attachment) in attachment_set.color_attachments().iter().enumerate() {
                    if let Some(attachment) = attachment {
                        attachment.attach(gl, target, Gl::COLOR_ATTACHMENT0 + i as u32);

                        attachment_ids[i] = Some(attachment.id());
                    }
                }

                if let Some((slot, image)) = match attachment_set.depth_stencil_attachment() {
                    DepthStencilAttachmentDescriptor::Depth(image) => {
                        Some((Gl::DEPTH_ATTACHMENT, image))
                    }
                    DepthStencilAttachmentDescriptor::Stencil(image) => {
                        Some((Gl::STENCIL_ATTACHMENT, image))
                    }
                    DepthStencilAttachmentDescriptor::DepthStencil(image) => {
                        Some((Gl::DEPTH_STENCIL_ATTACHMENT, image))
                    }
                    DepthStencilAttachmentDescriptor::None => None,
                } {
                    image.attach(gl, target, slot);

                    attachment_ids[16] = Some(image.id());
                }

                let framebuffer = Framebuffer {
                    fbo,
                    draw_buffers: [
                        DrawBuffer::Color0,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                        DrawBuffer::None,
                    ],
                };

                (framebuffer, attachment_ids)
            });

        CachedFramebuffer {
            framebuffer,
            max_draw_buffers,
            gl,
        }
    }

    pub(crate) fn remove_attachment_dependents(&mut self, attachment_id: JsId, gl: &Gl) {
        self.state
            .framebuffer_cache
            .retain(|_, (framebuffer, attachment_ids)| {
                let is_dependent = attachment_ids.iter().any(|id| id == &Some(attachment_id));

                if is_dependent {
                    gl.delete_framebuffer(Some(&framebuffer.fbo));
                }

                !is_dependent
            })
    }
}

pub(crate) trait AttachmentSet: Hash {
    fn color_attachments(&self) -> &[Option<AttachmentData>];

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor;
}

#[derive(Clone, PartialEq, Hash)]
pub(crate) enum DepthStencilAttachmentDescriptor {
    Depth(AttachmentData),
    Stencil(AttachmentData),
    DepthStencil(AttachmentData),
    None,
}

pub(crate) struct VertexArrayCache<'a> {
    state: &'a mut DynamicState,
}

impl<'a> VertexArrayCache<'a> {
    pub(crate) fn bind_or_create(
        &mut self,
        layout: &VertexInputLayoutDescriptor,
        vertex_buffers: &[BufferDescriptor],
        gl: &Gl,
    ) -> &WebGlVertexArrayObject {
        let mut hasher = FnvHasher::default();

        layout.hash(&mut hasher);
        vertex_buffers.hash(&mut hasher);

        let key = hasher.finish();
        let DynamicState {
            vertex_array_cache,
            bound_vertex_array,
            bound_array_buffer,
            ..
        } = &mut self.state;

        let (vao, _) = vertex_array_cache
            .entry(key)
            .and_modify(|(vertex_array, _)| {
                let current = unsafe { bound_vertex_array.map(|abi| JsValue::ref_from_abi(abi)) };

                if !identical(Some(vertex_array), current.as_ref().map(|v| v.deref())) {
                    gl.bind_vertex_array(Some(vertex_array));

                    *bound_vertex_array = Some((&*vertex_array).into_abi());
                }
            })
            .or_insert_with(|| {
                let vao = gl.create_vertex_array().unwrap();

                gl.bind_vertex_array(Some(&vao));

                *bound_vertex_array = Some((&vao).into_abi());

                let mut buffer_ids = [None; 17];

                for (i, (bind_slot, buffer_descriptor)) in
                    layout.buffer_slots().zip(vertex_buffers.iter()).enumerate()
                {
                    let buffer_id = buffer_descriptor.buffer_data.id();

                    unsafe {
                        buffer_id
                            .unwrap()
                            .with_value_unchecked(|buffer: &WebGlBuffer| {
                                *bound_array_buffer = Some(buffer.into_abi());

                                gl.bind_buffer(Gl::ARRAY_BUFFER, Some(buffer));
                            });
                    }

                    for attribute_descriptor in bind_slot.attributes() {
                        attribute_descriptor.apply(
                            gl,
                            bind_slot.stride_in_bytes() as i32,
                            buffer_descriptor.offset_in_bytes as i32,
                            bind_slot.input_rate(),
                        );
                    }

                    buffer_ids[i] = buffer_id;
                }

                (vao, buffer_ids)
            });

        vao
    }

    pub(crate) fn bind_or_create_indexed(
        &mut self,
        layout: &VertexInputLayoutDescriptor,
        vertex_buffers: &[BufferDescriptor],
        index_buffer: &IndexDataDescriptor,
        gl: &Gl,
    ) -> &WebGlVertexArrayObject {
        let mut hasher = FnvHasher::default();

        layout.hash(&mut hasher);
        vertex_buffers.hash(&mut hasher);
        index_buffer.hash(&mut hasher);

        let key = hasher.finish();
        let DynamicState {
            vertex_array_cache,
            bound_vertex_array,
            bound_array_buffer,
            ..
        } = &mut self.state;

        let (vao, _) = vertex_array_cache
            .entry(key)
            .and_modify(|(vertex_array, _)| {
                let current = unsafe { bound_vertex_array.map(|abi| JsValue::ref_from_abi(abi)) };

                if !identical(Some(vertex_array), current.as_ref().map(|v| v.deref())) {
                    gl.bind_vertex_array(Some(vertex_array));

                    *bound_vertex_array = Some((&*vertex_array).into_abi());
                }
            })
            .or_insert_with(|| {
                let vao = gl.create_vertex_array().unwrap();

                gl.bind_vertex_array(Some(&vao));

                *bound_vertex_array = Some((&vao).into_abi());

                let mut buffer_ids = [None; 17];

                for (i, (bind_slot, buffer_descriptor)) in
                    layout.buffer_slots().zip(vertex_buffers.iter()).enumerate()
                {
                    let buffer_id = buffer_descriptor.buffer_data.id();

                    unsafe {
                        buffer_id
                            .unwrap()
                            .with_value_unchecked(|buffer: &WebGlBuffer| {
                                *bound_array_buffer = Some(buffer.into_abi());

                                gl.bind_buffer(Gl::ARRAY_BUFFER, Some(buffer));
                            });
                    }

                    for attribute_descriptor in bind_slot.attributes() {
                        attribute_descriptor.apply(
                            gl,
                            bind_slot.stride_in_bytes() as i32,
                            buffer_descriptor.offset_in_bytes as i32,
                            bind_slot.input_rate(),
                        );
                    }

                    buffer_ids[i] = buffer_id;
                }

                let index_buffer_id = index_buffer.buffer_data.id();

                unsafe {
                    index_buffer_id
                        .unwrap()
                        .with_value_unchecked(|buffer: &WebGlBuffer| {
                            gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(buffer));
                        });
                }

                buffer_ids[16] = index_buffer_id;

                (vao, buffer_ids)
            });

        vao
    }

    pub(crate) fn remove_buffer_dependents(&mut self, buffer_id: JsId, gl: &Gl) {
        self.state
            .vertex_array_cache
            .retain(|_, (vao, buffer_ids)| {
                let is_dependent = buffer_ids.iter().any(|id| id == &Some(buffer_id));

                if is_dependent {
                    gl.delete_vertex_array(Some(vao));
                }

                !is_dependent
            })
    }
}

pub(crate) struct ProgramCache<'a> {
    state: &'a mut DynamicState,
}

impl<'a> ProgramCache<'a> {
    pub(crate) fn get_or_create(
        &mut self,
        key: ProgramKey,
        transform_feedback_layout: &Option<TransformFeedbackLayoutDescriptor>,
        gl: &Gl,
    ) -> Result<&Program, CreateProgramError> {
        let program = match self.state.program_cache.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let program_object = gl.create_program().unwrap();

                unsafe {
                    key.vertex_shader_id.with_value_unchecked(|shader_object| {
                        gl.attach_shader(&program_object, &shader_object);
                    });

                    key.fragment_shader_id
                        .with_value_unchecked(|shader_object| {
                            gl.attach_shader(&program_object, &shader_object);
                        });
                }

                if let Some(layout) = transform_feedback_layout {
                    let varyings = JsValue::from_serde(&TransformFeedbackVaryings(layout)).unwrap();

                    gl.transform_feedback_varyings(
                        &program_object,
                        &varyings,
                        Gl::INTERLEAVED_ATTRIBS,
                    );
                }

                gl.link_program(&program_object);

                if !gl
                    .get_program_parameter(&program_object, Gl::LINK_STATUS)
                    .as_bool()
                    .unwrap()
                {
                    let info = gl
                        .get_program_info_log(&program_object)
                        .unwrap_or("".to_string());

                    return Err(CreateProgramError::ShaderLinkingError(info));
                }

                let active_attribute_count = gl
                    .get_program_parameter(&program_object, Gl::ACTIVE_ATTRIBUTES)
                    .as_f64()
                    .unwrap() as u32;
                let mut attribute_slot_descriptors =
                    Vec::with_capacity(active_attribute_count as usize);

                for i in 0..active_attribute_count {
                    if let Some(info) = gl.get_active_attrib(&program_object, i) {
                        let name = info.name();
                        let location = gl.get_attrib_location(&program_object, &name);

                        if location != -1 {
                            let attribute_type = VertexAttributeType::from_type_id(info.type_());

                            attribute_slot_descriptors.push(VertexAttributeSlotDescriptor {
                                attribute_type,
                                location: location as u32,
                            });
                        }
                    }
                }

                let active_uniform_count = gl
                    .get_program_parameter(&program_object, Gl::ACTIVE_UNIFORMS)
                    .as_f64()
                    .unwrap() as u32;
                let active_block_count = gl
                    .get_program_parameter(&program_object, Gl::ACTIVE_UNIFORM_BLOCKS)
                    .as_f64()
                    .unwrap() as u32;
                let resource_slot_count = (active_uniform_count + active_block_count) as usize;
                let mut resource_slot_descriptors = Vec::with_capacity(resource_slot_count);

                for i in 0..active_block_count {
                    let name = gl
                        .get_active_uniform_block_name(&program_object, i)
                        .unwrap();
                    let identifier = ResourceSlotIdentifier::Dynamic(name);
                    let slot = UniformBlockSlot::new(gl, &program_object, i as usize);

                    resource_slot_descriptors
                        .push(ShaderResourceSlotDescriptor::new(identifier, slot.into()));
                }

                for i in 0..active_uniform_count {
                    let info = gl.get_active_uniform(&program_object, i).unwrap();
                    let name = info.name();

                    // As well as retrieving the location, this also filters out uniforms are part of
                    // uniform blocks, as these won't have locations.
                    if let Some(location) = gl.get_uniform_location(&program_object, &name) {
                        let identifier = ResourceSlotIdentifier::Dynamic(name);

                        if info.size() == 1 {
                            let slot = match info.type_() {
                                Gl::FLOAT => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "FLOAT",
                                    ));
                                }
                                Gl::FLOAT_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_VEC2",
                                    ));
                                }
                                Gl::FLOAT_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_VEC3",
                                    ));
                                }
                                Gl::FLOAT_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_VEC4",
                                    ));
                                }
                                Gl::INT => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "INT",
                                    ));
                                }
                                Gl::INT_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "INT_VEC2",
                                    ));
                                }
                                Gl::INT_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "INT_VEC3",
                                    ));
                                }
                                Gl::INT_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "INT_VEC4",
                                    ));
                                }
                                Gl::UNSIGNED_INT => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT",
                                    ));
                                }
                                Gl::UNSIGNED_INT_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_VEC2",
                                    ));
                                }
                                Gl::UNSIGNED_INT_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_VEC3",
                                    ));
                                }
                                Gl::UNSIGNED_INT_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_VEC4",
                                    ));
                                }
                                Gl::BOOL => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "BOOL",
                                    ));
                                }
                                Gl::BOOL_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "BOOL_VEC2",
                                    ));
                                }
                                Gl::BOOL_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "BOOL_VEC3",
                                    ));
                                }
                                Gl::BOOL_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "BOOL_VEC4",
                                    ));
                                }
                                Gl::FLOAT_MAT2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT2",
                                    ));
                                }
                                Gl::FLOAT_MAT3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT3",
                                    ));
                                }
                                Gl::FLOAT_MAT4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT4",
                                    ));
                                }
                                Gl::FLOAT_MAT2X3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT2x3",
                                    ));
                                }
                                Gl::FLOAT_MAT2X4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT2x4",
                                    ));
                                }
                                Gl::FLOAT_MAT3X2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT3x2",
                                    ));
                                }
                                Gl::FLOAT_MAT3X4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT3x4",
                                    ));
                                }
                                Gl::FLOAT_MAT4X2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT4x2",
                                    ));
                                }
                                Gl::FLOAT_MAT4X3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT4x3",
                                    ));
                                }
                                Gl::SAMPLER_2D => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::FloatSampler2D,
                                ),
                                Gl::SAMPLER_3D => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::FloatSampler3D,
                                ),
                                Gl::SAMPLER_CUBE => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::FloatSamplerCube,
                                ),
                                Gl::SAMPLER_2D_SHADOW => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::Sampler2DShadow,
                                ),
                                Gl::SAMPLER_2D_ARRAY => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::FloatSampler2DArray,
                                ),
                                Gl::SAMPLER_2D_ARRAY_SHADOW => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::Sampler2DArrayShadow,
                                ),
                                Gl::SAMPLER_CUBE_SHADOW => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::SamplerCubeShadow,
                                ),
                                Gl::INT_SAMPLER_2D => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::IntegerSampler2D,
                                ),
                                Gl::INT_SAMPLER_3D => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::IntegerSampler3D,
                                ),
                                Gl::INT_SAMPLER_CUBE => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::IntegerSamplerCube,
                                ),
                                Gl::INT_SAMPLER_2D_ARRAY => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::IntegerSampler2DArray,
                                ),
                                Gl::UNSIGNED_INT_SAMPLER_2D => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::UnsignedIntegerSampler2D,
                                ),
                                Gl::UNSIGNED_INT_SAMPLER_3D => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::UnsignedIntegerSampler3D,
                                ),
                                Gl::UNSIGNED_INT_SAMPLER_CUBE => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::UnsignedIntegerSamplerCube,
                                ),
                                Gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => TextureSamplerSlot::new(
                                    location,
                                    SampledTextureType::UnsignedIntegerSampler2DArray,
                                ),
                                _ => unreachable!(),
                            };

                            resource_slot_descriptors
                                .push(ShaderResourceSlotDescriptor::new(identifier, slot.into()));
                        } else {
                            match info.type_() {
                                Gl::FLOAT => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "FLOAT[]",
                                    ));
                                }
                                Gl::FLOAT_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_VEC2[]",
                                    ));
                                }
                                Gl::FLOAT_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_VEC3[]",
                                    ));
                                }
                                Gl::FLOAT_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_VEC4[]",
                                    ));
                                }
                                Gl::INT => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "INT[]",
                                    ));
                                }
                                Gl::INT_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_VEC2[]",
                                    ));
                                }
                                Gl::INT_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_VEC3[]",
                                    ));
                                }
                                Gl::INT_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_VEC4[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_VEC2[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_VEC3[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_VEC4[]",
                                    ));
                                }
                                Gl::BOOL => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier, "BOOL[]",
                                    ));
                                }
                                Gl::BOOL_VEC2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "BOOL_VEC2[]",
                                    ));
                                }
                                Gl::BOOL_VEC3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "BOOL_VEC3[]",
                                    ));
                                }
                                Gl::BOOL_VEC4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "BOOL_VEC4[]",
                                    ));
                                }
                                Gl::FLOAT_MAT2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT2[]",
                                    ));
                                }
                                Gl::FLOAT_MAT3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT3[]",
                                    ));
                                }
                                Gl::FLOAT_MAT4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT4[]",
                                    ));
                                }
                                Gl::FLOAT_MAT2X3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT2x3[]",
                                    ));
                                }
                                Gl::FLOAT_MAT2X4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT2x4[]",
                                    ));
                                }
                                Gl::FLOAT_MAT3X2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT3x2[]",
                                    ));
                                }
                                Gl::FLOAT_MAT3X4 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT3x4[]",
                                    ));
                                }
                                Gl::FLOAT_MAT4X2 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT4x2[]",
                                    ));
                                }
                                Gl::FLOAT_MAT4X3 => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "FLOAT_MAT4x3[]",
                                    ));
                                }
                                Gl::SAMPLER_2D => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_2D[]",
                                    ));
                                }
                                Gl::SAMPLER_3D => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_3D[]",
                                    ));
                                }
                                Gl::SAMPLER_CUBE => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_CUBE[]",
                                    ));
                                }
                                Gl::SAMPLER_2D_SHADOW => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_2D_SHADOW[]",
                                    ));
                                }
                                Gl::SAMPLER_2D_ARRAY => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_2D_ARRAY[]",
                                    ));
                                }
                                Gl::SAMPLER_2D_ARRAY_SHADOW => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_2D_ARRAY_SHADOW[]",
                                    ));
                                }
                                Gl::SAMPLER_CUBE_SHADOW => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "SAMPLER_CUBE_SHADOW[]",
                                    ));
                                }
                                Gl::INT_SAMPLER_2D => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_SAMPLER_2D[]",
                                    ));
                                }
                                Gl::INT_SAMPLER_3D => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_SAMPLER_3D[]",
                                    ));
                                }
                                Gl::INT_SAMPLER_CUBE => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_SAMPLER_CUBE[]",
                                    ));
                                }
                                Gl::INT_SAMPLER_2D_ARRAY => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "INT_SAMPLER_2D_ARRAY[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_SAMPLER_2D => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_SAMPLER_2D[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_SAMPLER_3D => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_SAMPLER_3D[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_SAMPLER_CUBE => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_SAMPLER_CUBE[]",
                                    ));
                                }
                                Gl::UNSIGNED_INT_SAMPLER_2D_ARRAY => {
                                    return Err(CreateProgramError::UnsupportedUniformType(
                                        identifier,
                                        "UNSIGNED_INT_SAMPLER_2D_ARRAY[]",
                                    ));
                                }
                                _ => unreachable!(),
                            };
                        }
                    }
                }

                entry.insert(Program {
                    gl_object: program_object,
                    attribute_slot_descriptors,
                    resource_slot_descriptors,
                })
            }
        };

        Ok(program)
    }

    pub(crate) fn remove_vertex_shader_dependent(&mut self, shader_id: JsId) {
        self.state
            .program_cache
            .retain(|key, _| key.vertex_shader_id != shader_id);
    }

    pub(crate) fn remove_fragment_shader_dependent(&mut self, shader_id: JsId) {
        self.state
            .program_cache
            .retain(|key, _| key.fragment_shader_id != shader_id);
    }
}

pub enum CreateProgramError {
    ShaderLinkingError(String),
    UnsupportedUniformType(ResourceSlotIdentifier, &'static str),
}

pub(crate) struct Program {
    gl_object: WebGlProgram,
    attribute_slot_descriptors: Vec<VertexAttributeSlotDescriptor>,
    resource_slot_descriptors: Vec<ShaderResourceSlotDescriptor>,
}

impl !Send for Program {}

impl Program {
    pub fn gl_object(&self) -> &WebGlProgram {
        &self.gl_object
    }

    pub fn attribute_slot_descriptors(&self) -> &[VertexAttributeSlotDescriptor] {
        &self.attribute_slot_descriptors
    }

    pub fn resource_slot_descriptors(&self) -> &[ShaderResourceSlotDescriptor] {
        &self.resource_slot_descriptors
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct ProgramKey {
    pub(crate) vertex_shader_id: JsId,
    pub(crate) fragment_shader_id: JsId,
    pub(crate) transform_feedback_layout_key: Option<u64>,
    pub(crate) resource_bindings_layout: u64,
}
