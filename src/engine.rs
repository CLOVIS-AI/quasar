use std::sync::Arc;

use vulkano::instance::{Instance, PhysicalDevice};

pub struct Engine {
    instance: Arc<Instance>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new() -> Engine {
        println!("Initializing Vulkan…");
        let required_extensions = vulkano_win::required_extensions();

        let instance = Instance::new(None, &required_extensions, None)
            .expect("Couldn't create the Vulkan instance.");

        Engine {
            instance,
        }
    }
}
