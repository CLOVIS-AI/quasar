use std::sync::Arc;

use log::{debug, info, trace};
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
    trace!("Initializing Vulkan…");
    let required_extensions = vulkano_win::required_extensions();

    Instance::new(None, Version::V1_2, &required_extensions, None)
        .expect("Couldn't create the Vulkan instance.")
}

fn new_device(instance: &Arc<Instance>) -> (Arc<Device>, QueuesIter) {
    debug!("Searching for available graphics cards…");
    for physical_device in PhysicalDevice::enumerate(&instance) {
        let properties = physical_device.properties();
        trace!(" - {} ({:?})", properties.device_name, properties.device_type);
        trace!("   API version {}", physical_device.api_version());
        trace!("   Driver version {}", properties.driver_version);
    }
    let physical_device = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("Couldn't select a graphics card.");

    info!("Selected GPU:");
    info!(" * {}", physical_device.properties().device_name);

    debug!("Listing available queue families…");
    for family in physical_device.queue_families() {
        trace!(" - Family {} ({} queues available)", family.id(), family.queues_count());
        trace!("   Graphics: {}", family.supports_graphics());
        trace!("   Compute: {}", family.supports_compute());
        trace!("   Minimal image granularity: {:?}", family.min_image_transfer_granularity());
        trace!("   Performant transfers: {}", family.explicitly_supports_transfers());
        trace!("   Sparse bindings: {}", family.supports_sparse_binding());
    }

    debug!("Selected families:");

    let graphical_family = physical_device.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Couldn't find a graphical queue family");
    debug!(" * Graphical family: Family {}", graphical_family.id());

    let transfer_family = physical_device.queue_families()
        .find(|&q| q.explicitly_supports_transfers())
        .expect("Couldn't find a transfer queue family");
    debug!(" * Transfer family:  Family {}", transfer_family.id());

    let compute_family = physical_device.queue_families()
        .find(|&q| q.supports_compute())
        .expect("Couldn't find a compute queue family");
    debug!(" * Compute family:   Family {}", compute_family.id());

    trace!("Creating a device…");
    let extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    Device::new(physical_device, &Features::none(), &extensions,
                [(graphical_family, 0.5), (transfer_family, 0.5), (compute_family, 0.5)].iter().cloned())
        .expect("Couldn't create device.")
}

// endregion
