// This example uses the `cgmath` crate to render a 3D cube.
//
// This example does not introduce new WebGlitz concepts over the `1_uniform_block` example, most of
// the differences are found in the shader code. We'll use the `cgmath` crate to help us do the
// math.

#![feature(
const_fn,
const_ptr_offset_from,
const_transmute,
ptr_offset_from,
const_loop,
const_if_match
)]

use std::f32::consts::PI;

use cgmath::{Matrix4, PerspectiveFov, Vector3, SquareMatrix, Rad};
use cgmath_std140::AsStd140;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::pipeline::graphics::{CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder, DepthTest};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};

use web_sys::{window, HtmlCanvasElement};
use web_glitz::pipeline::interface_block::InterfaceBlock;


#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float4_f32")]
    position: [f32; 4],
    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

// We'll use 3 separate 4x4 matrices to represent the "model", "view" and "projection"
// transformations:
//
// - The "model" transformation transforms model space coordinates to world space coordinates; our
//   vertex positions will start as model space coordinates.
// - The "view" transformation transforms world space coordinates into view (camera) space
//   coordinates.
// - The "projection" transformation transforms view space coordinates into screen space
//   coordinates; we'll be using a "perspective" projection.
#[std140::repr_std140]
#[derive(web_glitz::derive::InterfaceBlock, Clone, Copy)]
struct Uniforms {
    model: std140::mat4x4,
    view: std140::mat4x4,
    projection: std140::mat4x4,
}

#[derive(web_glitz::derive::Resources)]
struct Resources<'a> {
    #[resource(binding = 0, name = "Uniforms")]
    uniforms: &'a Buffer<Uniforms>,
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
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

    let pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&fragment_shader)
                .typed_vertex_attribute_layout::<Vertex>()
                .typed_resource_bindings_layout::<(Resources, ())>()
                .enable_depth_test(DepthTest::default())
                .finish(),
        )
        .unwrap();

    let vertex_data = [
        Vertex {
            position: [-5.0, -5.0, -5.0, 1.0],
            color: [255, 0, 0],
        },
        Vertex {
            position: [5.0, -5.0, -5.0, 1.0],
            color: [0, 255, 0],
        },
        Vertex {
            position: [-5.0, 5.0, -5.0, 1.0],
            color: [0, 0, 255],
        },
        Vertex {
            position: [5.0, 5.0, -5.0, 1.0],
            color: [255, 0, 0],
        },
        Vertex {
            position: [-5.0, -5.0, 5.0, 1.0],
            color: [0, 255, 255],
        },
        Vertex {
            position: [5.0, -5.0, 5.0, 1.0],
            color: [0, 0, 255],
        },
        Vertex {
            position: [-5.0, 5.0, 5.0, 1.0],
            color: [255, 255, 0],
        },
        Vertex {
            position: [5.0, 5.0, 5.0, 1.0],
            color: [0, 255, 0],
        },
    ];

    let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StreamDraw);

    let index_data: [u16; 36] = [
        0, 2, 1, // Back
        1, 2, 3,
        0, 6, 2, // Left
        0, 4, 6,
        1, 3, 7, // Right
        1, 7, 5,
        2, 7, 3, // Top
        2, 6, 7,
        0, 1, 5, // Bottom
        0, 5, 4,
        4, 5, 7, // Front
        6, 4, 7
    ];

    let index_buffer = context.create_buffer(index_data, UsageHint::StreamDraw);

    let uniforms = Uniforms {
        model: (Matrix4::from_angle_y(Rad(0.25 * PI)) * Matrix4::from_angle_x(Rad(0.25 * PI))).as_std140(),
        view: Matrix4::from_translation(Vector3::new(0.0, 0.0, 30.0)).invert().unwrap().as_std140(),
        projection: Matrix4::from(PerspectiveFov {
            fovy: Rad(0.3 * PI),
            aspect: 1.0,
            near: 1.0,
            far: 100.0
        }).as_std140()
    };

    let uniform_buffer = context.create_buffer(uniforms, UsageHint::StreamDraw);

    let bind_group_0 = context.create_bind_group(Resources {
        uniforms: &uniform_buffer,
    });

    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&vertex_buffer)
                .bind_index_buffer(&index_buffer)
                .bind_resources((&bind_group_0, &BindGroup::empty()))
                .draw_indexed(36, 1)
                .finish()
        })
    });

    context.submit(render_pass);

    // We should now see a rotated cube on the canvas!
}
