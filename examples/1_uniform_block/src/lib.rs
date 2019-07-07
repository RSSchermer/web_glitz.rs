// This example shows how to bind a buffer resource to a uniform block in a pipeline.
//
// Note that WebGlitz does not support "plain"/"non-opaque" uniforms, only sampler uniforms and
// uniform blocks are supported; if you want to use a a simple `float`, `vec4`, `mat4`, etc.
// uniform in your pipeline, it must be part of a uniform block. If you do attempt to use plain
// uniforms in your shaders, you will receive an error upon pipeline creation. However, WebGlitz
// attempts to make it very convenient to use uniform blocks that use the `std140` memory layout.
//
// This example builds on `/examples/0_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, SlotBindingStrategy, WindingOrder,
};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::std140;
use web_glitz::std140::repr_std140;
use web_glitz::vertex::VertexArrayDescriptor;

use web_sys::{window, HtmlCanvasElement};

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

// Define our uniform block type.
//
// We store an instance of this type in a GPU-accessible buffer to provide the data for our
// uniform block (as defined in `/src/primary_vertex.glsl`). We've told the driver that we'll be providing
// the data for the block using the `std140` layout, by annotating it with `layout(std140)`. We'll
// make sure that our Rust struct will follow the `std140` memory layout rules by adding the
// `#[repr_std140]` attribute to our struct declaration. `#[repr_std140]` can only be added to
// structs (not to enums or unions) and only if all fields of the struct implement
// `web_glitz::std140::ReprStd140` (a marker trait implemented for types that can be used as fields
// in `#[repr_std140]` structs), otherwise the struct will fail to compile. `ReprStd140` is
// implemented for all types in `web_glitz::std140` and is automatically implement for any struct
// that successfully compiles with the `#[repr_std140]`  attribute (which means our struct here will
// implement `ReprStd140`).
//
// Marking the struct with `#[repr_std140]` is not quite enough for us to use it with a uniform
// block, we must also derive `web_glitz::derive::InterfaceBlock`. This trait can only be derived on
// a struct when it implements `web_glitz::pipeline::interface_block::StableRepr` and when every
// field implements `web_glitz::pipeline::interface_block::InterfaceBlockComponent`:
//
// - `StableRepr` is a marker trait for types that will have a stable memory representation across
//   compilations. By default, the Rust compiler gives only very few guarantees about a struct's
//   memory layout. This allows the compiler to optimize the layout for performance and/or memory
//   footprint. However, this means that a different/future version of the Rust compiler might
//   produce a different memory layout for your type; on the old version the type's layout
//   (accidentally) matched the layout the pipeline expected, but now you're suddenly getting
//   errors! `StableRepr` is meant to ensure that you don't accidentally fall into this trap.
//   `StableRepr` implemented for all types that implement `ReprStd140`, including types marked
//   with `#[repr_std140]`.
// - `InterfaceBlockComponent` adds memory layout metadata to our struct which will be verified
//   against our pipeline when the pipeline is created; if our struct's memory layout does not match
//   the uniform block layout expected by the pipeline, we will receive an error when we attempt to
//   create the pipeline. `InterfaceBlockComponent` is implemented for all types in
//   `web_glitz::std140`.
//
// Note that although the field name matches the name of the block member in this example, this is
// not strictly necessary. Only the following has to match for us to be able to use our struct with
// a pipeline:
//
// - Positions: if your uniform block contains multiple members, then the order of the fields in the
//   struct must correspond to the order of the fields in the uniform block.
// - Field layout: the type and alignment of each struct field must match the memory layout of the
//   uniform block member in the corresponding position.
//
// In this case our very simple uniform block contains only one `float` member, so we'll match that
// in our struct.
#[repr_std140]
#[derive(web_glitz::derive::InterfaceBlock, Clone, Copy)]
struct Uniforms {
    scale: std140::float,
}

// Define our resources type and derive `web_glitz::derive::Resources`.
//
// We'll provide an instance of this type when we invoke our pipeline to supply it with the
// resources our pipeline needs access to. In this case we'll need only one resource: a buffer
// resource for our uniform block.
#[derive(web_glitz::derive::Resources)]
struct Resources<'a> {
    // We'll have to mark the field that will hold our buffer resource with a
    // `#[buffer_resource(...)]` attribute. We can only use this attribute on fields that implement
    // `web_glitz::pipeline::resources::BufferResource`, otherwise our struct will fail to compile.
    // We have to specify a positive integer for the `binding` index to which we'll bind the
    // resource, in this case we'll use `0`. If we were to use multiple buffer resources, then
    // we'd have to make sure that each buffer resource is bound to a unique `binding` index,
    // otherwise our struct would fail to compile (we can't bind more than one buffer resource to
    // the same buffer resource binding). Here we use only one buffer resource though, so
    // we don't have to worry about that.
    //
    // If the name of our field does not exactly match (case-sensitive) the name of the uniform
    // block we want to bind our buffer to, then we must also specify a `name`; in this case there
    // is not an exact match, so we explicitly specify the `name` of the block we want to bind to as
    // "Uniforms".
    #[buffer_resource(binding = 0, name = "Uniforms")]
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

    // Create our pipeline.
    //
    // Our pipeline is very similar to the pipeline we used in `/examples/0_triangle`, except this
    // time our vertex shader uses a uniform block, so we must declare a `resource_layout` type that
    // will provide a buffer to back it. We'll specify the `Resources` type we defined above.
    //
    // We'll also have to specify a `web_glitz::pipeline::resources::BindingStrategy`. We'll use
    // `BindingStrategy::Update`, to indicate that we wish to override the pipeline's default
    // bindings with the values we specified on our `Resources` type.
    let pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    winding_order: WindingOrder::CounterClockwise,
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&fragment_shader)
                .vertex_input_layout::<Vertex>()
                .resource_layout::<Resources>(SlotBindingStrategy::Update)
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

    let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
        vertex_input_state: &vertex_buffer,
        indices: (),
    });

    // Create an instance of our `Uniforms` type.
    let uniforms = Uniforms {
        scale: std140::float(0.5),
    };

    // Create a GPU-accessible buffer that holds our `uniforms` instance. In this case we'll only
    // draw once and we wont be updating this buffer, so in this simple example we'll use
    // `BufferUsage::StreamDraw` as the usage hint.
    let uniform_buffer = context.create_buffer(uniforms, UsageHint::StreamDraw);

    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            // Our render pass has thus far been identical to the render pass in
            // `/examples/0_triangle`. However, our pipeline now does use resources, so rather than
            // specifying the empty tuple `()` as the second argument to our draw command, we'll
            // instead provide an instance of our `Resources` type.
            //
            // Note that, as with the vertex array, WebGlitz wont have to do any additional runtime
            // safety checks here to ensure that the resources are compatible with the pipeline: we
            // checked this when we created the pipeline and we can now leverage Rust's type system
            // again to enforce safety at compile time.
            active_pipeline.draw_command(
                &vertex_array,
                Resources {
                    uniforms: &uniform_buffer,
                },
            )
        })
    });

    context.submit(render_pass);

    // We should now see our triangle on the canvas again, except this time it should be half the
    // size of the triangle we produced in `/examples/0_triangle`.
}
