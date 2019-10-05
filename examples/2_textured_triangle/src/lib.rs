// This example shows how to use a custom render target to render to a texture.
//
// This example builds on `/examples/0_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn, const_raw_ptr_to_usize_cast, raw_address_of)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};

use web_glitz::buffer::UsageHint;
use web_glitz::image::format::RGBA8;
use web_glitz::image::texture_2d::{FloatSampledTexture2D, Texture2DDescriptor};
use web_glitz::image::{Image2DSource, MipmapLevels};
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, WindingOrder,
};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::sampler::{MagnificationFilter, MinificationFilter, SamplerDescriptor};
use web_glitz::task::sequence_all;

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float2_f32")]
    texture_coordinates: [f32; 2],
}

// Define a `Resources` type and derive `web_glitz::derive::Resources`.
//
// We'll use this type to create a "bind group": a group of resources that may be bound to a GPU
// pipeline, such that each of the pipeline's invocations may access these resource when the
// pipeline is executing. In this case, our bind group will consist of only a single resource: a
// float-sampled 2D texture.
#[derive(web_glitz::derive::Resources)]
struct Resources<'a> {
    // We'll have to mark the field that will hold our texture resource with a `#[resource(...)]`
    // attribute. We can only use this attribute on fields that implement
    // `web_glitz::pipeline::resources::Resource`, otherwise our struct will fail to compile.
    // We have to specify a positive integer for the `binding` index to which we'll bind the
    // resource, in this case we'll use `0`. If we were to use multiple resources, then we'd have to
    // make sure that each resource is bound to a unique `binding` index, otherwise our struct would
    // fail to compile (we can't bind more than one texture resource to the same texture resource
    // binding).
    //
    // If the name of our field does not exactly match (case-sensitive) the name of the sampler
    // uniform we want to bind our texture to, then we must also specify a `name`; in this case
    // there is not an exact match, so we explicitly specify the `name` of the sampler uniform we
    // want to bind to as "diffuse_texture".
    #[resource(binding = 0, name = "diffuse_texture")]
    texture: FloatSampledTexture2D<'a>,
}

#[wasm_bindgen(start)]
pub fn start() {
    let document = window().unwrap().document().unwrap();

    let canvas: HtmlCanvasElement = document
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
    // Our pipeline is very similar to the pipeline we used in `/examples/0_triangle`, except this
    // time our vertex shader uses a float-sampled 2D texture, so we must declare a resource layout.
    // The resource layout must match the resource layout used in our shader code. Note that GLSL ES
    // 3.0 does not allow us to specify explicit bind groups for resources in the shader code.
    // Instead, WebGL 2.0 defines 2 implicit bind groups: 1 bind group for all uniform buffers
    // (bind group `0`) and 1 bind group for all texture-samplers (bind group `1`). We don't use any
    // uniform buffers in this example, so we'll use the empty tuple `()` to declare an empty layout
    // for bind group `0`; we'll use our `Resources` type to specify the layout for bind group `1`.
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
                .typed_resource_bindings_layout::<((), Resources)>()
                .finish(),
        )
        .unwrap();

    let vertex_data = [
        Vertex {
            position: [0.0, 0.5],
            texture_coordinates: [0.5, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5],
            texture_coordinates: [0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5],
            texture_coordinates: [1.0, 1.0],
        },
    ];

    let vertex_buffer = context.create_buffer(vertex_data, UsageHint::StreamDraw);

    // Create a new 2D texture that uses the RGBA8 storage format. This texture will start out with
    // its data set to all zeroes (which with an RGBA8 format essentially corresponds to
    // "transparent black"). Note that we only allocate 1 mipmap level (the "base" level) as we
    // won't make use of mipmapping in this example.
    let texture = context
        .create_texture_2d(&Texture2DDescriptor {
            format: RGBA8,
            width: 256,
            height: 256,
            levels: MipmapLevels::Partial(1),
        })
        .unwrap();

    // Create a new sampler.
    let sampler = context.create_sampler(&SamplerDescriptor {
        minification_filter: MinificationFilter::Linear,
        magnification_filter: MagnificationFilter::Linear,
        ..Default::default()
    });

    let image_element = document
        .get_element_by_id("checkerboard_color_gradient")
        .unwrap()
        .dyn_into()
        .unwrap();

    // Create an image source from an HTML image element.
    let image_src = Image2DSource::from_image_element(&image_element);

    // Create a command that will upload our image source to the texture's base level.
    let upload_command = texture.base_level().upload_command(image_src);

    // Create an empty bind group to match the uniform buffer bind group expected by the pipeline.
    let bind_group_0 = context.create_bind_group(());

    // Create a bind group for our resources.
    let bind_group_1 = context.create_bind_group(Resources {
        texture: texture.float_sampled(&sampler).unwrap(),
    });

    let render_pass = context.create_render_pass(render_target, |framebuffer| {
        framebuffer.pipeline_task(&pipeline, |active_pipeline| {
            // Our render pass has thus far been identical to the render pass in
            // `/examples/0_triangle`. However, our pipeline now does use resources, so we add
            // a `bind_resources` command that binds our bind group to the pipeline in bind group
            // slot `1`.
            //
            // Note that, as with the vertex array, WebGlitz wont have to do any additional runtime
            // safety checks here to ensure that the resources are compatible with the pipeline: we
            // checked this when we created the pipeline and we can now leverage Rust's type system
            // again to enforce safety at compile time.
            active_pipeline
                .task_builder()
                .bind_vertex_buffers(&vertex_buffer)
                .bind_resources((&bind_group_0, &bind_group_1))
                .draw(3, 1)
                .finish()
        })
    });

    // `/examples/0_triangle` only had to submit a render pass, but now we must also submit the
    // image upload command. It's important that the upload finished before we begin the render pass
    // so we'll use the `sequence_all!` macro to combine them into a sequenced task, which
    // guarantees that the tasks are executed in order: the render pass may only begin after the
    // upload has completed.
    context.submit(sequence_all![upload_command, render_pass]);

    // We should now see our triangle on the canvas again, except this time it should be covered by
    // the checkerboard color gradient in our texture image.
}
