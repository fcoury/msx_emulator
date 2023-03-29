pub mod bus;
pub mod cpu;
pub mod display;
pub mod input;
pub mod memory;
pub mod sound;
pub mod vdp;

pub trait IoDevice {
    fn is_valid_port(&self, port: u8) -> bool;
    fn read(&mut self, port: u8) -> u8;
    fn write(&mut self, port: u8, data: u8);
}

// +-----------+------------------------------------------------+
// | Range     | Description                                    |
// +-----------+------------------------------------------------+
// | 0x00      | VDP: Video Display Processor (TMS9918) - Data  |
// | 0x01      | VDP: Video Display Processor (TMS9918) - Ctrl  |
// | 0x10      | PSG: Programmable Sound Generator (AY-3-8910)  |
// | 0x11      | PSG: Programmable Sound Generator (AY-3-8910)  |
// | 0x20      | PPI: Peripheral Interface Adapter (8255) - A   |
// | 0x21      | PPI: Peripheral Interface Adapter (8255) - B   |
// | 0x22      | PPI: Peripheral Interface Adapter (8255) - C   |
// | 0x23      | PPI: Peripheral Interface Adapter (8255) - Ctrl|
// | 0xA0-0xAF | Slot Select Register (Only in some models)     |
// | 0x98-0x9B | MSX-MIDI (in MSX-MIDI equipped machines)       |
// +-----------+------------------------------------------------+
