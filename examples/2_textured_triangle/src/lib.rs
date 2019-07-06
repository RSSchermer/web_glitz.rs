// This example shows how to bind a texture resource to a pipeline.
//
// This example builds on `/examples/0_triangle`, the comments in this example will focus on the
// differences/additions.

#![feature(const_fn)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_glitz::buffer::{Buffer, UsageHint};
use web_glitz::image::format::RGBA8;
use web_glitz::image::texture_2d::{FloatSampledTexture2D, Texture2DDescriptor};
use web_glitz::pipeline::graphics::{
    CullingMode, GraphicsPipelineDescriptor, PrimitiveAssembly, SlotBindingStrategy, WindingOrder,
};
use web_glitz::runtime::{single_threaded, ContextOptions, RenderingContext};
use web_glitz::std140;
use web_glitz::std140::repr_std140;
use web_glitz::task::sequence_all;
use web_glitz::vertex::VertexArrayDescriptor;

use std::mem;
use web_glitz::image::{Image2DSource, MipmapLevels};
use web_glitz::sampler::{MagnificationFilter, MinificationFilter, SamplerDescriptor};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

#[derive(web_glitz::derive::Vertex, Clone, Copy)]
struct Vertex {
    #[vertex_attribute(location = 0, format = "Float2_f32")]
    position: [f32; 2],
    #[vertex_attribute(location = 1, format = "Float2_f32")]
    texture_coordinates: [f32; 2],
}

// Define our resources type and derive `web_glitz::derive::Resources`.
//
// We'll provide an instance of this type when we invoke our pipeline to supply it with the
// resources our pipeline needs access to. In this case we'll need only one resource: a 2D texture
// resource that we'll sample for floating point color values.
#[derive(web_glitz::derive::Resources)]
struct Resources<'a> {
    // We'll have to mark the field that will hold our texture resource with a
    // `#[texture_resource(...)]` attribute. We can only use this attribute on fields that implement
    // `web_glitz::pipeline::resources::TextureResource`, otherwise our struct will fail to compile.
    // We have to specify a positive integer for the `binding` index to which we'll bind the
    // resource, in this case we'll use `0`. If we were to use multiple texture resources, then
    // we'd have to make sure that each texture resource is bound to a unique `binding` index,
    // otherwise our struct would fail to compile (we can't bind more than one texture resource to
    // the same texture resource binding). Here we use only one texture resource though, so
    // we don't have to worry about that.
    //
    // If the name of our field does not exactly match (case-sensitive) the name of the sampler
    // uniform we want to bind our texture to, then we must also specify a `name`; in this case
    // there is not an exact match, so we explicitly specify the `name` of the sampler uniform we
    // want to bind to as "diffuse_texture".
    #[texture_resource(binding = 0, name = "diffuse_texture")]
    texture: FloatSampledTexture2D<'a>,
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

    // Create our pipeline.
    //
    // Our pipeline is very similar to the pipeline we used in `/examples/0_triangle`, except this
    // time our fragment shader uses a texture sampler, so we must declare a `resource_layout` type
    // that will provide a texture resource to back it. We'll specify the `Resources` type we
    // defined above.
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

    let vertex_array = context.create_vertex_array(&VertexArrayDescriptor {
        vertex_input_state: &vertex_buffer,
        indices: (),
    });

    // Create a new 2D texture.
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

    let image_src =
        Image2DSource::from_pixels(get_image_data("checkerboard_color_gradient"), 256, 256)
            .unwrap();

    let upload_command = texture.base_level().upload_command(image_src);

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
                    texture: texture.float_sampled(&sampler).unwrap(),
                },
            )
        })
    });

    context.submit(sequence_all![upload_command, render_pass]);

    // We should now see our triangle on the canvas again, except this time it should be covered by
    // the checkerboard color gradient in our texture image.
}

fn get_image_data(id: &str) -> Vec<[u8; 4]> {
    let document = window().unwrap().document().unwrap();

    let image: HtmlImageElement = document.get_element_by_id(id).unwrap().dyn_into().unwrap();

    let width = image.natural_width();
    let height = image.natural_height();

    let canvas = document
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    context
        .draw_image_with_html_image_element(&image, 0.0, 0.0)
        .unwrap();

    let mut image_data = context
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .unwrap()
        .data();

    let len = image_data.len();
    let capacity = image_data.capacity();
    let ptr = image_data.as_mut_ptr();

    mem::forget(image_data);

    unsafe { Vec::from_raw_parts(mem::transmute(ptr), len / 4, capacity / 4) }
}