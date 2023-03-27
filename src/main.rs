mod components;
mod msx;

#[allow(unused_imports)]
use components::{cpu::Z80, input::Input, memory::Memory, sound::AY38910, vdp::TMS9918};
use env_logger::Builder;
use log::LevelFilter;

use crate::msx::Msx;

fn main() {
    Builder::new()
        .filter(Some("msx_emulator::components::cpu"), LevelFilter::Error)
        .filter(None, LevelFilter::Trace)
        .init();

    let rom_path = std::env::args().nth(1).expect("No binary path provided");
    let max_cycles = std::env::args()
        .nth(2)
        .map(|length| length.parse().unwrap());

    let mut msx = Msx::new();
    msx.load_bios(&rom_path).expect("Failed to load the BIOS");

    msx.max_cycles = max_cycles;
    msx.run();
}
