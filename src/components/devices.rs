use super::IoDevice;

pub struct DummyIODevice;

impl IoDevice for DummyIODevice {
    fn read(&mut self, _port: u16) -> u8 {
        0xFF
    }

    fn write(&mut self, _port: u16, _data: u8) {}
}
