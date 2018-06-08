extern crate stdweb;
extern crate web_draw_2;

use stdweb::web::document;
use stdweb::web::html_element::CanvasElement;
use web_draw_2::core::{
    RenderingContext as WebDraw2Context,
    Shader,
    Buffer
};

#[derive(VertexDefinition)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3]
}

fn main() {
    let canvas: CanvasElement = document().query_selector("#canvas").unwrap().unwrap().try_into().unwrap();
    let context: WebDraw2Context = canvas.get_context().expect("WebDraw2 not supported");

    let vertex_shader = Shader::compile(&context, include_str!("vertex.glsl")).unwrap();
    let fragment_shader = Shader::compile(&context, include_str!("fragment.glsl")).unwrap();

    let pipeline = Pipeline::start()
        .vertex_shader(vertex_shader)
        .primitive_assembly(Topology::Triangles)
        .fragment_shader(fragment_shader)
        .build().unwrap();

    let vertex_data = Buffer::upload(&context, Usage::StaticDraw, [
        Vertex { position: [0.0, 0.5], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, -0.5], color: [0.0, 0.0, 1.0] },
    ]);

    pipeline.bind_target(&context.default_framebuffer(), Output::All).unwrap()
        .draw(&vertex_data, Uniforms::empty());
}
