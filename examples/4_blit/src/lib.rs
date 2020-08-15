// This example renders to a secondary render target and blits the resulting image to the
// default render target.
//
// This example builds on `/examples/0_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn, const_ptr_offset_from, const_raw_ptr_deref, ptr_offset_from)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};

use web_glitz::buffer::UsageHint;
use web_glitz::image::format::RGBA8;
use web_glitz::image::renderbuffer::RenderbufferDescriptor;
use web_glitz::image::Region2D;
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::rendering::{LoadOp, RenderTargetDescriptor, StoreOp};
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

    // We'll disable antialiasing on the default render target for this example, as blit operations
    // are only available on single-sample render targets.
    let options = ContextOptions::begin().disable_antialias().finish();

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

    // We create a Renderbuffer that will serve as the color target for our secondary render target.
    let mut renderbuffer = context.create_renderbuffer(&RenderbufferDescriptor {
        format: RGBA8,
        width: 500,
        height: 500,
    });

    // This render pass is largely equivalent to the render pass in `/examples/0_triangle`, except
    // that here we use a custom render target that uses our `renderbuffer`, rather than the default
    // render target.
    let mut secondary_render_target =
        context.create_render_target(RenderTargetDescriptor::new().attach_color_float(
            &mut renderbuffer,
            LoadOp::Clear([0.0, 0.0, 0.0, 0.0]),
            StoreOp::Store,
        ));

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

    // This second pass blits the image in the renderbuffer to the color buffer of the default
    // render target.
    let blit_pass = default_render_target.create_render_pass(|framebuffer| {
        framebuffer.blit_color_linear_command(Region2D::Fill, &renderbuffer)
    });

    // `/examples/0_triangle` only had to submit a single render pass, but here we must submit both
    // the secondary render pass and the blit pass. It's important that the secondary pass finishes
    // before we begin the blit pass so we'll use the `sequence_all!` macro to combine them into a
    // sequenced task, which guarantees that the tasks are executed in order.
    context.submit(sequence_all![secondary_render_pass, blit_pass]);
}
