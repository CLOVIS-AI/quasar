use std::sync::Arc;

use log::debug;
use winit::event_loop::EventLoop;

use crate::drawing::hardware::Hardware;
use crate::drawing::screen::Screen;

pub struct Engine {
    event_loop: Arc<EventLoop<()>>,
    pub hardware: Arc<Hardware>,
    pub screen: Arc<Screen>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new() -> Engine {
        let event_loop = EventLoop::new();
        let hardware = Arc::new(Hardware::new(&event_loop));
        let screen = Arc::new(Screen::new(Arc::clone(&hardware), &event_loop));

        debug!("Vulkan initialization finished.");
        Engine {
            event_loop: Arc::new(event_loop),
            hardware,
            screen,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
