use log::info;
use winit::event_loop::EventLoop;

use mcrs::app::StateApplication;
use mcrs::game;

fn main() {
    env_logger::init();

    info!("Starting MCRS");
    let event_loop = EventLoop::new().unwrap();
    let window_state = StateApplication::new();
    let mut game = game::MCRS::new(window_state, event_loop);
    game.run();
}
