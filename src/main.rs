mod components;
mod msx;
mod open_msx;

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
    track_flags: bool,

    #[clap(short, long)]
    breakpoint: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    Builder::new()
        // .filter(Some("msx_emulator::components::cpu"), LevelFilter::Info)
        .filter(Some("rustyline"), LevelFilter::Info)
        .filter(None, LevelFilter::Trace)
        .init();

    let mut msx = Msx::new();
    msx.load_bios(cli.rom_path)
        .expect("Failed to load the BIOS");

    msx.max_cycles = cli.max_cycles;
    msx.track_flags = cli.track_flags;
    for breakpoint in cli.breakpoint {
        let breakpoint = u16::from_str_radix(&breakpoint[2..], 16).unwrap();
        msx.add_breakpoint(breakpoint);
    }
    msx.run().unwrap();
}
