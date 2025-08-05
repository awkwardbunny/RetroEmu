use libretro::machine::AppleIIe;
use log::{info, LevelFilter};
use env_logger;

const LOG_LEVEL: LevelFilter = LevelFilter::Debug;
fn main() {
    env_logger::builder().filter_level(LOG_LEVEL).init();
    info!("Starting RetroEmu");

    let mut a2e = AppleIIe::new();
    a2e.reset();
    a2e.run();
}
