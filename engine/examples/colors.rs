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

    // This example uses a single fragment shader responsible for rendering the whole screen
    // Because fragment shaders are only called on visible vertices, we need to force the rasterizer to consider
    // the whole screen as a single object.

    // For this, we use a trick from https://stackoverflow.com/a/59739538: we draw a single triangle that covers the
    // whole screen, so a single fragment shader is called for everything displayed.
    let vertex1 = Vertex {
        position: [-1.0, -1.0],
    };
    let vertex2 = Vertex {
        position: [-1.0, 4.0],
    };
    let vertex3 = Vertex {
        position: [4.0, -1.0],
    };

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        Arc::clone(engine.hardware.graphics_device()),
        BufferUsage::vertex_buffer(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    ).unwrap();

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
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(Arc::clone(&render_pass), 0).unwrap())
        .build(Arc::clone(engine.hardware.graphics_device()))
        .unwrap();

    engine.run(render_pass, move |hardware, _screen, frame, viewport| {
        let clear_values = vec![[0.0, 0.0, 0.0, 0.0].into()];

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
layout(location = 0) out vec2 fragPosition;

// Correct the positions of the three vertices
// The corrected coordinates are passed to the fragment shader, which will handle drawing
vec2 positions[3] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 2.5),
    vec2(2.5, 0.0)
);

void main() {
	gl_Position = vec4(position, 0.0, 1.0);
	fragPosition = positions[gl_VertexIndex];
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "#version 450
layout(location = 0) out vec4 f_color;

// The screenspace position of the pixel currently being shaded
// The coordinates follow these rules:
//  - (0, 0): top-left of the screen
//  - (0, 1): top-right of the screen
//  - (1, 0): bottom-left of the screen
layout(location = 0) in vec2 position;

void main() {
    vec3 color = vec3(position.xy, 0.25);

	f_color = vec4(color, 1.0);
}
        "
    }
}
