mod components;
mod internal_state;
mod msx;
mod open_msx;

use std::path::PathBuf;

use clap::Parser;
#[allow(unused_imports)]
use components::{cpu::Z80, input::Ppi, memory::Memory, sound::AY38910, vdp::TMS9918};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::msx::Msx;

#[derive(Parser, Debug)]
pub struct Cli {
    rom_path: PathBuf,

    #[clap(short = 'c', long)]
    max_cycles: Option<u64>,

    #[clap(short, long)]
    track_flags: bool,

    #[clap(short, long)]
    breakpoint: Vec<String>,

    #[clap(short, long)]
    open_msx: bool,

    #[clap(short = 'm', long)]
    break_on_mismatch: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = "info";
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(log_level))?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Builder::new()
    //     .filter(Some("msx_emulator::components::cpu"), LevelFilter::Info)
    //     .filter(Some("rustyline"), LevelFilter::Info)
    //     .filter(None, LevelFilter::Trace)
    //     .init();

    let mut msx = Msx::new(&cli);
    msx.load_bios(cli.rom_path)
        .expect("Failed to load the BIOS");

    msx.max_cycles = cli.max_cycles;
    msx.track_flags = cli.track_flags;
    for breakpoint in cli.breakpoint {
        let breakpoint = u16::from_str_radix(&breakpoint[2..], 16)?;
        msx.add_breakpoint(breakpoint);
    }
    msx.run()?;

    Ok(())
}
