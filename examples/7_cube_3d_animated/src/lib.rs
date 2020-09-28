// This example uses the `cgmath` crate to render a 3D cube.
//
// This example on the `6_cube_3d` example. It is very similar, except this time we'll be drawing
// many times, on every animation frame the browser can provide. On each frame we'll update the
// contents of our uniform buffer to make the cube spin.
//
// So far we`ve been using plain `web_sys` to interact with the browser, but now we'll use the
// `arwa` crate to help us request animation frames. We'll also need the `futures` and
// `wasm_bindgen_futures` crates. Finally, we need the `fn_traits`, `unboxed_closures` to create a
// stateful self-referential function that we can call in a "loop" on each animation frame.

#![feature(
    const_fn,
    const_maybe_uninit_as_ptr,
    const_ptr_offset_from,
    const_raw_ptr_deref,
    fn_traits,
    unboxed_closures
)]

use std::convert::TryInto;
use std::f32::consts::PI;

use arwa::html::HtmlCanvasElement;
use arwa::{window, AnimationFrameCancelled, Document, Window};

use cgmath::{Matrix4, PerspectiveFov, Rad, SquareMatrix, Vector3};
use cgmath_std140::AsStd140;

use futures::FutureExt;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::pipeline::graphics::{
    CullingMode, DepthTest, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::runtime::single_threaded::SingleThreadedContext;
use web_glitz::runtime::{single_threaded, Connection, ContextOptions, RenderingContext};
use web_glitz::task::{sequence, GpuTask};

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float4_f32")]
    position: [f32; 4],
    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

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

// Our animation frame loop. We implement the `FnOnce` trait for this type so that we can use it as
// a callback for an animation frame, while we hold onto (and can update, if necessary) the state
// that's relevant to our rendering code inside this struct.
struct AnimationLoop<T> {
    rendering_context: SingleThreadedContext,
    uniform_buffer: Buffer<Uniforms>,
    render_pass: T,
    view: std140::mat4x4,
    projection: std140::mat4x4,
    frame_provider: Window,
}

impl<T> FnOnce<(Result<f64, AnimationFrameCancelled>,)> for AnimationLoop<T>
where
    T: GpuTask<Connection> + Clone + 'static,
{
    type Output = ();

    extern "rust-call" fn call_once(
        self,
        args: (Result<f64, AnimationFrameCancelled>,),
    ) -> Self::Output {
        // Only render if the frame has not been cancelled.
        if let Ok(time) = args.0 {
            let time = time as f32;

            // Compute the new values for our uniforms
            let rotate_x = Matrix4::from_angle_x(Rad(time / 1000.0));
            let rotate_y = Matrix4::from_angle_y(Rad(time / 1000.0));
            let model = rotate_y * rotate_x;
            let uniforms = Uniforms {
                model: model.as_std140(),
                view: self.view,
                projection: self.projection,
            };

            // Create a command that updates the uniform buffer.
            let update_command = self.uniform_buffer.upload_command(uniforms);

            // Submit a task that sequences the update command with our render pass. Note that we've
            // preconstructed our render pass task and that for each frame we submit a clone,
            // without having to reconstruct our render pass task every time.
            self.rendering_context
                .submit(sequence(update_command, self.render_pass.clone()));

            // Create a request for the next frame.
            let next = self.frame_provider.request_animation_frame();

            // Dispatch the request with another call to this function chained onto it.
            spawn_local(next.map(self));
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvas: HtmlCanvasElement = document
        .query_id("canvas")
        .expect("No element with id `canvas`.")
        .try_into()
        .expect("Element is not a canvas element.");

    let (context, mut render_target) = unsafe {
        single_threaded::init(
            canvas.as_ref(),
            &ContextOptions::begin().enable_depth().finish(),
        )
        .unwrap()
    };

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

    let index_buffer = context.create_index_buffer(index_data, UsageHint::StreamDraw);

    let view = Matrix4::from_translation(Vector3::new(0.0, 0.0, 30.0))
        .invert()
        .unwrap()
        .as_std140();
    let projection = Matrix4::from(PerspectiveFov {
        fovy: Rad(0.3 * PI),
        aspect: 1.0,
        near: 1.0,
        far: 100.0,
    })
    .as_std140();
    let uniforms = Uniforms {
        model: Matrix4::identity().as_std140(),
        view,
        projection,
    };

    let uniform_buffer = context.create_buffer(uniforms, UsageHint::StreamDraw);

    let bind_group_0 = context.create_bind_group(Resources {
        uniforms: &uniform_buffer,
    });

    let render_pass = render_target.create_render_pass(|framebuffer| {
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

    // Instantiate our animation loop.
    let animation_loop = AnimationLoop {
        rendering_context: context,
        uniform_buffer,
        render_pass,
        view,
        projection,
        frame_provider: window.clone(),
    };

    // Create the initial animation frame request.
    let request = window.request_animation_frame();

    // Dispatch our request with the initial call to our animation loop chained onto it.
    spawn_local(request.map(animation_loop));

    // We should now see a spinning cube on the canvas!
}
