use std::sync::Arc;

use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::Instance;
use vulkano::Version;

pub struct Engine {
    instance: Arc<Instance>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new() -> Engine {
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

        Engine {
            instance,
        }
    }
}
