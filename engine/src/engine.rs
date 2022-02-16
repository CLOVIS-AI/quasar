use std::rc::Rc;
use std::sync::Arc;

use winit::event_loop::EventLoop;

use crate::painter::Painter;
use crate::renderer::Renderer;

#[derive(Clone)]
pub struct Engine {
    event_loop: Rc<EventLoop<()>>,
    pub renderer: Arc<Renderer>,
    pub screen: Arc<Painter>,
}

impl Engine {
    /// Instantiates the Quasar Engine.
    pub fn new() -> Engine {
        let event_loop = EventLoop::new();
        let renderer = Renderer::new();
        let screen = Painter::new(renderer.clone(), &event_loop);

        println!("Vulkan initialization finished.");
        Engine {
            event_loop: Rc::new(event_loop),
            renderer,
            screen,
        }
    }
}
