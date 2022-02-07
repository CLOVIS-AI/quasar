use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::format::Format;
use vulkano::image::{ImageDimensions, StorageImage};
use vulkano::image::view::ImageView;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync;
use vulkano::sync::GpuFuture;

use quasar_engine::engine::Engine;

use crate::graphics::demo_graphics;

mod graphics;

fn main() {
    println!("\nStarting…");

    let engine = Engine::new();

    // region Copy data from one buffer to another
    println!("\nDemo: Copy data from one buffer to another.");

    // Source contents: 0, 1, 2, 3, … 63 (size 64)
    let source_content = 0..64;
    println!("Source contents:               {:?}", source_content.clone().collect::<Vec<i32>>());
    let source = CpuAccessibleBuffer::from_iter(engine.device.clone(), BufferUsage::all(), false, source_content)
        .expect("Failed to create source buffer");

    // Destination contents: 0, 0, 0, … 0 (size 64)
    let destination_content = (0..64).map(|_| 0);
    println!("Destination contents (before): {:?}", destination_content.clone().collect::<Vec<i32>>());
    let destination = CpuAccessibleBuffer::from_iter(engine.device.clone(), BufferUsage::all(), false, destination_content)
        .expect("Failed to create destination buffer");

    // Create a command buffer…
    let mut builder = AutoCommandBufferBuilder::primary(engine.device.clone(), engine.graphics_queue.clone().family(), CommandBufferUsage::MultipleSubmit)
        .expect("Could not create buffer builder.");
    // …that copies the buffers…
    builder.copy_buffer(source.clone(), destination.clone())
        .expect("Could not copy buffer.");
    // …and finally, get the actual buffer.
    let command_buffer = builder.build()
        .expect("Could not build buffer.");

    // We can now start it.
    let finished = command_buffer.execute(engine.graphics_queue.clone())
        .expect("Could not execute the command buffer.");
    // Wait until it is done…
    finished.then_signal_fence_and_flush().expect("Could not fence and flush the executed buffer.")
        .wait(None)
        .expect("Failure during waiting for the executed buffer.");

    // We can now read the destination buffer.
    let updated_destination = destination.read().expect("Could not read from the destination buffer");
    println!("Destination contents (after):  {:?}", &*updated_destination);
    // endregion

    // region Multiply an array by 12 in a single operation
    println!("\nDemo: Multiply an array of size 65536 by 12 in a single operation");

    let data = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(engine.device.clone(), BufferUsage::all(), false, data)
        .expect("Failed to create buffer.");

    // Load the shader defined below
    let shader = times_twelve::load(engine.device.clone())
        .expect("Failed to load shader module.");

    // Create a compute pipeline
    // This pipeline represents "the order of executing the shader"
    let compute_pipeline = ComputePipeline::new(
        engine.device.clone(),
        shader.entry_point("main").expect("Couldn't find entry point 'main' in shader"),
        &(),
        None,
        |_| {},
    ).expect("Failed to create compute pipeline.");

    // Represents the necessary data layout given as parameter 0 to the compute pipeline
    let layout: Arc<DescriptorSetLayout> = compute_pipeline.layout().descriptor_set_layouts().get(0)
        .expect("Couldn't find layout descriptor 0.")
        .clone();

    // Creates a descriptor set that describes how the data is fed to the compute pipeline
    let set = PersistentDescriptorSet::new(
        layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding
    ).expect("Could not create DescriptorSet.");

    let mut command_builder = AutoCommandBufferBuilder::primary(
        engine.device.clone(),
        engine.graphics_queue.clone().family(),
        CommandBufferUsage::OneTimeSubmit,
    ).expect("Could not create command builder.");

    command_builder.bind_pipeline_compute(compute_pipeline.clone());
    command_builder.bind_descriptor_sets(
        PipelineBindPoint::Compute,
        compute_pipeline.layout().clone(),
        0,
        set,
    );
    // The pipeline will be dispatched over 1024 work groups, in a single dimension
    command_builder.dispatch([1024, 1, 1]).expect("Could not dispatch the command builder");

    let command_buffer = command_builder.build().expect("Could not build the command builder");

    // Tell the GPU to start the pipeline
    let future = sync::now(engine.device.clone())
        .then_execute(engine.graphics_queue.clone(), command_buffer)
        .expect("Could not execute the command buffer")
        .then_signal_fence_and_flush()
        .expect("Could not fence and flush");

    future.wait(None).expect("The command buffer never finished");

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }
    println!("The product succeeded!");

    // endregion

    // region Fractal
    println!("\nDemo: Fractal");

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

    // Create a view of the image
    let view = ImageView::new(image.clone()).expect("Could not create the view of the image.");

    // Load the shader defined below
    let shader = fractal::load(engine.device.clone())
        .expect("Failed to load shader module.");

    // Create a compute pipeline
    // This pipeline represents "the order of executing the shader"
    let compute_pipeline = ComputePipeline::new(
        engine.device.clone(),
        shader.entry_point("main").expect("Couldn't find entry point 'main' in shader"),
        &(),
        None,
        |_| {},
    ).expect("Failed to create compute pipeline.");

    let layout = compute_pipeline
        .layout()
        .descriptor_set_layouts()
        .get(0)
        .expect("Could not get the layout of the compute pipeline");
    let set = PersistentDescriptorSet::new(
        layout.clone(),
        [WriteDescriptorSet::image_view(0, view.clone())], // 0 is the binding
    ).expect("Could not create the descriptor set");

    let destination = CpuAccessibleBuffer::from_iter(
        engine.device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    ).expect("Couldn't create destination buffer");

    let mut fractal_builder = AutoCommandBufferBuilder::primary(
        engine.device.clone(),
        engine.graphics_queue.family(),
        OneTimeSubmit,
    ).expect("Could not create the command buffer builder");

    fractal_builder.bind_pipeline_compute(compute_pipeline.clone());
    fractal_builder.bind_descriptor_sets(
        PipelineBindPoint::Compute,
        compute_pipeline.layout().clone(),
        0,
        set,
    );
    fractal_builder.dispatch([1024 / 8, 1024 / 8, 1]).expect("Could not dispatch the work groups");
    fractal_builder.copy_image_to_buffer(image.clone(), destination.clone()).expect("Could not request the copy");

    let command_buffer = fractal_builder.build().expect("Could not create the image clearing command buffer.");

    println!("Sending orders to the GPU…");
    let future = sync::now(engine.device.clone())
        .then_execute(engine.graphics_queue.clone(), command_buffer)
        .expect("Could not execute the command buffer")
        .then_signal_fence_and_flush()
        .expect("Could not flush the command.");

    println!("Waiting for the GPU to finish working…");
    future.wait(None).expect("Could not create the image.");

    println!("Saving the results…");
    let buffer_content = destination.read().expect("Could not read from the destination buffer");
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).expect("Could not create raw image");
    image.save("image.png").expect("Could not save image");
    println!("Created file 'image.png'.");

    // endregion

    demo_graphics(&engine);
}

mod times_twelve {
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
            #version 450

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Data {
                uint data[];
            } buf;

            void main() {
                uint idx = gl_GlobalInvocationID.x;
                buf.data[idx] *= 12;
            }
        "
    }
}

mod fractal {
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
            #version 450

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

            layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

            void main() {
                vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
	            vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

	            vec2 z = vec2(0.0, 0.0);
	            float i;
	            for (i = 0.0; i < 1.0; i += 0.005) {
		            z = vec2(
		                z.x * z.x - z.y * z.y + c.x,
		                z.y * z.x + z.x * z.y + c.y
		            );

		            if (length(z) > 4.0) {
			            break;
		            }
	            }

	            vec4 to_write = vec4(vec3(i), 1.0);
	            imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
            }
        "
    }
}
