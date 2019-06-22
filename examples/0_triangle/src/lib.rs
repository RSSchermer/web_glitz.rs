// The hello world of graphics programming: a single colored triangle.
//
// This example assumes some familiarity with graphics pipelines, vertex shaders and fragment
// shaders. However, if you've ever done any graphics programming with another graphics API
// (OpenGL/WebGl, Direct3D, Metal, Vulkan, ...), then this should hopefully make sense.

// For the time being, the `web_glitz::Vertex` derive macro requires that we enable this feature:
#![feature(const_fn)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::UsageHint;
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::vertex::VertexArrayDescriptor;

use web_sys::{window, HtmlCanvasElement};

// First we declare our vertex type and derive `web_glitz::Vertex`. In this example we'll store an
// array of 3 of these vertices in a GPU-accessible memory buffer. We'll then feed these as the
// input to a very simple graphics pipeline that will assemble them into a single triangle.
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
// There's still one additional thing we have to do: we have to make sure our type implements `Copy`
// if we want to be able to store it in a GPU-accessible buffer. This is because uploading data to
// and downloading data from a buffer involves doing a bitwise copy of the data. WebGlitz relies on
// the semantics associated with the `Copy` trait to make sure that this is safe. `Clone` is a
// supertrait of `Copy`, so we'll also have to implement `Clone`. Fortunately this is pretty easy:
// we can automatically derive both `Clone` and `Copy`.
#[derive(web_glitz::Vertex, Clone, Copy)]
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
    //   this context remains alive.
    //
    // We'll use the default `ContextOptions` for this example (see
    // `web_glitz::runtime::ContextOptions` for details on the configurable options).
    //
    // This returns 2 things:
    //
    // - a context object, which we'll use to interact with the GPU, and;
    // - a handle to the "default" render target, which is the render target the browser will
    //   display on the canvas element.
    let (context, render_target) =
        unsafe { single_threaded::init(&canvas, &ContextOptions::default()).unwrap() };

    // Create and compile our vertex shader using the GLSL code in `/src/vertex.glsl`.
    let vertex_shader = context
        .create_vertex_shader(include_str!("vertex.glsl"))
        .unwrap();

    // Create and compile our fragment shader using the GLSL code in `/src/fragment.glsl`.
    let fragment_shader = context
        .create_fragment_shader(include_str!("fragment.glsl"))
        .unwrap();

    // Create our graphics pipeline. We'll use the vertex and fragment shaders we just initialized
    // and we'll assemble our vertices into triangles. We also have to specify a type for the vertex
    // input layout we intend to use with this pipeline. When the pipeline is created, WebGlitz will
    // reflect on the shader code to verify that this vertex input layout is indeed compatible with
    // the pipeline, otherwise this will return an error. We'll use our `Vertex` as the vertex input
    // layout.
    let pipeline = context
        .create_graphics_pipeline(
            &GraphicsPipelineDescriptor::begin()
                .vertex_shader(&vertex_shader)
                .primitive_assembly(PrimitiveAssembly::Triangles {
                    // Because we're using a triangle topology, we have to specify a `WindingOrder`.
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
                .vertex_input_layout::<Vertex>()
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

    // Initialize a vertex array using our vertex buffer. We'll tell WebGlitz we wont be using an
    // index buffer with this vertex array by passing an empty tuple `()`.
    let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
        vertex_input_state: &vertex_buffer,
        indices: (),
    });

    // Create a render pass for our default render target.
    //
    // Here's the simplified conceptual explanation of what will happen in our render pass:
    //
    // When this render pass gets executed, the images associated with the render target (a.k.a. the
    // "attached images" or the "attachments", in this case an RGBA color image that is displayed on
    // the canvas) will be loaded into "framebuffer memory". The second argument is a function that
    // takes this framebuffer as an argument and returns a render pass task that may then modify the
    // contents of the framebuffer. Our render pass will tell the driver to run our `pipeline` once
    // on our `vertex_array` so that we'll "draw" our triangle to the framebuffer. When the task is
    // done, the contents of the framebuffer will be stored back into the render target image(s). In
    // this case, that should result in our triangle showing up on the canvas.
    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        // Return the render pass task. Our render pass task will consist of a pipeline task that
        // uses the pipeline we initialized earlier. The second argument is a function that takes
        // the activated pipeline and returns a pipeline task.
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            // Return a pipeline task. In this case the pipeline task is a single draw command:
            // we'll tell the GPU driver to run our pipeline once, using our `vertex_array` as
            // input.
            //
            // The second argument specifies the additional resources ("uniforms") we'll give the
            // pipeline access to. However, our simple pipeline does not use an resources, so we'll
            // pass an empty tuple `()`.
            //
            // Note that WebGlitz wont have to do additional runtime safety checks here to ensure
            // that the `vertex_array` is compatible with the pipeline: we'll just leverage Rust's
            // type system to verify at compile time that the `vertex_array` type is compatible with
            // the `vertex_input_layout` we declared when we created the pipeline.
            active_pipeline.draw_command(&vertex_array, &())
        })
    });

    // Finally, submit our render pass task to the context's command queue where it will be
    // executed.
    context.submit(render_pass);

    // We should be seeing our first triangle on the canvas now!
}
