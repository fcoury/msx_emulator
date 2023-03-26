mod components;

#[allow(unused_imports)]
use components::{cpu::Z80, input::Input, memory::Memory, sound::AY38910, vdp::TMS9918};

fn main() {
    let memory = Memory::new(65_536);
    // let mut vdp = TMS9918::new();
    // let mut sound_chip = AY38910::new();
    // let mut input = Input::new();
    let mut cpu = Z80::new(memory);

    // Initialize user interface and event handling here

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
