// This example shows how to bind a texture resource to a pipeline.
//
// This example builds on `/examples/3_textured_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn, const_ptr_offset_from, const_transmute, ptr_offset_from)]

use futures::FutureExt;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlCanvasElement};

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::image::format::RGBA8;
use web_glitz::image::texture_2d::{FloatSampledTexture2D, Texture2DDescriptor};
use web_glitz::image::MipmapLevels;
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::rendering::{LoadOp, RenderTargetDescriptor, StoreOp};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::sampler::{Linear, SamplerDescriptor, Wrap};
use web_glitz::task::{sequence_all, sequence_right};

// This example will use 2 render passes:
// - One to render the texture on a custom render target; the example refers to this pass as the
//   "secondary" render pass.
// - One to render the final result to the default render target; the example refers to this pass as
//   the "primary render pass.
//
// The passes will use different pipelines and the example will also refer to these pipeline's and
// their associated resources as "primary" and "secondary" accordingly.

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct PrimaryVertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float2_f32")]
    texture_coordinates: [f32; 2],
}

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct SecondaryVertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

#[derive(web_glitz::derive::Resources)]
struct PrimaryResources<'a> {
    #[resource(binding = 0, name = "diffuse_texture")]
    texture: FloatSampledTexture2D<'a>,
}

#[wasm_bindgen(start)]
pub fn start() {
    let document = window().unwrap().document().unwrap();

    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    let (context, mut default_render_target) =
        unsafe { single_threaded::init(&canvas, &ContextOptions::default()).unwrap() };

    let primary_vertex_shader = context
        .try_create_vertex_shader(include_str!("primary_vertex.glsl"))
        .unwrap();

    let primary_fragment_shader = context
        .try_create_fragment_shader(include_str!("primary_fragment.glsl"))
        .unwrap();

    let primary_pipeline = context
        .try_create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&primary_vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&primary_fragment_shader)
                .typed_vertex_attribute_layout::<PrimaryVertex>()
                .typed_resource_bindings_layout::<((), PrimaryResources)>()
                .finish(),
        )
        .unwrap();

    let secondary_vertex_shader = context
        .try_create_vertex_shader(include_str!("secondary_vertex.glsl"))
        .unwrap();

    let secondary_fragment_shader = context
        .try_create_fragment_shader(include_str!("secondary_fragment.glsl"))
        .unwrap();

    let secondary_pipeline = context
        .try_create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&secondary_vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&secondary_fragment_shader)
                .typed_vertex_attribute_layout::<SecondaryVertex>()
                .typed_resource_bindings_layout::<((), ())>()
                .finish(),
        )
        .unwrap();

    // We have to mark the texture as `mut` here, as using a texture as a render target attachment
    // (see below) requires a mut reference to a texture image.
    let mut texture = context
        .try_create_texture_2d(&Texture2DDescriptor {
            format: RGBA8,
            width: 256,
            height: 256,
            levels: MipmapLevels::Partial(1),
        })
        .unwrap();

    let secondary_vertex_data = [
        SecondaryVertex {
            position: [1.0, 1.0],
            color: [255, 0, 0],
        },
        SecondaryVertex {
            position: [-1.0, 1.0],
            color: [0, 255, 0],
        },
        SecondaryVertex {
            position: [0.0, -1.0],
            color: [0, 0, 255],
        },
    ];

    let secondary_vertex_buffer =
        context.create_buffer(secondary_vertex_data, UsageHint::StreamDraw);

    // Our secondary render pass uses a custom render target. This render target only has 1 color
    // attachment: we attach the base level of our texture as a "float" attachment. For details on
    // how to create custom render targets, see the documentation for the `web_glitz::rendering`
    // module.
    let mut secondary_render_target =
        context.create_render_target(RenderTargetDescriptor::new().attach_color_float(
            texture.base_level_mut(),
            LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
            StoreOp::Store,
        ));

    let secondary_render_pass = secondary_render_target.create_render_pass(|framebuffer| {
        framebuffer.pipeline_task(&secondary_pipeline, |active_pipeline| {
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&secondary_vertex_buffer)
                .bind_resources((&BindGroup::empty(), &BindGroup::empty()))
                .draw(3, 1)
                .finish()
        })
    });

    let primary_vertex_data = [
        PrimaryVertex {
            position: [0.0, 0.5],
            texture_coordinates: [0.0, -2.0],
        },
        PrimaryVertex {
            position: [-0.5, -0.5],
            texture_coordinates: [-2.0, 2.0],
        },
        PrimaryVertex {
            position: [0.5, -0.5],
            texture_coordinates: [2.0, 2.0],
        },
    ];

    let primary_vertex_buffer = context.create_buffer(primary_vertex_data, UsageHint::StreamDraw);

    // We'll use a sampler that repeats our texture for texture coordinates outside of the
    // `0.0..=1.0` range.
    let sampler = context.create_sampler(&SamplerDescriptor {
        minification_filter: Linear,
        magnification_filter: Linear,
        wrap_s: Wrap::Repeat,
        wrap_t: Wrap::Repeat,
        ..Default::default()
    });

    let bind_group_1 = context.create_bind_group(PrimaryResources {
        texture: texture.float_sampled(&sampler),
    });

    // Our primary render pass is essentially identical to the render pass used in the
    // `/examples/3_textured_triangle` example.
    let primary_render_pass = default_render_target.create_render_pass(|framebuffer| {
        framebuffer.pipeline_task(&primary_pipeline, |active_pipeline| {
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&primary_vertex_buffer)
                .bind_resources((&BindGroup::empty(), &bind_group_1))
                .draw(3, 1)
                .finish()
        })
    });

    // We use the `sequence_all!` macro again to make sure our secondary render pass finishes before
    // the primary render pass begins.
    context.submit(sequence_all![secondary_render_pass, primary_render_pass]);

    // Let's also retrieve the texture data we generated and output it to the console. We'll first
    // pixel-pack the data into a new buffer and then download the buffer's contents.
    // TODO: introduce way to zero-initialize buffers of "zeroable" types (e.g.
    // `bytemuck::Zeroable`).
    let buffer: Buffer<[[u8; 4]]> =
        context.create_buffer(vec![[0, 0, 0, 0]; 256 * 256], UsageHint::StreamRead);
    let pack_command = texture
        .base_level()
        .pack_to_buffer_command((&buffer).into());
    let download_command = buffer.download_command();

    // GPU tasks produce output when submitted to a context. We're only interested in the output
    // of the download command, so we use `sequence_right` to select just its output and disregard
    // the pack command's output (which is just `()`). The output may not be ready immediately and
    // therefore `submit` returns a future for the output, rather than returning the output
    // directly.
    let future_output = context.submit(sequence_right(pack_command, download_command));

    // We'll have to spawn the future before it can begin executing. We'll use the
    // `wasm-bindgen-futures` crate to run our future on the browser's event loop.
    spawn_local(future_output.map(|pixels| {
        // The output of our download command is a boxed slice of pixel values. We'll use debug
        // formatting to turn it into a string and log it to the console with `web_sys`.
        web_sys::console::log_1(&format!("Pixel data: {:?}", pixels).into());
    }));
}
