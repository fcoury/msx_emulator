#![allow(dead_code)]

use tracing::{error, trace};

use super::IoDevice;
pub struct TMS9918 {
    pub vram: Vec<u8>,
    pub screen_mode: u8,
    pub screen_buffer: Vec<Vec<u32>>,
    pub address_register: u16,
    pub data_latch: u8,
    pub status_register: u8,
    pub is_second_write: bool,
}

impl TMS9918 {
    pub fn new() -> Self {
        const SCREEN_WIDTH: usize = 320;
        const SCREEN_HEIGHT: usize = 192;

        let screen_buffer = vec![vec![0; SCREEN_WIDTH]; SCREEN_HEIGHT];

        Self {
            vram: vec![0; 16 * 1024], // 16 KB VRAM
            screen_mode: 0,
            screen_buffer,
            address_register: 0,
            data_latch: 0,
            status_register: 0,
            is_second_write: false,
        }
    }

    pub fn render_scanline(&mut self, scanline: u16) {
        #[allow(clippy::single_match)]
        match self.screen_mode {
            0 => self.render_scanline_text_mode(scanline),
            // Add other screen modes here
            _ => {
                error!("\nUnsupported screen mode: {}\n", self.screen_mode)
            }
        }
    }

    fn render_scanline_text_mode(&mut self, scanline: u16) {
        const CHARS_PER_ROW: u16 = 40;
        const ROWS: u16 = 24;
        const PATTERN_HEIGHT: u16 = 8;

        let foreground_color = 0xFFFFFFFF; // White color in RGBA8888 format
        let background_color = 0xFF0000FF; // Black color in RGBA8888 format

        if scanline >= ROWS * PATTERN_HEIGHT {
            return; // Beyond the visible screen area
        }

        let row = scanline / PATTERN_HEIGHT;
        let y_within_pattern = scanline % PATTERN_HEIGHT;

        for col in 0..CHARS_PER_ROW {
            let char_index = self.vram[(row * CHARS_PER_ROW + col) as usize];
            let pattern_offset = (char_index as u16) * PATTERN_HEIGHT + y_within_pattern;
            let pattern_line = self.vram[pattern_offset as usize];

            for x_within_pattern in 0..6 {
                let pixel = (pattern_line >> (7 - x_within_pattern)) & 1;
                let x = (col * 8 + x_within_pattern) as usize;
                let y = scanline as usize;
                self.screen_buffer[y][x] = if pixel == 1 {
                    foreground_color
                } else {
                    background_color
                };
            }
        }
    }
}

impl IoDevice for TMS9918 {
    fn is_valid_port(&self, port: u8) -> bool {
        matches!(port, 0x98 | 0x99)
    }

    fn read(&mut self, port: u8) -> u8 {
        trace!("[vdp] Read from VDP port: {:02X}", port);
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

    fn write(&mut self, port: u8, data: u8) {
        match port {
            0x98 => {
                // Write to Data port
                trace!(
                    "[vdp] Write to VRAM[{:04X}] = {:02X}",
                    self.address_register,
                    data
                );
                self.vram[self.address_register as usize] = data;
                self.address_register = self.address_register.wrapping_add(1) & 0x3FFF;
            }
            0x99 => {
                // Write to Control port
                trace!("[vdp] Write to VDP control port: {:02X}", data);
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
