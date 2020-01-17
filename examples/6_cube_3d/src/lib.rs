// This example uses the `cgmath` crate to render a 3D cube.
//
// This example on the `1_uniform_block` example. It introduces the concept of indexed drawing.
// We'll also use the `cgmath` crate to help us do the math to get our 3D cube projected onto
// screen space.

#![feature(
    const_fn,
    const_ptr_offset_from,
    const_transmute,
    ptr_offset_from,
    const_loop,
    const_if_match
)]

use std::f32::consts::PI;

use cgmath::{Matrix4, PerspectiveFov, Rad, SquareMatrix, Vector3};
use cgmath_std140::AsStd140;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::pipeline::graphics::{
    CullingMode, DepthTest, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};

use web_sys::{window, HtmlCanvasElement};

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
    let canvas: HtmlCanvasElement = window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    // We won't use use the default context options this time. Instead we build our own options for
    // which we enable the depth buffer: without a depth buffer a depth test can't be performed (see
    // below).
    let (context, render_target) = unsafe {
        single_threaded::init(&canvas, &ContextOptions::begin().enable_depth().finish()).unwrap()
    };

    let vertex_shader = context
        .create_vertex_shader(include_str!("vertex.glsl"))
        .unwrap();

    let fragment_shader = context
        .create_fragment_shader(include_str!("fragment.glsl"))
        .unwrap();

    // Pipeline construction very similar to `2_uniform_buffer`, except this time we also enable the
    // depth test (with default depth test settings).
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

    // We'll use an index list to reuse our vertices multiple times. We've only defined 8 vertices
    // but we want to draw 12 triangles, which each require 3 vertices. We'll use `u16` indices to
    // reference each of our vertices 4 times.
    //
    // TODO: we have to use a `Vec` for now, as Borrow<[u16]> is only implemented for arrays up to
    // length 32 for the time being. I expect this will change as const generics get stabilized,
    // switch to an array when that happens.
    let index_data: Vec<u16> = vec![
        0, 2, 1, // Back
        1, 2, 3, //
        0, 6, 2, // Left
        0, 4, 6, //
        1, 3, 7, // Right
        1, 7, 5, //
        2, 7, 3, // Top
        2, 6, 7, //
        0, 1, 5, // Bottom
        0, 5, 4, //
        4, 5, 7, // Front
        6, 4, 7, //
    ];

    // Create an index buffer. An index buffer is slightly different from a normal buffer in that it
    // may only ever be used for index data for reasons of web security (see the documentation for
    // `web_glitz::pipeline::graphics::IndexBuffer` for details).
    let index_buffer = context.create_index_buffer(index_data, UsageHint::StreamDraw);

    // Specify values for our model, view and projection matrices. We'll slightly rotate the cube
    // along the Y axis and along the X axis to show it off better. We'll move the camera 30 units
    // back along the Z axis. Finally, we use a perspective projection to project our 3D cube onto
    // our flat screen.
    let rotate_x = Matrix4::from_angle_x(Rad(0.25 * PI));
    let rotate_y = Matrix4::from_angle_y(Rad(0.25 * PI));
    let model = rotate_y * rotate_x;
    let view = Matrix4::from_translation(Vector3::new(0.0, 0.0, 30.0))
        .invert()
        .unwrap();
    let projection = Matrix4::from(PerspectiveFov {
        fovy: Rad(0.3 * PI),
        aspect: 1.0,
        near: 1.0,
        far: 100.0,
    });
    let uniforms = Uniforms {
        model: model.as_std140(),
        view: view.as_std140(),
        projection: projection.as_std140(),
    };

    let uniform_buffer = context.create_buffer(uniforms, UsageHint::StreamDraw);

    let bind_group_0 = context.create_bind_group(Resources {
        uniforms: &uniform_buffer,
    });

    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        // This time we'll also bind our index buffer with `bind_index_buffer`. Then, instead of
        // drawing with `draw`, we draw with `draw_indexed` where we specify the number of indices
        // to use (36) and the number of instances to render (1).
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
