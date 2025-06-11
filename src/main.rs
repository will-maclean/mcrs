use mcrs::run;
use pollster::block_on;

fn main() {
    env_logger::init();

    block_on(run());
}
