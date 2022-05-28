use log::info;
use simple_logger::SimpleLogger;

use quasar_engine::drawing::engine::Engine;

fn main() {
    SimpleLogger::new().init().unwrap_or_else(|info| {
        println!(
            "Could not initialize the logger. Logging is disabled: {}",
            info
        )
    });

    info!("Starting…");
    Engine::new();
}
