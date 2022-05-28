use std::sync::Arc;

use log::{debug, info, trace};
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo};
use vulkano::device::DeviceExtensions;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

/// Relay between the [`Engine`] and Vulkan.
pub struct Hardware {
    surface: Arc<Surface<Window>>,
    graphics_queue: Arc<Queue>,
    compute_queue: Arc<Queue>,
}

impl Hardware {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        debug!("Vulkan and window initialization…");
        trace!("Connecting to Vulkan…");
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        })
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
                info!(
                    " - {} ({:?})",
                    properties.device_name, properties.device_type
                );
                trace!("   API: {}", physical.api_version());
                trace!("   Driver: {}", properties.driver_version);
                physical
            })
            .filter(|physical| {
                physical
                    .supported_extensions()
                    .is_superset_of(&device_extensions)
            })
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
            trace!(
                " - {} with score {}",
                physical_candidate.properties().device_name,
                score
            );
            for family in physical_candidate.queue_families() {
                trace!(
                    "    - Family {} ({} queues)",
                    family.id(),
                    family.queues_count()
                );
                trace!("      Graphics: {}", family.supports_graphics());
                trace!("      Compute: {}", family.supports_compute());
                trace!(
                    "      Minimal image granularity: {:?}",
                    family.min_image_transfer_granularity()
                );
                trace!(
                    "      Performant transfers: {}",
                    family.explicitly_supports_transfers()
                );
                trace!(
                    "      Sparse bindings: {}",
                    family.supports_sparse_binding()
                );
            }
        }

        // Find a graphics queue and a compute queue
        let (_, graphics_physical, graphics_family) = physical_candidates
            .iter()
            .filter_map(|(score, physical)| {
                physical
                    .queue_families()
                    .find(|family| {
                        family.supports_graphics()
                            && family.supports_surface(&surface).unwrap_or(false)
                    })
                    .map(|family| (score, physical, family))
            })
            .min_by_key(|(score, _, _)| *score)
            .expect("Could not find a suitable graphics queue family");
        info!(
            "Selected for graphics: {} / family {}",
            graphics_physical.properties().device_name,
            graphics_family.id()
        );

        let (_, compute_physical, compute_family) = physical_candidates
            .iter()
            .filter_map(|(score, physical)| {
                physical
                    .queue_families()
                    .find(|family| family.supports_compute())
                    .map(|family| (score, physical, family))
            })
            .min_by_key(|(score, _, _)| *score)
            .expect("Could not find a suitable compute queue family");
        info!(
            "Selected for compute: {} / family {}",
            compute_physical.properties().device_name,
            compute_family.id()
        );

        debug!("Creating the device(s)…");
        // Case 1: different GPUs
        // Case 2: same GPU, but different families
        // Case 3: same GPU, same family
        let graphics_device: Arc<Device>;
        let graphics_queue: Arc<Queue>;
        let compute_device: Arc<Device>;
        let compute_queue: Arc<Queue>;
        if graphics_physical.index() == compute_physical.index() {
            let queue_create_infos = if graphics_family.id() == compute_family.id() {
                vec![QueueCreateInfo {
                    family: graphics_family,
                    queues: vec![0.5, 0.5],
                    _ne: Default::default(),
                }]
            } else {
                vec![
                    QueueCreateInfo::family(graphics_family),
                    QueueCreateInfo::family(compute_family),
                ]
            };

            let (device, mut queues) = Device::new(
                *graphics_physical,
                DeviceCreateInfo {
                    enabled_extensions: graphics_physical
                        .required_extensions()
                        .union(&device_extensions),
                    queue_create_infos,
                    ..Default::default()
                },
            )
                .expect("Couldn't instantiate the device");

            graphics_device = Arc::clone(&device);
            compute_device = Arc::clone(&device);
            graphics_queue = queues
                .next()
                .expect("Couldn't instantiate the graphics queue");
            compute_queue = queues
                .next()
                .expect("Couldn't instantiate the compute queue");
        } else {
            let (graphics_device_, mut graphics_queues) = Device::new(
                *graphics_physical,
                DeviceCreateInfo {
                    enabled_extensions: graphics_physical
                        .required_extensions()
                        .union(&device_extensions),
                    queue_create_infos: vec![QueueCreateInfo::family(graphics_family)],
                    ..Default::default()
                },
            )
                .expect("Couldn't instantiate the graphics device");

            let (compute_device_, mut compute_queues) = Device::new(
                *compute_physical,
                DeviceCreateInfo {
                    enabled_extensions: compute_physical
                        .required_extensions()
                        .union(&device_extensions),
                    queue_create_infos: vec![QueueCreateInfo::family(compute_family)],
                    ..Default::default()
                },
            )
                .expect("Couldn't instantiate the compute device");

            graphics_device = graphics_device_;
            graphics_queue = graphics_queues
                .next()
                .expect("Couldn't instantiate the graphics queue");
            compute_device = compute_device_;
            compute_queue = compute_queues
                .next()
                .expect("Couldn't instantiate the compute queue");
        }

        trace!("Done creating the devices.");

        Hardware {
            surface,
            graphics_queue,
            compute_queue,
        }
    }

    pub fn surface(&self) -> &Arc<Surface<Window>> {
        &self.surface
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    pub fn graphics_queue(&self) -> &Arc<Queue> {
        &self.graphics_queue
    }

    pub fn graphics_device(&self) -> &Arc<Device> {
        self.graphics_queue.device()
    }

    pub fn compute_queue(&self) -> &Arc<Queue> {
        &self.compute_queue
    }

    pub fn compute_device(&self) -> &Arc<Device> {
        self.compute_queue.device()
    }
}
