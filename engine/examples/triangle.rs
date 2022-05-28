use std::sync::Arc;

use bytemuck::Pod;
use bytemuck::Zeroable;
use log::trace;
use simple_logger::SimpleLogger;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::Subpass;

use quasar_engine::drawing::engine::Engine;

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position);

fn main() {
    SimpleLogger::new().init().unwrap();

    let engine = Engine::new();

    // Simple triangle
    trace!("Creating the triangle's vertices");
    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        Arc::clone(engine.hardware.graphics_device()),
        BufferUsage::vertex_buffer(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
        .unwrap();

    trace!("Creating the render pass");
    let render_pass = vulkano::single_pass_renderpass!(
        engine.hardware.graphics_device().clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: engine.screen.swapchain().image_format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
        .unwrap();

    trace!("Loading the shaders");
    let vs = vs::load(Arc::clone(engine.hardware.graphics_device())).unwrap();
    let fs = fs::load(Arc::clone(engine.hardware.graphics_device())).unwrap();

    trace!("Creating the graphics pipeline");
    let pipeline = GraphicsPipeline::start()
        // We need to indicate the layout of the vertices.
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        // See `vertex_shader`.
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        // We have to indicate which subpass of which render pass this pipeline is going to be used
        // in. The pipeline will only be usable from this particular subpass.
        .render_pass(Subpass::from(Arc::clone(&render_pass), 0).unwrap())
        // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
        .build(Arc::clone(engine.hardware.graphics_device()))
        .unwrap();

    engine.run(render_pass, move |hardware, _screen, frame, viewport| {
        let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

        let mut builder = AutoCommandBufferBuilder::primary(
            Arc::clone(hardware.graphics_device()),
            hardware.graphics_queue().family(),
            CommandBufferUsage::OneTimeSubmit,
        )
            .unwrap();

        builder
            .begin_render_pass(Arc::clone(frame), SubpassContents::Inline, clear_values)
            .unwrap()
            .set_viewport(0, [viewport.clone()])
            .bind_pipeline_graphics(pipeline.clone())
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .draw(vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        builder.build().unwrap()
    });
}

mod vs {
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

mod fs {
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
