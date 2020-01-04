// This example shows how to record transform feedback from a pipeline task.
//
// This example builds on `/examples/1_uniform_block`, the comments in this example will focus on
// the differences/additions.

#![feature(
    const_fn,
    const_ptr_offset_from,
    const_transmute,
    ptr_offset_from,
    const_loop,
    const_if_match
)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::task::sequence;

use web_sys::{window, HtmlCanvasElement};

// In this example we'll use the same type both as our "Vertex" type and as our "TransformFeedback"
// type. To facilitate this in a safe way we derive `web_glitz::derive::TransformFeedback` in
// addition to deriving `web_glitz::derive::Vertex` as we did in `/examples/1_uniform_block`. Also,
// to successfully use this type with our graphics pipeline, we need to make sure that the field
// names we use exactly match the names of the `out` values that we wish to record in our vertex
// shader (see `./vertex.glsl`). We must also ensure that the field types we use are compatible with
// the GLSL types used for these `out` values. This will be verified by reflecting on the shader
// code when we create our pipeline.
#[derive(web_glitz::derive::Vertex, web_glitz::derive::TransformFeedback, Clone, Copy, Default)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    varying_position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float3_f32")]
    varying_color: [f32; 3],
}

#[std140::repr_std140]
#[derive(web_glitz::derive::InterfaceBlock, Clone, Copy)]
struct Uniforms {
    scale: std140::float,
}

#[derive(web_glitz::derive::Resources)]
struct Resources<'a> {
    #[resource(binding = 0, name = "Uniforms")]
    uniforms: &'a Buffer<Uniforms>,
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

    let (context, render_target) =
        unsafe { single_threaded::init(&canvas, &ContextOptions::default()).unwrap() };

    let vertex_shader = context
        .create_vertex_shader(include_str!("vertex.glsl"))
        .unwrap();

    let fragment_shader = context
        .create_fragment_shader(include_str!("fragment.glsl"))
        .unwrap();

    // Create a pipeline.
    //
    // Our pipeline is very similar to the pipeline we used in `/examples/1_uniform_block`, except
    // this time we specify a typed transform feedback layout. We'll use the same `Vertex` type that
    // we also use as our vertex type.
    let mut pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&fragment_shader)
                .typed_vertex_attribute_layout::<Vertex>()
                .typed_transform_feedback_layout::<Vertex>()
                .typed_resource_bindings_layout::<(Resources, ())>()
                .finish(),
        )
        .unwrap();

    let vertex_data = [
        Vertex {
            varying_position: [0.0, 0.5],
            varying_color: [1.0, 0.0, 0.0],
        },
        Vertex {
            varying_position: [-0.5, -0.5],
            varying_color: [0.0, 1.0, 0.0],
        },
        Vertex {
            varying_position: [0.5, -0.5],
            varying_color: [0.0, 0.0, 1.0],
        },
    ];

    let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StreamDraw);

    let uniforms = Uniforms {
        scale: std140::float(-0.5),
    };

    let uniform_buffer = context.create_buffer(uniforms, UsageHint::StreamDraw);

    let bind_group_0 = context.create_bind_group(Resources {
        uniforms: &uniform_buffer,
    });

    // We'll use this buffer to record the transform feedback. We use `UsageHint::StreamCopy` as the
    // data will be written by the device and is then used to draw only once.
    let mut transform_feedback_buffer =
        context.create_buffer([Vertex::default(); 3], UsageHint::StreamCopy);

    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        // Our render pass consists of 2 pipeline tasks: the first one will record transform
        // feedback into the `transform_feedback_buffer` and use our original vertex data as stored
        // in `vertex_buffer` as its vertex input; the second one will not record transform feedback
        // and will use the feedback we recorded into `transform_feedback_buffer` as its vertex
        // input. Note that to keep this example simple we use the same pipeline for both tasks; for
        // a real use case you would typically use different pipelines.
        //
        // We tell WebGlitz to record the feedback by wrapping our pipeline in a "recording wrapper"
        // by calling `record_transform_feedback` with an exclusive reference to the buffer we wish
        // to record to (or a tuple of exclusive references to buffers should we wish to record
        // different `out` values to separate buffers). Note that the borrow checker statically
        // protects us against accidentally accessing the same buffer again inside our pipeline task
        // (e.g. as vertex input), which would cause undefined behaviour.
        sequence(
            framebuffer.pipeline_task(
                &pipeline.record_transform_feedback(&mut transform_feedback_buffer),
                |active_pipeline| {
                    active_pipeline
                        .task_builder()
                        .bind_vertex_buffers(&vertex_buffer)
                        .bind_resources((&bind_group_0, &BindGroup::empty()))
                        .draw(3, 1)
                        .finish()
                },
            ),
            framebuffer.pipeline_task(&pipeline, |active_pipeline| {
                active_pipeline
                    .task_builder()
                    .bind_vertex_buffers(&transform_feedback_buffer)
                    .bind_resources((&bind_group_0, &BindGroup::empty()))
                    .draw(3, 1)
                    .finish()
            }),
        )
    });

    context.submit(render_pass);
}
