use std::sync::Arc;

use log::{debug, warn};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::image::{ImageAccess, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{acquire_next_image, AcquireError, SwapchainCreationError};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::drawing::hardware::Hardware;
use crate::drawing::screen::Screen;

pub struct Engine {
    event_loop: EventLoop<()>,
    pub hardware: Arc<Hardware>,
    pub screen: Arc<Screen>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new() -> Engine {
        let event_loop = EventLoop::new();
        let hardware = Arc::new(Hardware::new(&event_loop));
        let screen = Arc::new(Screen::new(Arc::clone(&hardware), &event_loop));

        debug!("Vulkan initialization finished.");
        Engine {
            event_loop,
            hardware,
            screen,
        }
    }

    pub fn run<D>(mut self, render_pass: Arc<RenderPass>, draw: D)
        where
            D: Fn(&Hardware, &Screen, &Arc<Framebuffer>, &Viewport) -> PrimaryAutoCommandBuffer
            + 'static,
    {
        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let mut framebuffers = window_size_dependent_setup(
            self.screen.images(),
            Arc::clone(&render_pass),
            &mut viewport,
        );

        let mut recreate_swapchain = false;

        let mut previous_frame_end =
            Some(sync::now(Arc::clone(self.hardware.graphics_device())).boxed());

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    recreate_swapchain = true;
                }
                Event::RedrawEventsCleared => {
                    // Clean stuff reserved by the GPU
                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    //region Recreate the swapchain if necessary
                    if recreate_swapchain {
                        let new_screen = self.screen.recreate();
                        let new_screen = match new_screen {
                            Ok(r) => r,
                            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                            Err(e) => panic!("Couldn't recreate the swapchain: {:?}", e),
                        };
                        self.screen = Arc::new(new_screen);

                        framebuffers = window_size_dependent_setup(
                            self.screen.images(),
                            render_pass.clone(),
                            &mut viewport,
                        );
                        recreate_swapchain = false;
                    }
                    //endregion

                    let (image_num, suboptimal, acquire_future) =
                        match acquire_next_image(Arc::clone(self.screen.swapchain()), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                recreate_swapchain = true;
                                return;
                            }
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    let command_buffer = draw(
                        &self.hardware,
                        &self.screen,
                        &framebuffers[image_num],
                        &viewport,
                    );

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(Arc::clone(self.hardware.graphics_queue()), command_buffer)
                        .unwrap()
                        .then_swapchain_present(
                            Arc::clone(self.hardware.graphics_queue()),
                            Arc::clone(self.screen.swapchain()),
                            image_num,
                        )
                        .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => {
                            previous_frame_end = Some(future.boxed());
                        }
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_end = Some(
                                sync::now(Arc::clone(self.hardware.graphics_device())).boxed(),
                            );
                        }
                        Err(e) => {
                            warn!("Failed to flush future: {:?}", e);
                            previous_frame_end = Some(
                                sync::now(Arc::clone(self.hardware.graphics_device())).boxed(),
                            );
                        }
                    }
                }
                _ => (),
            }
        });
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
                .unwrap()
        })
        .collect::<Vec<_>>()
}
