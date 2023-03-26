#![allow(dead_code)]

use super::IoDevice;
pub struct TMS9918 {
    vram: Vec<u8>,
    screen_mode: u8,
    address_register: u16,
    data_latch: u8,
    status_register: u8,
    is_second_write: bool,
}

impl TMS9918 {
    pub fn new() -> Self {
        Self {
            vram: vec![0; 16 * 1024], // 16 KB VRAM
            screen_mode: 2,
            address_register: 0,
            data_latch: 0,
            status_register: 0,
            is_second_write: false,
        }
    }

    pub fn render_scanline(&mut self) {}
}

impl IoDevice for TMS9918 {
    fn read(&mut self, port: u16) -> u8 {
        match port {
            0x98 => {
                // Read from Data port
                let data = self.vram[self.address_register as usize];
                self.address_register = self.address_register.wrapping_add(1) & 0x3FFF;
                data
            }
            0x99 => {
                // Read from Control port (Status register)
                let status = self.status_register;
                self.status_register &= !(0x80 | 0x40 | 0x20); // Clear bits 7, 6, and 5
                self.is_second_write = false; // Reset the write sequence
                status
            }
            _ => 0, // Ignore other ports
        }
    }

    fn write(&mut self, port: u16, data: u8) {
        match port {
            0x98 => {
                // Write to Data port
                self.vram[self.address_register as usize] = data;
                self.address_register = self.address_register.wrapping_add(1) & 0x3FFF;
            }
            0x99 => {
                // Write to Control port
                if self.is_second_write {
                    // Second write: set high bits of address and command bits
                    self.address_register =
                        (self.address_register & 0x00FF) | ((data as u16 & 0x3F) << 8);
                    self.is_second_write = false;

                    // Handle VDP commands based on the data written (e.g., screen mode change, VRAM access, etc.)
                    // ...
                } else {
                    // First write: set low bits of address and latch data
                    self.address_register = (self.address_register & 0xFF00) | (data as u16);
                    self.is_second_write = true;
                }
            }
            _ => {} // Ignore other ports
        }
    }
}
