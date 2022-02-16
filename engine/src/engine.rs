use std::sync::Arc;

use vulkano::device::{Device, Queue};
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::physical::PhysicalDevice;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::swapchain::ColorSpace::SrgbNonLinear;
use vulkano::swapchain::FullscreenExclusive::Default;
use vulkano::swapchain::PresentMode::Fifo;
use vulkano::swapchain::SurfaceTransform::Identity;
use vulkano::Version;
use vulkano_win::create_vk_surface_from_handle;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

#[derive(Clone)]
pub struct Engine {
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface<Window>>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub images: Vec<Arc<SwapchainImage<Window>>>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new(event_loop: &EventLoop<()>) -> Engine {
        println!("Initializing Vulkan…");
        let required_extensions = vulkano_win::required_extensions();

        let instance = Instance::new(None, Version::V1_2, &required_extensions, None)
            .expect("Couldn't create the Vulkan instance.");

        println!("\nSearching for available graphics cards…");
        for physical_device in PhysicalDevice::enumerate(&instance) {
            println!(" - \t{} ({:?})\n\tAPI version {}\n\tDriver version {}",
                     physical_device.properties().device_name,
                     physical_device.properties().device_type,
                     physical_device.api_version(),
                     physical_device.properties().driver_version);
        }
        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .expect("Couldn't select a graphics card.");

        println!("Selected:");
        println!(" * \t{}", physical_device.properties().device_name);

        println!("\nListing available queue families…");
        for family in physical_device.queue_families() {
            println!(" - \tFamily {} ({} queues available)\n\tGraphics: {}\n\tCompute: {}\n\tMinimal image granularity: {:?}\n\tPerformant transfers: {}\n\tSparse bindings: {}",
                     family.id(),
                     family.queues_count(),
                     family.supports_graphics(),
                     family.supports_compute(),
                     family.min_image_transfer_granularity(),
                     family.explicitly_supports_transfers(),
                     family.supports_sparse_binding());
        }

        println!("Selected:");
        let graphical_family = physical_device.queue_families()
            .find(|&q| q.supports_graphics())
            .expect("couldn't find a graphical queue family");
        println!(" * \tGraphical family: {}", graphical_family.id());

        println!("\nCreating a device…");
        let (device, mut queues) = {
            let extensions = DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            };

            Device::new(physical_device, &Features::none(), &extensions,
                        [(graphical_family, 0.5)].iter().cloned())
                .expect("Couldn't create device.")
        };
        let queue = queues.next().expect("Could not find a queue.");

        println!("\nCreating the surface…");
        let window = WindowBuilder::new().build(&event_loop).expect("Couldn't create the window");
        let surface = create_vk_surface_from_handle(window, instance.clone()).expect("Couldn't create the Vulkan surface");

        println!("\nCreating the swap-chain…");
        let capabilities = surface.capabilities(device.physical_device()).expect("Couldn't instantiate the capabilities for the swap chain");
        let dimensions = capabilities.current_extent.unwrap_or([1280, 1024]);
        let alpha = capabilities.supported_composite_alpha.iter().next().expect("Couldn't get the supported alpha");
        let format = capabilities.supported_formats[0].0;

        let (swapchain, images) = Swapchain::start(device.clone(), surface.clone())
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

        println!("Vulkan initialization finished.");
        Engine {
            instance,
            surface,
            swapchain,
            images,
            device,
            graphics_queue: queue,
        }
    }
}
