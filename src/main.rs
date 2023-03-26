mod components;
mod msx;

#[allow(unused_imports)]
use components::{cpu::Z80, input::Input, memory::Memory, sound::AY38910, vdp::TMS9918};
use env_logger::Builder;
use log::LevelFilter;

use crate::msx::Msx;

fn main() {
    Builder::new().filter(None, LevelFilter::Trace).init();

    let binary_path = std::env::args().nth(1).expect("No binary path provided");
    let max_cycles = std::env::args()
        .nth(2)
        .map(|length| length.parse().unwrap());
    let load_address = 0x0000; // Change this value based on your desired load address

    let mut msx = Msx::new();
    msx.load_binary(&binary_path, load_address)
        .expect("Failed to load the binary");
    msx.max_cycles = max_cycles;
    msx.run();
}
