use std::sync::Arc;

use log::{debug, trace};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::swapchain::ColorSpace::SrgbNonLinear;
use vulkano::swapchain::FullscreenExclusive::Default;
use vulkano::swapchain::PresentMode::Fifo;
use vulkano::swapchain::SurfaceTransform::Identity;
use vulkano_win::create_vk_surface_from_handle;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

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
            let capabilities = hardware.surface()
                .capabilities(hardware.graphics_device().physical_device())
                .expect("Could not query the surface capabilities");

            let composite_alpha = capabilities.supported_composite_alpha.iter().next().expect("Could not select any alpha capabilities");

            let format = capabilities.supported_formats.iter().next().expect("Could not select any format capabilities").0;

            let dimensions: [u32; 2] = hardware.window().inner_size().into();

            Swapchain::start(Arc::clone(hardware.graphics_device()), Arc::clone(hardware.surface()))
                .num_images(capabilities.min_image_count)
                .format(format)
                .dimensions(dimensions)
                .usage(ImageUsage::color_attachment())
                .sharing_mode(hardware.graphics_queue())
                .composite_alpha(composite_alpha)
                .build()
                .expect("Couldn't create the swapchain")
        };

        Screen {
            hardware,
            swapchain,
            images,
        }
    }
}
