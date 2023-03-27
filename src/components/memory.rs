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
        match address {
            0x0000..=0x3FFF => self.data[address as usize],
            0x4000..=0x7FFF => self.data[address as usize],
            0x8000..=0xBFFF => self.data[address as usize],
            0xC000..=0xDFFF => self.data[address as usize],
            0xE000..=0xFFFF => {
                // Implement I/O read behavior here
                0xFF // Return a default value for now
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => {
                // Writing to BIOS is typically not allowed
                panic!("Writing to BIOS is not allowed")
            }
            0x4000..=0x7FFF => {
                // Writing to BASIC is typically not allowed
                panic!("Writing to BASIC is not allowed")
            }
            0x8000..=0xBFFF => {
                panic!("Writing to VRAM is not allowed")
            }
            0xC000..=0xDFFF => self.data[address as usize] = value,
            0xE000..=0xFFFF => {
                // Implement I/O write behavior here
            }
        }
    }

    pub fn load_bios(&mut self, buffer: &[u8]) -> std::io::Result<()> {
        let load_address: u16 = 0x0000;
        for (i, byte) in buffer.iter().enumerate() {
            let address = load_address.wrapping_add(i as u16);
            self.data[address as usize] = *byte;
        }

        Ok(())
    }

    // pub fn read_byte(&self, address: u16) -> u8 {
    //     self.data[address as usize]
    // }

    // pub fn write_byte(&mut self, address: u16, value: u8) {
    //     self.data[address as usize] = value;
    // }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let low_byte = (value & 0x00FF) as u8;
        let high_byte = ((value & 0xFF00) >> 8) as u8;
        self.write_byte(address, low_byte);
        self.write_byte(address + 1, high_byte);
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address) as u16;
        let high_byte = self.read_byte(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    #[allow(unused)]
    pub fn load_rom(&mut self, start_address: u16, data: &[u8]) {
        let start = start_address as usize;
        let end = start + data.len();
        self.data[start..end].copy_from_slice(data);
    }
}
