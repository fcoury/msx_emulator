use super::IoDevice;

pub struct Ppi {
    port_a: u8,
    port_b: u8,
    port_c: u8,
    control: u8,
}

impl Ppi {
    pub fn new() -> Self {
        Ppi {
            port_a: 0,
            port_b: 0,
            port_c: 0,
            control: 0,
        }
    }

    fn is_port_a_input(&self) -> bool {
        self.control & 0b1000_0000 == 0
    }

    fn is_port_b_input(&self) -> bool {
        self.control & 0b10 == 0
    }
}

impl IoDevice for Ppi {
    fn is_valid_port(&self, port: u8) -> bool {
        matches!(port, 0xA8 | 0xA9 | 0xAA | 0xAB)
    }

    fn read(&mut self, port: u8) -> u8 {
        match port {
            0xA8 => {
                println!(
                    "  *** [PPI] Reading from PPI port {:02X} (input? {}) = {:02X}",
                    port,
                    self.is_port_a_input(),
                    self.port_a,
                );
                if self.is_port_a_input() {
                    self.port_a
                } else {
                    0xFF
                }
            }
            0xA9 => {
                if self.is_port_b_input() {
                    self.port_b
                } else {
                    0xFF
                }
            }
            0xAA => self.port_c,
            _ => 0xFF,
        }
    }

    fn write(&mut self, port: u8, value: u8) {
        match port {
            0xA8 => {
                println!(
                    "  *** [PPI] Writing '{:02X}' to PPI port 0xA8 (output? {})",
                    value,
                    !self.is_port_a_input()
                );
                // if !self.is_port_a_input() {
                self.port_a = value;
                // }
            }
            0xA9 => {
                println!(
                    "  *** [PPI] Writing '{:02X}' to PPI port 0xA9 (output? {})",
                    value,
                    !self.is_port_a_input()
                );
                // if !self.is_port_b_input() {
                self.port_b = value;
                // }
            }
            0xAA => {
                println!("  *** [PPI] Writing '{:02X}' to PPI port 0xAA", value);
                self.port_c = value;
            }
            0xAB => {
                println!(
                    "  *** [PPI] Writing '{:02X}' to PPI port 0xAB (control)",
                    value
                );
                self.control = value & 0x7F;
                let bit_number = (value >> 1) & 0x07;
                let bit_status = value & 0x01;
                if bit_status == 0 {
                    self.port_c &= !(1 << bit_number);
                } else {
                    self.port_c |= 1 << bit_number;
                }
            }
            _ => (),
        }
    }
}
