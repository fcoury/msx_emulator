mod components;
mod msx;

use std::path::PathBuf;

use clap::Parser;
#[allow(unused_imports)]
use components::{cpu::Z80, input::Input, memory::Memory, sound::AY38910, vdp::TMS9918};
use env_logger::Builder;
use log::LevelFilter;

use crate::msx::Msx;

#[derive(Parser, Debug)]
struct Cli {
    rom_path: PathBuf,
    max_cycles: Option<u64>,

    #[clap(short, long)]
    breakpoint: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    Builder::new()
        // .filter(Some("msx_emulator::components::cpu"), LevelFilter::Error)
        .filter(None, LevelFilter::Trace)
        .init();

    // let rom_path = std::env::args().nth(1).expect("No binary path provided");
    // let max_cycles = std::env::args()
    //     .nth(2)
    //     .map(|length| length.parse().unwrap());
    // let breakpoint = std::env::args().nth(3).unwrap_or("".to_string());

    let mut msx = Msx::new();
    msx.load_bios(cli.rom_path)
        .expect("Failed to load the BIOS");

    msx.max_cycles = cli.max_cycles;
    for breakpoint in cli.breakpoint {
        // parses breakpoint like 0x1234
        let breakpoint = u16::from_str_radix(&breakpoint[2..], 16).unwrap();
        msx.add_breakpoint(breakpoint);
    }
    msx.run();
}
