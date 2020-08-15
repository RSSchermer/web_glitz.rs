// This example shows how to bind a buffer resource to a uniform block in a pipeline.
//
// Note that WebGlitz does not support "plain"/"non-opaque" uniforms, only sampler uniforms and
// uniform blocks are supported; if you want to use a simple `float`, `vec4`, `mat4`, etc.
// uniform in your pipeline, then it must be part of a uniform block. If you do attempt to use plain
// uniforms in your shaders, you will receive an error upon pipeline creation. However, WebGlitz
// attempts to make it very convenient to use uniform blocks that use the `std140` memory layout.
//
// This example builds on `/examples/0_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn, const_ptr_offset_from, const_raw_ptr_deref, ptr_offset_from)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};

use web_sys::{window, HtmlCanvasElement};

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

// Define a uniform block type.
//
// We store an instance of this type in a GPU-accessible buffer to provide the data for our uniform
// block (as defined in `/src/primary_vertex.glsl`). We've told the driver that we'll be providing
// the data for the block using the `std140` layout, by annotating it with `layout(std140)` in the
// shader code. We can use the `std140` crate (see Cargo.toml) to make sure that our Rust struct is
// compatible with the `std140` memory layout rules by adding the `#[std140::repr_std140]` attribute
// to our struct declaration. `#[std140::repr_std140]` can only be added to structs (not to enums or
// unions), and only if all fields of the struct implement `std140::ReprStd140` (a marker trait
// implemented for types that can be used as fields in `#[std::repr_std140]` structs), otherwise the
// struct will fail to compile. `std140::ReprStd140` is implemented for all types in `std140` and is
// automatically implement for any struct that successfully compiles with the
// `#[std140::repr_std140]` attribute.
//
// Marking the struct with `#[std140::repr_std140]` is not quite enough for us to use it with a
// uniform block, we must also derive `web_glitz::derive::InterfaceBlock`. This trait can only be
// derived on a struct when it implements `web_glitz::pipeline::interface_block::StableRepr` and
// when every field implements `web_glitz::pipeline::interface_block::InterfaceBlockComponent`:
//
// - `StableRepr` is a marker trait for types that will have a stable memory representation across
//   compilations. By default, the Rust compiler gives only limited guarantees about a struct's
//   memory layout. This allows the compiler to optimize the layout for performance and/or memory
//   footprint. However, this means that a different/future version of the Rust compiler may
//   produce a different memory layout for your type: on the old version the type's layout may have
//   (accidentally) matched the layout the pipeline expected, but on the new version you're suddenly
//   getting errors! `StableRepr` is meant to ensure that you don't accidentally fall into this
//   trap. `StableRepr` implemented for all types that implement `std140::ReprStd140`, including
//   types marked with `#[std140::repr_std140]`.
// - `InterfaceBlockComponent` adds memory layout metadata to our struct which will be verified
//   against our pipeline when the pipeline is created; if our struct's memory layout does not match
//   the uniform block layout expected by the pipeline, we will receive an error when we attempt to
//   create the pipeline. `InterfaceBlockComponent` is implemented for all types in `std140`.
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
#[std140::repr_std140]
#[derive(web_glitz::derive::InterfaceBlock, Clone, Copy)]
struct Uniforms {
    scale: std140::float,
}

// Define a resources type and derive `web_glitz::derive::Resources`.
//
// We'll use this type to create a "bind group": a group of resources that may be bound to a GPU
// pipeline, such that each of the pipeline's invocations may access these resource when the
// pipeline is executing. In this case, our bind group will consist of only a single resource: a
// buffer resource for our uniform block.
#[derive(web_glitz::derive::Resources)]
struct Resources<'a> {
    // We'll have to mark the field that will hold our buffer resource with a `#[resource(...)]`
    // attribute. We can only use this attribute on fields that implement
    // `web_glitz::pipeline::resources::BufferResource`, otherwise our struct will fail to compile.
    // We have to specify a positive integer for the `binding` index to which we'll bind the
    // resource, in this case we'll use `0`. If we were to use multiple resources, then we'd have to
    // make sure that each buffer resource is bound to a unique `binding` index, otherwise our
    // struct would fail to compile (we can't bind more than one resource to the same resource
    // binding).
    //
    // If the name of the field does not match exactly (case-sensitive) the name of the uniform
    // block we want to bind our buffer to, then we must also specify a `name`; in this case there
    // is not an exact match, so we explicitly specify the `name` of the block we want to bind to as
    // "Uniforms".
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

    let (context, mut render_target) =
        unsafe { single_threaded::init(&canvas, &ContextOptions::default()).unwrap() };

    let vertex_shader = context
        .try_create_vertex_shader(include_str!("vertex.glsl"))
        .unwrap();

    let fragment_shader = context
        .try_create_fragment_shader(include_str!("fragment.glsl"))
        .unwrap();

    // Create a pipeline.
    //
    // Our pipeline is very similar to the pipeline we used in `/examples/0_triangle`, except this
    // time our vertex shader uses a uniform block, so we must declare a resource layout. The
    // resource layout must match the resource layout used in our shader code. Note that GLSL ES
    // 3.0 does not allow us to specify explicit bind groups for resources in the shader code.
    // Instead, WebGL 2.0 defines 2 implicit bind groups: 1 bind group for all uniform buffers
    // (bind group `0`) and 1 bind group for all texture-samplers (bind group `1`). We'll use our
    // `Resources` type to specify the layout for bind group `0`. We're not using any textures, so
    // we'll use the empty tuple `()` to specify an empty layout for bind group `1`.
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

    // Create an instance of our `Uniforms` type.
    let uniforms = Uniforms {
        scale: std140::float(0.5),
    };

    // Create a GPU-accessible buffer that holds our `uniforms` instance. In this case we'll only
    // draw once and we wont be updating this buffer, so in this simple example we'll use
    // `BufferUsage::StreamDraw` as the usage hint.
    let uniform_buffer = context.create_buffer(uniforms, UsageHint::StreamDraw);

    // Create a bind group for our `Resources` type.
    let bind_group_0 = context.create_bind_group(Resources {
        uniforms: &uniform_buffer,
    });

    let render_pass = render_target.create_render_pass(|framebuffer| {
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            // Our render pass has thus far been identical to the render pass in
            // `/examples/0_triangle`. However, our pipeline now does use resources, so we add
            // a `bind_resources` command to bind our bind group to the pipeline in bind group slot
            // `0`; we bind an empty bind group to slot `1`.
            //
            // Note that, as with the vertex array, WebGlitz wont have to do any additional runtime
            // safety checks here to ensure that the bind group is compatible with the pipeline: we
            // checked this when we created the pipeline and we can now once again leverage Rust's
            // type system to enforce safety at compile time.
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&vertex_buffer)
                .bind_resources((&bind_group_0, &BindGroup::empty()))
                .draw(3, 1)
                .finish()
        })
    });

    context.submit(render_pass);

    // We should now see our triangle on the canvas again, except this time it should be half the
    // size of the triangle we produced in `/examples/0_triangle`.
}
