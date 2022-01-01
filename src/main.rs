use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::sync::GpuFuture;

use crate::engine::Engine;

mod engine;

fn main() {
    println!("\nStarting…");

    let engine = Engine::new();

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
    let mut builder = AutoCommandBufferBuilder::primary(engine.device.clone(), engine.graphics_queue.family(), CommandBufferUsage::MultipleSubmit)
        .expect("Could not create buffer builder.");
    // …that copies the buffers…
    builder.copy_buffer(source.clone(), destination.clone())
        .expect("Could not copy buffer.");
    // …and finally, get the actual buffer.
    let command_buffer = builder.build()
        .expect("Could not build buffer.");

    // We can now start it.
    let finished = command_buffer.execute(engine.graphics_queue)
        .expect("Could not execute the command buffer.");
    // Wait until it is done…
    finished.then_signal_fence_and_flush().expect("Could not fence and flush the executed buffer.")
        .wait(None)
        .expect("Failure during waiting for the executed buffer.");

    // We can now read the destination buffer.
    let updated_destination = destination.read().expect("Could not read from the destination buffer");
    println!("Destination contents (after):  {:?}", &*updated_destination);
}
