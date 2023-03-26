mod components;

use std::{fs::File, io::Read};

#[allow(unused_imports)]
use components::{cpu::Z80, input::Input, memory::Memory, sound::AY38910, vdp::TMS9918};
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    Builder::new().filter(None, LevelFilter::Trace).init();

    let mut memory = Memory::new(65_536);
    // let mut vdp = TMS9918::new();
    // let mut sound_chip = AY38910::new();
    // let mut input = Input::new();

    // gets the rom path from args
    let rom_path = std::env::args().nth(1).expect("No rom path provided");
    let max_cycles = std::env::args()
        .nth(2)
        .map(|length| length.parse().unwrap());

    let load_address = 0x0000; // Change this value based on your desired load address
    load_binary(&mut memory, &rom_path, load_address).expect("Failed to load the binary");

    let mut cpu = Z80::new(memory);
    cpu.max_cycles = max_cycles;
    println!("max_cycles: {:?}", cpu.max_cycles);

    loop {
        // Main emulator loop
        cpu.execute_cycle();
        // vdp.render_scanline();

        // Generate audio samples and output audio here

        // Check for and handle user input events
        // input.handle_input(event);

        // Update the user interface
        // This could include updating the display, showing debug information, etc.

        // Break the loop if the user decides to exit the application
    }
}

fn load_binary(memory: &mut Memory, path: &str, load_address: u16) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    for (i, byte) in buffer.iter().enumerate() {
        let address = load_address.wrapping_add(i as u16);
        memory.write_byte(address, *byte);
    }

    Ok(())
}
