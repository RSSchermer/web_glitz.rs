#![feature(const_fn)]

#[macro_use]
extern crate web_glitz;
extern crate console_error_panic_hook;

use std::panic;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::pipeline::graphics::vertex_input::{Vertex, VertexArrayDescriptor};
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, Topology, WindingOrder,
};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};

use web_glitz::buffer::BufferUsage;
use web_sys::{window, HtmlCanvasElement};

#[derive(Vertex)]
struct SimpleVertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float3_f32")]
    color: [f32; 3],
}

#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let canvas: HtmlCanvasElement = window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector("#canvas")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    let (context, render_target) =
        unsafe { single_threaded::context(&canvas, &ContextOptions::default()).unwrap() };

    let vertex_shader = context.create_vertex_shader(include_str!("vertex.glsl").to_string());
    let fragment_shader = context.create_fragment_shader(include_str!("fragment.glsl").to_string());

    let pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly {
                    topology: Topology::Triangle,
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&fragment_shader)
                .vertex_input_layout::<SimpleVertex>()
                .finish(),
        )
        .unwrap();

    let vertex_data = [
        SimpleVertex {
            position: [0.0, 0.5],
            color: [1.0, 0.0, 0.0],
        },
        SimpleVertex {
            position: [-0.5, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        SimpleVertex {
            position: [0.5, -0.5],
            color: [0.0, 0.0, 1.0],
        },
    ];

    let vertex_buffer = context.create_buffer(vertex_data, BufferUsage::StaticDraw);

    let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
        vertex_buffers: vertex_buffer,
        index_buffer: (),
    });

    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            active_pipeline.draw_command(&vertex_array, &())
        })
    });

    context.submit(render_pass);
}
