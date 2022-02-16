use std::sync::Arc;

use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::swapchain::ColorSpace::SrgbNonLinear;
use vulkano::swapchain::FullscreenExclusive::Default;
use vulkano::swapchain::PresentMode::Fifo;
use vulkano::swapchain::SurfaceTransform::Identity;
use vulkano_win::create_vk_surface_from_handle;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::renderer::Renderer;

pub struct Painter {
    renderer: Arc<Renderer>,
    surface: Arc<Surface<Window>>,
    swap_chain: Arc<Swapchain<Window>>,
    swap_chain_images: Vec<Arc<SwapchainImage<Window>>>,
}

impl Painter {
    pub fn new(renderer: Arc<Renderer>, event_loop: &EventLoop<()>) -> Arc<Self> {
        println!("\nCreating the surface…");
        let window = WindowBuilder::new().build(&event_loop).expect("Couldn't create the window");
        let surface = create_vk_surface_from_handle(window, renderer.instance.clone()).expect("Couldn't create the Vulkan surface");

        println!("\nCreating the swap-chain…");
        let capabilities = surface.capabilities(renderer.device.physical_device()).expect("Couldn't instantiate the capabilities for the swap-chain");
        let dimensions = capabilities.current_extent.unwrap_or([1280, 1024]);
        let alpha = capabilities.supported_composite_alpha.iter().next().expect("Couldn't get the supported alpha");
        let format = capabilities.supported_formats[0].0;

        let (swap_chain, swap_chain_images) = Swapchain::start(renderer.device.clone(), surface.clone())
            .num_images(capabilities.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .layers(1)
            .usage(ImageUsage::color_attachment())
            .transform(Identity)
            .composite_alpha(alpha)
            .present_mode(Fifo)
            .fullscreen_exclusive(Default)
            .clipped(true)
            .color_space(SrgbNonLinear)
            .build().expect("Couldn't build the swap-chain");

        Arc::new(
            Painter {
                renderer,
                surface,
                swap_chain,
                swap_chain_images,
            }
        )
    }
}
