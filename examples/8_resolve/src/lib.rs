// This example renders to a secondary multisample render target and resolves the resulting
// multisample image onto the default render target.
//
// This example is very similar to on `/examples/4_blit`, except it uses a multisample render target
// and uses the resolve operation rather than a blit operation to transfer the image data to the
// default render target.

#![feature(
const_fn_trait_bound,
    const_maybe_uninit_as_ptr,
    const_ptr_offset_from,
    const_raw_ptr_deref
)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};

use web_glitz::buffer::UsageHint;
use web_glitz::image::format::{Multisample, RGBA8};
use web_glitz::image::renderbuffer::RenderbufferDescriptor;
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::rendering::{LoadOp, MultisampleRenderTargetDescriptor, StoreOp};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::task::sequence_all;

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],

    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

#[wasm_bindgen(start)]
pub fn start() {
    let canvas: HtmlCanvasElement = window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    // We'll disable antialiasing on the default render target for this example, as resolve
    // operations require the destination to be a single-sample render target.
    let options = ContextOptions::begin()
        .enable_depth()
        .disable_antialias()
        .finish();

    let (context, mut default_render_target) =
        unsafe { single_threaded::init(&canvas, &options).unwrap() };

    let vertex_shader = context
        .try_create_vertex_shader(include_str!("vertex.glsl"))
        .unwrap();

    let fragment_shader = context
        .try_create_fragment_shader(include_str!("fragment.glsl"))
        .unwrap();

    let pipeline = context
        .try_create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&fragment_shader)
                .typed_vertex_attribute_layout::<Vertex>()
                .typed_resource_bindings_layout::<((), ())>()
                .finish(),
        )
        .unwrap();

    let vertex_data = [
        Vertex {
            position: [0.0, 0.5],
            color: [255, 0, 0],
        },
        Vertex {
            position: [-0.5, -0.5],
            color: [0, 255, 0],
        },
        Vertex {
            position: [0.5, -0.5],
            color: [0, 0, 255],
        },
    ];

    let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StreamDraw);

    let samples = context
        .supported_samples(RGBA8)
        .max_samples()
        .expect("Multisampling not available for RGBA8!");

    // We create a multisample Renderbuffer that will serve as the color target for our secondary
    // render target.
    let mut renderbuffer = context
        .try_create_multisample_renderbuffer(&RenderbufferDescriptor {
            format: Multisample(RGBA8, samples),
            width: 500,
            height: 500,
        })
        .unwrap();

    let mut secondary_render_target = context.create_multisample_render_target(
        MultisampleRenderTargetDescriptor::new(samples).attach_color_float(
            &mut renderbuffer,
            LoadOp::Clear([0.0, 0.0, 0.0, 0.0]),
            StoreOp::Store,
        ),
    );

    let secondary_render_pass = secondary_render_target.create_render_pass(|framebuffer| {
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&vertex_buffer)
                .bind_resources((&BindGroup::empty(), &BindGroup::empty()))
                .draw(3, 1)
                .finish()
        })
    });

    // This second pass resolves a single-sample image from the multisample image in the
    // renderbuffer, into the color buffer of the default render target.
    let resolve_pass = default_render_target
        .create_render_pass(|framebuffer| framebuffer.resolve_color_command(&renderbuffer));

    context.submit(sequence_all![secondary_render_pass, resolve_pass]);
}
