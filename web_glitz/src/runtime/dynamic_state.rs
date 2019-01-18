use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

use js_sys::Uint32Array;
use wasm_bindgen::JsValue;
use web_sys::{
    WebGl2RenderingContext as Gl, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlRenderbuffer,
    WebGlSampler, WebGlSync, WebGlTexture, WebGlVertexArrayObject,
};

use crate::render_pass::AttachableImageDescriptor;
use crate::runtime::index_lru::IndexLRU;
use crate::util::identical;
use fnv::{FnvHashMap, FnvHasher};
use util::JsId;

pub struct DynamicState {
    pub(crate) framebuffer_cache: FnvHashMap<u64, (Framebuffer, [Option<JsId>; 17])>,
    max_draw_buffers: usize,
    active_program: Option<WebGlProgram>,
    bound_array_buffer: Option<WebGlBuffer>,
    bound_element_array_buffer: Option<WebGlBuffer>,
    bound_copy_read_buffer: Option<WebGlBuffer>,
    bound_copy_write_buffer: Option<WebGlBuffer>,
    bound_pixel_pack_buffer: Option<WebGlBuffer>,
    bound_pixel_unpack_buffer: Option<WebGlBuffer>,
    bound_transform_feedback_buffers: Vec<BufferRange<WebGlBuffer>>,
    active_uniform_buffer_index: u32,
    bound_uniform_buffers: Vec<BufferRange<WebGlBuffer>>,
    uniform_buffer_index_lru: IndexLRU,
    bound_draw_framebuffer: Option<WebGlFramebuffer>,
    bound_read_framebuffer: Option<WebGlFramebuffer>,
    bound_renderbuffer: Option<WebGlRenderbuffer>,
    bound_texture_2d: Option<WebGlTexture>,
    bound_texture_cube_map: Option<WebGlTexture>,
    bound_texture_3d: Option<WebGlTexture>,
    bound_texture_2d_array: Option<WebGlTexture>,
    bound_samplers: Vec<Option<WebGlSampler>>,
    texture_units_lru: IndexLRU,
    texture_units_textures: Vec<Option<WebGlTexture>>,
    bound_vertex_array: Option<WebGlVertexArrayObject>,
    active_texture: u32,
    clear_color: [f32; 4],
    clear_depth: f32,
    clear_stencil: i32,
    depth_test_enabled: bool,
    stencil_test_enabled: bool,
    scissor_test_enabled: bool,
    blend_enabled: bool,
    cull_face_enabled: bool,
    dither_enabled: bool,
    polygon_offset_fill_enabled: bool,
    sample_alpha_to_coverage_enabled: bool,
    sample_coverage_enabled: bool,
    rasterizer_discard_enabled: bool,
    //    read_buffer: ReadBuffer,
    //    blend_color: [f32;4],
    //    blend_equation_rgb: BlendEquation,
    //    blend_equation_alpha: BlendEquation,
    //    blend_func_rgb: BlendFunc,
    //    blend_func_alpha: BlendFunc,
    //    color_mask: [bool;4],
    //    cull_face: CullFace,
    //    front_face: FrontFace,
    //    line_width: f32,
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
    //    viewport: Region,
    //    stencil_func_rgb: StencilFunc,
    //    stencil_func_alpha: StencilFunc,
    //    stencil_mask_rgb: u32,
    //    stencil_mask_alpha: u32,
    //    stencil_op_rgb: StencilOp,
    //    stencil_op_alpha: StencilOp,
}

impl DynamicState {
    pub(crate) fn framebuffer_cache_mut(&mut self) -> FramebufferCache {
        FramebufferCache { state: self }
    }

    pub fn max_draw_buffers(&self) -> usize {
        self.max_draw_buffers
    }

    pub fn active_program(&self) -> Option<&WebGlProgram> {
        self.active_program.as_ref()
    }

    pub fn set_active_program<'a>(
        &mut self,
        program: Option<&'a WebGlProgram>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(program, self.active_program.as_ref()) {
            self.active_program = program.map(|p| p.clone());

            Some(move |context: &Gl| {
                context.use_program(program);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_array_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_array_buffer.as_ref()
    }

    pub fn set_bound_array_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_array_buffer.as_ref()) {
            self.bound_array_buffer = buffer.map(|b| b.clone());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::ARRAY_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_element_array_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_element_array_buffer.as_ref()
    }

    pub fn set_bound_element_array_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_element_array_buffer.as_ref()) {
            self.bound_element_array_buffer = buffer.map(|b| b.clone());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_copy_read_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_copy_read_buffer.as_ref()
    }

    pub fn set_bound_copy_read_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_copy_read_buffer.as_ref()) {
            self.bound_copy_read_buffer = buffer.map(|b| b.clone());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::COPY_READ_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_copy_write_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_copy_write_buffer.as_ref()
    }

    pub fn set_bound_copy_write_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_copy_write_buffer.as_ref()) {
            self.bound_copy_write_buffer = buffer.map(|b| b.clone());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::COPY_WRITE_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_pixel_pack_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_pixel_pack_buffer.as_ref()
    }

    pub fn set_bound_pixel_pack_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_pixel_pack_buffer.as_ref()) {
            self.bound_pixel_pack_buffer = buffer.map(|b| b.clone());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::PIXEL_PACK_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_pixel_unpack_buffer(&self) -> Option<&WebGlBuffer> {
        self.bound_pixel_unpack_buffer.as_ref()
    }

    pub fn set_bound_pixel_unpack_buffer<'a>(
        &mut self,
        buffer: Option<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(buffer, self.bound_pixel_unpack_buffer.as_ref()) {
            self.bound_pixel_unpack_buffer = buffer.map(|b| b.clone());

            Some(move |context: &Gl| {
                context.bind_buffer(Gl::PIXEL_UNPACK_BUFFER, buffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_transform_feedback_buffer_range(&self, index: u32) -> BufferRange<&WebGlBuffer> {
        self.bound_transform_feedback_buffers[index as usize].as_ref()
    }

    pub fn set_bound_transform_feedback_buffer_range<'a>(
        &mut self,
        index: u32,
        buffer_range: BufferRange<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if buffer_range != self.bound_transform_feedback_buffers[index as usize].as_ref() {
            self.bound_transform_feedback_buffers[index as usize] = buffer_range.to_owned_buffer();

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

    pub fn bound_uniform_buffer_range(&self, index: u32) -> BufferRange<&WebGlBuffer> {
        self.bound_uniform_buffers[index as usize].as_ref()
    }

    pub fn set_bound_uniform_buffer_range<'a>(
        &mut self,
        buffer_range: BufferRange<&'a WebGlBuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        let index = self.active_uniform_buffer_index;

        if buffer_range != self.bound_uniform_buffers[index as usize].as_ref() {
            self.bound_uniform_buffers[index as usize] = buffer_range.to_owned_buffer();

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

    pub fn bound_draw_framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.bound_draw_framebuffer.as_ref()
    }

    pub fn set_bound_draw_framebuffer<'a>(
        &mut self,
        framebuffer: Option<&'a WebGlFramebuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(framebuffer, self.bound_draw_framebuffer.as_ref()) {
            self.bound_draw_framebuffer = framebuffer.map(|f| f.clone());

            Some(move |context: &Gl| {
                context.bind_framebuffer(Gl::DRAW_FRAMEBUFFER, framebuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_read_framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.bound_read_framebuffer.as_ref()
    }

    pub fn set_bound_read_framebuffer<'a>(
        &mut self,
        framebuffer: Option<&'a WebGlFramebuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(framebuffer, self.bound_read_framebuffer.as_ref()) {
            self.bound_read_framebuffer = framebuffer.map(|f| f.clone());

            Some(move |context: &Gl| {
                context.bind_framebuffer(Gl::READ_FRAMEBUFFER, framebuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_renderbuffer(&self) -> Option<&WebGlRenderbuffer> {
        self.bound_renderbuffer.as_ref()
    }

    pub fn set_bound_renderbuffer<'a>(
        &mut self,
        renderbuffer: Option<&'a WebGlRenderbuffer>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(renderbuffer, self.bound_renderbuffer.as_ref()) {
            self.bound_renderbuffer = renderbuffer.map(|r| r.clone());

            Some(move |context: &Gl| {
                context.bind_renderbuffer(Gl::RENDERBUFFER, renderbuffer);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_2d(&self) -> Option<&WebGlTexture> {
        self.bound_texture_2d.as_ref()
    }

    pub fn set_bound_texture_2d<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        if !identical(texture, self.bound_texture_2d.as_ref())
            || !identical(texture, active_unit_texture.as_ref())
        {
            self.bound_texture_2d = texture.map(|t| t.clone());
            *active_unit_texture = texture.map(|t| t.clone());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_2D, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_2d_array(&self) -> Option<&WebGlTexture> {
        self.bound_texture_2d_array.as_ref()
    }

    pub fn set_bound_texture_2d_array<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        if !identical(texture, self.bound_texture_2d_array.as_ref())
            || !identical(texture, active_unit_texture.as_ref())
        {
            self.bound_texture_2d_array = texture.map(|t| t.clone());
            *active_unit_texture = texture.map(|t| t.clone());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_2D_ARRAY, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_3d(&self) -> Option<&WebGlTexture> {
        self.bound_texture_3d.as_ref()
    }

    pub fn set_bound_texture_3d<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        if !identical(texture, self.bound_texture_3d.as_ref())
            || !identical(texture, active_unit_texture.as_ref())
        {
            self.bound_texture_3d = texture.map(|t| t.clone());
            *active_unit_texture = texture.map(|t| t.clone());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_3D, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_texture_cube_map(&self) -> Option<&WebGlTexture> {
        self.bound_texture_cube_map.as_ref()
    }

    pub fn set_bound_texture_cube_map<'a>(
        &mut self,
        texture: Option<&'a WebGlTexture>,
    ) -> impl ContextUpdate<'a, ()> {
        let active_unit_texture = &mut self.texture_units_textures[self.active_texture as usize];

        if !identical(texture, self.bound_texture_cube_map.as_ref())
            || !identical(texture, active_unit_texture.as_ref())
        {
            self.bound_texture_cube_map = texture.map(|t| t.clone());
            *active_unit_texture = texture.map(|t| t.clone());

            Some(move |context: &Gl| {
                context.bind_texture(Gl::TEXTURE_CUBE_MAP, texture);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn texture_units_textures(&self) -> &[Option<WebGlTexture>] {
        &self.texture_units_textures
    }

    pub fn texture_units_textures_mut(&mut self) -> &mut [Option<WebGlTexture>] {
        &mut self.texture_units_textures
    }

    pub fn bound_sampler(&self, texture_unit: u32) -> Option<&WebGlSampler> {
        self.bound_samplers[texture_unit as usize].as_ref()
    }

    pub fn set_bound_sampler<'a>(
        &mut self,
        texture_unit: u32,
        sampler: Option<&'a WebGlSampler>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(sampler, self.bound_samplers[texture_unit as usize].as_ref()) {
            self.bound_samplers[texture_unit as usize] = sampler.map(|v| v.clone());

            Some(move |context: &Gl| {
                context.bind_sampler(texture_unit, sampler);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn bound_vertex_array(&self) -> Option<&WebGlVertexArrayObject> {
        self.bound_vertex_array.as_ref()
    }

    pub fn set_bound_vertex_array<'a>(
        &mut self,
        vertex_array: Option<&'a WebGlVertexArrayObject>,
    ) -> impl ContextUpdate<'a, ()> {
        if !identical(vertex_array, self.bound_vertex_array.as_ref()) {
            self.bound_vertex_array = vertex_array.map(|v| v.clone());

            Some(move |context: &Gl| {
                context.bind_vertex_array(vertex_array);

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
                context.enable(Gl::DEPTH_TEST);

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
                context.enable(Gl::STENCIL_TEST);

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
                context.enable(Gl::SCISSOR_TEST);

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
                context.enable(Gl::BLEND);

                Ok(())
            })
        } else {
            None
        }
    }

    pub fn cull_face_enabled(&self) -> bool {
        self.cull_face_enabled
    }

    pub fn set_cull_face_enabled(
        &mut self,
        cull_face_enabled: bool,
    ) -> impl ContextUpdate<'static, ()> {
        if cull_face_enabled != self.cull_face_enabled {
            self.cull_face_enabled = cull_face_enabled;

            Some(move |context: &Gl| {
                context.enable(Gl::CULL_FACE);

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
                context.enable(Gl::DITHER);

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
                context.enable(Gl::POLYGON_OFFSET_FILL);

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
                context.enable(Gl::SAMPLE_ALPHA_TO_COVERAGE);

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
                context.enable(Gl::SAMPLE_COVERAGE);

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
                context.enable(Gl::RASTERIZER_DISCARD);

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
            cull_face_enabled: false,
            dither_enabled: true,
            polygon_offset_fill_enabled: false,
            sample_alpha_to_coverage_enabled: false,
            sample_coverage_enabled: false,
            rasterizer_discard_enabled: false,
            scissor: (0, 0, 0, 0),
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

#[derive(Clone)]
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
}

impl<'a> BufferRange<&'a WebGlBuffer> {
    pub fn to_owned_buffer(&self) -> BufferRange<WebGlBuffer> {
        match *self {
            BufferRange::None => BufferRange::None,
            BufferRange::Full(buffer) => BufferRange::Full(buffer.clone()),
            BufferRange::OffsetSize(buffer, offset, size) => {
                BufferRange::OffsetSize(buffer.clone(), offset, size)
            }
        }
    }
}

impl<T> PartialEq for BufferRange<T>
where
    T: Borrow<WebGlBuffer>,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BufferRange::None, BufferRange::None) => true,
            (BufferRange::Full(a), BufferRange::Full(b)) => {
                AsRef::<JsValue>::as_ref(a.borrow()) == AsRef::<JsValue>::as_ref(b.borrow())
            }
            (
                BufferRange::OffsetSize(a, offset_a, size_a),
                BufferRange::OffsetSize(b, offset_b, size_b),
            ) => {
                offset_a == offset_b
                    && size_a == size_b
                    && AsRef::<JsValue>::as_ref(a.borrow()) == AsRef::<JsValue>::as_ref(b.borrow())
            }
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
                if !identical(Some(&framebuffer.fbo), bound_draw_framebuffer.as_ref()) {
                    gl.bind_framebuffer(target, Some(&framebuffer.fbo));

                    *bound_draw_framebuffer = Some(framebuffer.fbo.clone());
                }
            })
            .or_insert_with(|| {
                let fbo = gl.create_framebuffer().unwrap();

                gl.bind_framebuffer(target, Some(&fbo));

                *bound_draw_framebuffer = Some(fbo.clone());

                let mut attachment_ids = [None; 17];

                for (i, attachment) in attachment_set.color_attachments().iter().enumerate() {
                    attachment.attach(gl, target, Gl::COLOR_ATTACHMENT0 + i as u32);

                    attachment_ids[i] = Some(attachment.id());
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

                is_dependent
            })
    }
}

pub(crate) trait AttachmentSet: Hash {
    fn color_attachments(&self) -> &[AttachableImageDescriptor];

    fn depth_stencil_attachment(&self) -> &DepthStencilAttachmentDescriptor;
}

#[derive(PartialEq, Hash)]
pub(crate) enum DepthStencilAttachmentDescriptor {
    Depth(AttachableImageDescriptor),
    Stencil(AttachableImageDescriptor),
    DepthStencil(AttachableImageDescriptor),
    None,
}
