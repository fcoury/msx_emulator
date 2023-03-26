pub mod cpu;
pub mod devices;
pub mod input;
pub mod memory;
pub mod sound;
pub mod vdp;

pub trait IoDevice {
    fn read(&mut self, port: u16) -> u8;
    fn write(&mut self, port: u16, data: u8);
}
