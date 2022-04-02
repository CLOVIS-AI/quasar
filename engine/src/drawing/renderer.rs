use std::sync::Arc;

use vulkano::device::{Device, Queue, QueuesIter};
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::Instance;
use vulkano::Version;

/// Relay between the [`Engine`] and Vulkan.
pub struct Renderer {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub transfer_queue: Arc<Queue>,
    pub compute_queue: Arc<Queue>,
}

impl Renderer {
    pub fn new() -> Arc<Self> {
        let instance = new_instance();
        let (device, mut queues) = new_device(&instance);
        let graphics_queue = queues.next().expect("Couldn't access the graphics queue");
        let transfer_queue = queues.next().expect("Couldn't access the transfer queue");
        let compute_queue = queues.next().expect("Couldn't access the compute queue");

        Arc::new(
            Renderer {
                instance,
                device,
                graphics_queue,
                transfer_queue,
                compute_queue,
            }
        )
    }
}

// region Constructor utilities

fn new_instance() -> Arc<Instance> {
    println!("Initializing Vulkan…");
    let required_extensions = vulkano_win::required_extensions();

    Instance::new(None, Version::V1_2, &required_extensions, None)
        .expect("Couldn't create the Vulkan instance.")
}

fn new_device(instance: &Arc<Instance>) -> (Arc<Device>, QueuesIter) {
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
        .expect("Couldn't find a graphical queue family");
    println!(" * \tGraphical family: Family {}", graphical_family.id());

    let transfer_family = physical_device.queue_families()
        .find(|&q| q.explicitly_supports_transfers())
        .expect("Couldn't find a transfer queue family");
    println!(" * \tTransfer family:  Family {}", transfer_family.id());

    let compute_family = physical_device.queue_families()
        .find(|&q| q.supports_compute())
        .expect("Couldn't find a compute queue family");
    println!(" * \tCompute family:   Family {}", compute_family.id());

    println!("\nCreating a device…");
    let extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    Device::new(physical_device, &Features::none(), &extensions,
                [(graphical_family, 0.5), (transfer_family, 0.5), (compute_family, 0.5)].iter().cloned())
        .expect("Couldn't create device.")
}

// endregion
