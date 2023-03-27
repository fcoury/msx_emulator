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
