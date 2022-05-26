use std::sync::Arc;

use log::{debug, trace};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::swapchain::{Swapchain, SwapchainCreateInfo};
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::drawing::hardware::Hardware;

pub struct Screen {
    hardware: Arc<Hardware>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
}

impl Screen {
    pub fn new(hardware: Arc<Hardware>, event_loop: &EventLoop<()>) -> Self {
        debug!("Creating a painter…");

        trace!("Creating the swap-chain…");
        let (mut swapchain, images) = {
            let capabilities = hardware.graphics_device().physical_device()
                .surface_capabilities(hardware.surface(), Default::default())
                .expect("Could not query the surface capabilities");

            let format = hardware.graphics_device().physical_device()
                .surface_formats(hardware.surface(), Default::default())
                .expect("Could not select any format capabilities")
                [0].0;

            Swapchain::new(
                Arc::clone(hardware.graphics_device()),
                Arc::clone(hardware.surface()),
                SwapchainCreateInfo {
                    min_image_count: capabilities.min_image_count,
                    image_format: Some(format),
                    image_extent: hardware.window().inner_size().into(),
                    image_usage: ImageUsage::color_attachment(),
                    composite_alpha: capabilities
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .expect("Could not select an alpha capability"),
                    ..Default::default()
                },
            ).expect("Could not create the swapchain")
        };

        Screen {
            hardware,
            swapchain,
            images,
        }
    }
}
