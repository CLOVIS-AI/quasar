use image::{ImageBuffer, Rgba};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::format::Format;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, Subpass};
use vulkano::sync;
use vulkano::sync::GpuFuture;

use quasar_engine::engine::Engine;

#[derive(Default, Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

pub fn demo_graphics(engine: &Engine) {
    println!("\nDemo: Graphics pipeline");

    let vertex1 = Vertex { position: [-0.5, -0.5] };
    let vertex2 = Vertex { position: [0.0, 0.5] };
    let vertex3 = Vertex { position: [0.5, -0.25] };

    println!("Loading the shaders…");
    let vertex_shader = vertex_shader::load(engine.device.clone()).expect("Couldn't load the vertex shader");
    let fragment_shader = fragment_shader::load(engine.device.clone()).expect("Couldn't load the fragment shader");

    println!("Creating the graphics pipeline…");
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        engine.device.clone(),
        BufferUsage::all(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    ).expect("Couldn't create a vertex buffer");

    let render_pass = vulkano::single_pass_renderpass!(
        engine.device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).expect("Could not create render pass");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1024.0, 1024.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vertex_shader.entry_point("main").expect("Couldn't find the vertex shader's main"), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fragment_shader.entry_point("main").expect("Couldn't find the fragment shader's main"), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).expect("Couldn't create the render sub pass"))
        .build(engine.device.clone())
        .expect("Couldn't build the graphics pipeline");

    //noinspection DuplicatedCode
    let image = StorageImage::new(
        engine.device.clone(),
        ImageDimensions::Dim2d {
            width: 1024,
            height: 1024,
            array_layers: 1,
        },
        Format::R8G8B8A8_UNORM,
        Some(engine.graphics_queue.family()),
    ).expect("Could not create storage image.");

    let destination = CpuAccessibleBuffer::from_iter(
        engine.device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    ).expect("Couldn't create destination buffer");

    let view = ImageView::new(image.clone()).expect("Could not create the image view");

    let framebuffer = Framebuffer::start(render_pass.clone())
        .add(view).expect("Couldn't add the view to the frame buffer")
        .build().expect("Couldn't build the frame buffer");

    let mut command_builder = AutoCommandBufferBuilder::primary(
        engine.device.clone(),
        engine.graphics_queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    ).expect("Could not create command builder");

    command_builder
        .begin_render_pass(
            framebuffer.clone(),
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        ).expect("Could not request to begin the render pass")

        .bind_pipeline_graphics(pipeline.clone())
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .draw(3, 1, 0, 0)
        .expect("Couldn't create the draw command")

        .end_render_pass()
        .expect("Could not request to end the render pass")

        .copy_image_to_buffer(image, destination.clone())
        .expect("Couldn't request the copy to the destination buffer");

    let command_buffer = command_builder
        .build().expect("Could not build the command buffer");

    println!("Sending orders to the GPU…");
    let future = sync::now(engine.device.clone())
        .then_execute(engine.graphics_queue.clone(), command_buffer)
        .expect("Couldn't request the execution of the command buffer")

        .then_signal_fence_and_flush()
        .expect("Couldn't request the fence and flush");

    println!("Waiting for the GPU to finish working…");
    future.wait(None).expect("The command buffer failed");

    println!("Saving the results…");
    let buffer_content = destination.read().expect("Could not read from the destination buffer");
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).expect("Could not create raw image");
    image.save("triangle.png").expect("Could not save image");
    println!("Created file 'triangle.png'.");
}

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "
    }
}
