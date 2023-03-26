pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            data: vec![0; size],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    pub fn load_rom(&mut self, start_address: u16, data: &[u8]) {
        let start = start_address as usize;
        let end = start + data.len();
        self.data[start..end].copy_from_slice(data);
    }
}
