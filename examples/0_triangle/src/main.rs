extern crate web_glitz;

#[derive(Vertex)]
struct Vertex {
    #[vertex_attribute(location = 0)]
    position: [f32; 2],
    #[vertex_attribute(location = 1)]
    color: [f32; 3]
}

fn main() {
    let canvas: CanvasElement = document().query_selector("#canvas").unwrap().unwrap().try_into().unwrap();
    let context: WebDraw2Context = canvas.get_context().expect("WebDraw2 not supported");

    let vertex_shader = context.create_shader(include_str!("vertex.glsl"));
    let fragment_shader = context.create_shader(include_str!("fragment.glsl"));

    let pipeline_descriptor = PipelineDescriptor::begin()
        .vertex_shader(vertex_shader)
        .primitive_assembly(Topology::Triangles)
        .fragment_shader(fragment_shader)
        .finish()
        .unwrap();

    let pipeline = context.create_pipeline(pipeline_descriptor);

    let vertex_data = [
        Vertex { position: [0.0, 0.5], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, -0.5], color: [0.0, 0.0, 1.0] },
    ];

    let vertex_buffer = context.create_array_buffer(3, BufferUsage::StaticDraw);
    let upload_vertex_data_task = vertex_buffer.upload_task(vertex_data);
    let vertex_array = context.create_vertex_array(vertex_data);

    let render_task = context.default_framebuffer().task(|framebuffer| {
        render_pass([DrawBuffer::Back], sequence_all![
            framebuffer.clear_color_task([0.0, 0.0, 0.0, 0.0]),
            pipeline.draw_task(vertex_array, Uniforms::empty())
        ])
    });

    context.submit(sequence_all![upload_vertex_data_task, render_task]);
}
