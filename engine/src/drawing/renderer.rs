use std::sync::Arc;

use log::{debug, info, trace};
use vulkano::device::{Device, Queue, QueuesIter};
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::instance::Instance;
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

/// Relay between the [`Engine`] and Vulkan.
pub struct Renderer {
    graphics_queue: Arc<Queue>,
    compute_queue: Arc<Queue>,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        debug!("Vulkan and window initialization…");
        trace!("Connecting to Vulkan…");
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, Version::V1_2, &required_extensions, None)
            .expect("Couldn't instantiate the Vulkan instance");

        trace!("Creating the surface…");
        let surface = WindowBuilder::new()
            .build_vk_surface(event_loop, Arc::clone(&instance))
            .expect("Couldn't create a Vulkan surface");

        // The extensions required by the engine
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        info!("Selecting the devices to use…");
        let physical_candidates: Vec<(i32, PhysicalDevice)> = PhysicalDevice::enumerate(&instance)
            .map(|physical| {
                let properties = physical.properties();
                info!(" - {} ({:?})", properties.device_name, properties.device_type);
                trace!("   API: {}", physical.api_version());
                trace!("   Driver: {}", properties.driver_version);
                physical
            })
            .filter(|physical| physical.supported_extensions().is_superset_of(&device_extensions))
            .map(|physical| {
                // Assign a score to each type of device
                // Lower means better
                let score = match physical.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 0,
                    PhysicalDeviceType::IntegratedGpu => 1,
                    PhysicalDeviceType::VirtualGpu => 2,
                    PhysicalDeviceType::Cpu => 3,
                    PhysicalDeviceType::Other => 4,
                };

                (score, physical)
            })
            .collect();

        // Debug the different queues
        trace!("Available family queues:");
        for (score, physical_candidate) in &physical_candidates {
            trace!(" - {} with score {}", physical_candidate.properties().device_name, score);
            for family in physical_candidate.queue_families() {
                trace!("    - Family {} ({} queues)", family.id(), family.queues_count());
                trace!("      Graphics: {}", family.supports_graphics());
                trace!("      Compute: {}", family.supports_compute());
                trace!("      Minimal image granularity: {:?}", family.min_image_transfer_granularity());
                trace!("      Performant transfers: {}", family.explicitly_supports_transfers());
                trace!("      Sparse bindings: {}", family.supports_sparse_binding());
            }
        }

        // Find a graphics queue and a compute queue
        let (_, graphics_physical, graphics_family) = physical_candidates.iter()
            .filter_map(|(score, physical)| {
                physical.queue_families()
                    .find(|family| family.supports_graphics() && surface.is_supported(*family).unwrap_or(false))
                    .map(|family| (score, physical, family))
            })
            .min_by_key(|(score, _, _)| *score)
            .expect("Could not find a suitable graphics queue family");
        info!("Selected for graphics: {} / family {}", graphics_physical.properties().device_name, graphics_family.id());

        let (_, compute_physical, compute_family) = physical_candidates.iter()
            .filter_map(|(score, physical)| {
                physical.queue_families()
                    .find(|family| family.supports_compute())
                    .map(|family| (score, physical, family))
            })
            .min_by_key(|(score, _, _)| *score)
            .expect("Could not find a suitable compute queue family");
        info!("Selected for compute: {} / family {}", compute_physical.properties().device_name, compute_family.id());

        debug!("Creating the device(s)…");
        let graphics_device: Arc<Device>;
        let graphics_queue: Arc<Queue>;
        let compute_device: Arc<Device>;
        let compute_queue: Arc<Queue>;
        if graphics_physical.index() == compute_physical.index() {
            let (device, mut queues): (Arc<Device>, QueuesIter) = Device::new(
                *graphics_physical,
                &Features::none(),
                &graphics_physical
                    .required_extensions()
                    .union(&device_extensions),
                [(graphics_family, 0.5), (compute_family, 0.5)].iter().cloned(),
            ).expect("Couldn't instantiate the device");

            graphics_device = Arc::clone(&device);
            compute_device = Arc::clone(&device);
            graphics_queue = queues.next().expect("Couldn't instantiate the graphics queue");
            compute_queue = queues.next().expect("Couldn't instantiate the compute queue");
        } else {
            let (graphics_device_, mut graphics_queues): (Arc<Device>, QueuesIter) = Device::new(
                *graphics_physical,
                &Features::none(),
                &graphics_physical
                    .required_extensions()
                    .union(&device_extensions),
                [(graphics_family, 0.5)].iter().cloned(),
            ).expect("Couldn't instantiate the graphics device");

            let (compute_device_, mut compute_queues_): (Arc<Device>, QueuesIter) = Device::new(
                *compute_physical,
                &Features::none(),
                &compute_physical
                    .required_extensions()
                    .union(&device_extensions),
                [(compute_family, 0.5)].iter().cloned(),
            ).expect("Couldn't instantiate the graphics device");

            graphics_device = graphics_device_;
            graphics_queue = graphics_queues.next().expect("Couldn't instantiate the graphics queue");
            compute_device = compute_device_;
            compute_queue = compute_queues_.next().expect("Couldn't instantiate the compute queue");
        }

        trace!("Done creating the devices.");

        Renderer {
            graphics_queue,
            compute_queue,
        }
    }
}
