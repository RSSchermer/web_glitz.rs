// The hello world of graphics programming: a single colored triangle.
//
// This example assumes some familiarity with graphics pipelines, vertex shaders and fragment
// shaders. However, if you've ever done any graphics programming with another graphics API
// (OpenGL/WebGl, Direct3D, Metal, Vulkan, ...), then this will hopefully look familiar.

// For the time being, the `web_glitz::derive::Vertex` derive macro requires that we enable some
// nightly features:
#![feature(ptr_offset_from, const_fn, const_ptr_offset_from, const_raw_ptr_deref)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::UsageHint;
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::pipeline::resources::BindGroup;
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};

use web_sys::{window, HtmlCanvasElement};

// First we declare a vertex type and derive `web_glitz::derive::Vertex`. In this example we'll
// store an array of 3 of these vertices in a GPU-accessible memory buffer. We'll then feed these as
// the input to a very simple graphics pipeline that will assemble them into a single triangle.
//
// We have to mark the fields that we intend to read in our pipeline's vertex shader stage. In this
// case our pipeline expects 2 input attributes: a `vec2` `position` attribute and a `vec3` `color`
// attribute. Attribute fields are marked with `#[vertex_attribute(...)]`, other fields will be
// ignored (here all fields are attribute fields). A vertex attribute has to declare a `location`
// and a `format`. The `location` must match the location of the attribute you intend to bind to in
// the graphics pipeline (as declared with `layout(location=...)` in the vertex shader code, see
// `src/vertex.glsl`). The `format` must be the name of one of the format types declared in
// `web_glitz::pipeline::graphics::vertex_input::attribute_format` and the format must correspond to
// the data type the pipeline expects for the attribute; this correspondence will be verified when
// the pipeline is created.
//
// We also have to make sure our type implements `Copy` (and it's super-trait `Clone`) if we want to
// be able to store it in a GPU-accessible buffer. This is because uploading data to and downloading
// data from a buffer involves doing a bitwise copy of the data. WebGlitz relies on the semantics
// associated with the `Copy` trait to ensure that this is safe.
#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    // We intend to bind this field to the `position` attribute in the vertex shader, which is of
    // type `vec2`. We therefor need to match the `location` we've assigned to the attribute in the
    // vertex shader (`0`) and use a `format` that is compatible with a `vec2`. A `vec2` requires 2
    // floating point values, so we need to use one of the `Float2_...` format types (see
    // `web_glitz::pipeline::graphics::vertex_input::attribute_format` for all formats). In this
    // case we'll pick `Float2_f32`. We then have to use a field type that implements
    // `FormatCompatible<Float2_f32>`. In this case we'll use `[f32; 2]` (the only type for which
    // WebGlitz implements `FormatCompatible<Float2_f32>` out of the box, but you might for example
    // implement it for your own "Vector2" type).
    //
    // Note that while the name of the rust field and the name of the vertex shader input are the
    // same in this example, this is not strictly necessary: only the `location` is used to match
    // the field to the input attribute.
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],

    // This field we intend to bind to the `color` attribute in the vertex shader, which is of
    // type `vec3`. The process is much the same as what we did for the `position`, however, in
    // this case we'll need a `Float3_...` attribute format. We've decided that 8 bits are quite
    // enough for our color values, so rather than using `Float3_f32`, we'll use `Float3_u8_norm`,
    // which takes `u8` values and "normalizes" the value range `0..=255` onto `0.0..=1.0`, thus
    // converting the `u8`s into the floating point values the pipeline expects.
    #[vertex_attribute(location = 1, format = "Float3_u8_norm")]
    color: [u8; 3],
}

#[wasm_bindgen(start)]
pub fn start() {
    // Obtain a reference to the `web_sys::HtmlCanvasElement` we want to render to.
    let canvas: HtmlCanvasElement = window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    // Initialize a single threaded WebGlitz context for the canvas. This is unsafe: WebGlitz tracks
    // various bits of state for the context. There is currently no way to prevent you from simply
    // obtaining another copy of the WebGL 2.0 context and modifying the state while bypassing
    // WebGlitz's state tracking. Therefore, obtaining a WebGlitz context is only safe if:
    //
    // - the canvas context is in its default state (you've not previously obtained another context
    //   and modified the state), and;
    // - you will not modify the context state through another copy of the context for as long as
    //   the WebGlitz context remains alive.
    //
    // We'll use the default `ContextOptions` for this example (see
    // `web_glitz::runtime::ContextOptions` for details on the configurable options).
    //
    // This returns 2 things:
    //
    // - a context object, which we'll use to interact with the GPU, and;
    // - a handle to the "default" render target, which is the render target the browser will
    //   display on the canvas element.
    let (context, mut render_target) =
        unsafe { single_threaded::init(&canvas, &ContextOptions::default()).unwrap() };

    // Create and compile a vertex shader using the GLSL code in `/src/vertex.glsl`.
    let vertex_shader = context
        .try_create_vertex_shader(include_str!("vertex.glsl"))
        .unwrap();

    // Create and compile a fragment shader using the GLSL code in `/src/fragment.glsl`.
    let fragment_shader = context
        .try_create_fragment_shader(include_str!("fragment.glsl"))
        .unwrap();

    // Create a graphics pipeline. We'll use the vertex and fragment shaders we just initialized
    // and we'll assemble our vertices into triangles. We also specify a type for the vertex input
    // layout we intend to use with this pipeline. When the pipeline is created, WebGlitz will
    // reflect on the shader code to verify that this vertex input layout is indeed compatible with
    // the pipeline, otherwise this will return an error. We'll use our `Vertex` type to describe
    // the vertex input layout.
    let pipeline = context
        .try_create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    // Because we're using triangle topology, we have to specify a `WindingOrder`.
                    // The winder order determines which side of a triangle its "front" side and
                    // which side is its "back" side, see
                    // `web_glitz::pipeline::graphics::WindingOrder` for details.
                    winding_order: WindingOrder::CounterClockwise,

                    // We also have to specify a `CullingMode`, see
                    // `web_glitz::pipeline::graphics::CullingMode` for details. We're only
                    // rendering one triangle and we don't want to cull it, so we'll use
                    // `CullingMode::None`.
                    face_culling: CullingMode::None,
                })
                .fragment_shader(&fragment_shader)
                .typed_vertex_attribute_layout::<Vertex>()
                .typed_resource_bindings_layout::<((), ())>()
                .finish(),
        )
        .unwrap();

    // Create an array with the data for the 3 vertices we'll use to draw our triangle.
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

    // Create a GPU-accessible memory buffer that holds our vertex data. We intend to use this
    // buffer for drawing once and we don't intend to update its contents after this, so we'll tell
    // the driver that we intend to use it as `BufferUsage::StreamDraw` (see
    // `web_glitz::buffer::BufferUsage` for details on buffer usage hints).
    let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StreamDraw);

    // Create a render pass for our default render target.
    //
    // When this render pass gets executed, the images associated with the render target (a.k.a. the
    // "attached images" or the "attachments"; in this case an RGBA color image that is displayed on
    // the canvas) will be loaded into "framebuffer memory". The second argument is a function that
    // takes this framebuffer as an argument and returns a render pass task that may modify the
    // contents of the framebuffer. Our render pass will tell the driver to run our `pipeline` once
    // on our `vertex_buffer` so that we'll "draw" our triangle to the framebuffer. When the task is
    // done, the contents of the framebuffer will be stored back into the render target image(s).
    // Since we're using the default render target, that should result in our triangle showing up on
    // the canvas.
    let render_pass = render_target.create_render_pass(|framebuffer| {
        // Return the render pass task. Our render pass task will consist of a pipeline task that
        // uses the pipeline we initialized earlier. The second argument is a function that takes
        // the activated pipeline and returns a pipeline task.
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            // This function needs to return a pipeline task. We want to invoke the pipeline with a
            // "draw" command while our vertex data is bound to it. We'll use the pipeline task
            // builder interface to construct our task safely without runtime checks: the task
            // builder interface leverages Rust's type system to ensure at compile time that
            // matching data is bound to the pipeline, before allowing us to encode the draw
            // command.
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&vertex_buffer)
                .bind_resources((&BindGroup::empty(), &BindGroup::empty()))
                .draw(3, 1)
                .finish()
        })
    });

    // Finally, submit our render pass task to the context's command queue where it will be
    // executed.
    context.submit(render_pass);

    // We should see our first triangle on the canvas now!
}
