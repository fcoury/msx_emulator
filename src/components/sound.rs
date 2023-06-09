#![allow(dead_code)]

use tracing::trace;

use super::IoDevice;

pub struct AY38910 {
    registers: [u8; 16],
    selected_register: u8,
}

impl AY38910 {
    pub fn new() -> Self {
        Self {
            registers: [0; 16],
            selected_register: 0,
            // ... (Initialize other fields)
        }
    }

    pub fn generate_sample(&mut self) -> f32 {
        // Generate a single audio sample
        todo!()
    }
}

impl IoDevice for AY38910 {
    fn is_valid_port(&self, port: u8) -> bool {
        matches!(port, 0xA0 | 0xA1)
    }

    fn read(&mut self, port: u8) -> u8 {
        match port {
            0xA0 => self.selected_register,
            0xA1 => self.registers[self.selected_register as usize],
            _ => 0,
        }
    }

    fn write(&mut self, port: u8, data: u8) {
        match port {
            0xA0 => {
                trace!("[psg] Selecting register {:02X}", data);
                self.selected_register = data & 0x0F;
            }
            0xA1 => {
                trace!(
                    "[psg] Writing {:02X} to register {:02X}",
                    data,
                    self.selected_register
                );
                self.registers[self.selected_register as usize] = data;
                // ... (Update the internal state of the PSG based on the new register value)
            }
            _ => {}
        }
    }
}
