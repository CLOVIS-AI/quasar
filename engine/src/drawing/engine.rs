use std::sync::Arc;

use log::debug;
use winit::event_loop::EventLoop;

use crate::drawing::painter::Painter;
use crate::drawing::renderer::Renderer;

#[derive(Clone)]
pub struct Engine {
    event_loop: Arc<EventLoop<()>>,
    pub renderer: Arc<Renderer>,
    pub screen: Arc<Painter>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new() -> Engine {
        let event_loop = EventLoop::new();
        let renderer = Renderer::new();
        let screen = Painter::new(renderer.clone(), &event_loop);

        debug!("Vulkan initialization finished.");
        Engine {
            event_loop: Arc::new(event_loop),
            renderer,
            screen,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
