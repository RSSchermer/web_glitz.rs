// This example shows how to bind a texture resource to a pipeline.
//
// This example builds on `/examples/3_textured_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};

use web_glitz::buffer::UsageHint;
use web_glitz::image::format::RGBA8;
use web_glitz::image::texture_2d::{FloatSampledTexture2D, Texture2DDescriptor};
use web_glitz::image::MipmapLevels;
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, SlotBindingStrategy, WindingOrder,
};
use web_glitz::render_target::{FloatAttachment, LoadOp, RenderTarget, StoreOp};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::sampler::{MagnificationFilter, MinificationFilter, SamplerDescriptor, Wrap};
use web_glitz::task::sequence_all;
use web_glitz::vertex::VertexArrayDescriptor;

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
    #[texture_resource(binding = 0, name = "diffuse_texture")]
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

    let (context, default_render_target) =
        unsafe { single_threaded::init(&canvas, &ContextOptions::default()).unwrap() };

    let primary_vertex_shader = context
        .create_vertex_shader(include_str!("primary_vertex.glsl"))
        .unwrap();

    let primary_fragment_shader = context
        .create_fragment_shader(include_str!("primary_fragment.glsl"))
        .unwrap();

    let primary_pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&primary_vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&primary_fragment_shader)
                .vertex_input_layout::<PrimaryVertex>()
                .resource_layout::<PrimaryResources>(SlotBindingStrategy::Update)
                .finish(),
        )
        .unwrap();

    let secondary_vertex_shader = context
        .create_vertex_shader(include_str!("secondary_vertex.glsl"))
        .unwrap();

    let secondary_fragment_shader = context
        .create_fragment_shader(include_str!("secondary_fragment.glsl"))
        .unwrap();

    let secondary_pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&secondary_vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&secondary_fragment_shader)
                .vertex_input_layout::<SecondaryVertex>()
                .finish(),
        )
        .unwrap();

    // We have to mark the texture as `mut` here, as using a texture as a render target attachment
    // (see below) requires a mut reference to a texture image.
    let mut texture = context
        .create_texture_2d(&Texture2DDescriptor {
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

    let secondary_vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
        vertex_input_state: &secondary_vertex_buffer,
        indices: (),
    });

    // Our secondary render pass uses a custom render target. This render target only has 1 color
    // attachment: we attach the base level of our texture as a "float" attachment. For details on
    // how to create custom render targets, see the documentation for the `web_glitz::render_target`
    // module.
    let secondary_render_pass = context.create_render_pass(
        RenderTarget {
            color: FloatAttachment {
                // Note that we need to provide a mut reference to the texture image. This prevents
                // us from accidentally reading from the same texture elsewhere in the render pass
                // (by attaching it as a pipeline resource), as reading from a texture while it is
                // also attached to the current render target would cause undefined behaviour.
                image: texture.base_level_mut(),
                load_op: LoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
                store_op: StoreOp::Store,
            },
            depth_stencil: (),
        },
        |framebuffer| {
            framebuffer.pipeline_task(&secondary_pipeline, |active_pipeline| {
                active_pipeline.draw_command(&secondary_vertex_array, ())
            })
        },
    );

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

    let primary_vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
        vertex_input_state: &primary_vertex_buffer,
        indices: (),
    });

    // We'll use a sampler that repeats our texture for texture coordinates outside of the
    // `0.0..=1.0` range.
    let sampler = context.create_sampler(&SamplerDescriptor {
        minification_filter: MinificationFilter::Linear,
        magnification_filter: MagnificationFilter::Linear,
        wrap_s: Wrap::Repeat,
        wrap_t: Wrap::Repeat,
        ..Default::default()
    });

    // Our primary render pass is essentially identical to the render pass used in the
    // `/examples/3_textured_triangle` example.
    let primary_render_pass = context.create_render_pass(default_render_target, |framebuffer| {
        framebuffer.pipeline_task(&primary_pipeline, |active_pipeline| {
            active_pipeline.draw_command(
                &primary_vertex_array,
                PrimaryResources {
                    texture: texture.float_sampled(&sampler).unwrap(),
                },
            )
        })
    });

    // We use the `sequence_all!` macro again to make sure our secondary render pass finishes before
    // the primary render pass begins.
    context.submit(sequence_all![secondary_render_pass, primary_render_pass]);
}