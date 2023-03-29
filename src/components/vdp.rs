#![allow(dead_code)]

use tracing::error;

use super::IoDevice;

#[derive(Debug, Clone, Copy, Default)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub pattern: u32,
    pub color: u8,
    pub collision: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TMS9918 {
    pub vram: [u8; 0x4000],
    pub registers: [u8; 8],
    pub status: u8,
    pub address: u16,
    pub latch: bool,
    pub command: u8,
    pub screen_buffer: [u8; 256 * 192],
    pub sprites: [Sprite; 8],
    pub frame: u8,
    pub line: u8,
    pub vblank: bool,
}

impl TMS9918 {
    pub fn new() -> Self {
        Self {
            vram: [0; 0x4000],
            registers: [0; 8],
            status: 0,
            address: 0,
            latch: false,
            command: 0,
            screen_buffer: [0; 256 * 192],
            sprites: [Sprite {
                x: 0,
                y: 0,
                pattern: 0,
                color: 0,
                collision: false,
            }; 8],
            frame: 0,
            line: 0,
            vblank: false,
        }
    }

    fn read_vram(&mut self) -> u8 {
        let data = self.vram[self.address as usize];
        self.address = self.address.wrapping_add(1);
        self.latch = false;
        data
    }

    fn write_vram(&mut self, data: u8) {
        self.vram[self.address as usize] = data;
        self.address = self.address.wrapping_add(1);
        self.latch = false;
    }

    fn read_register(&mut self) -> u8 {
        let data = self.status;
        // TODO: m_StatusReg = m_FifthSprite;
        // TODO: check_interrupt();
        self.latch = false;
        data
    }

    fn write_register(&mut self, data: u8) {
        if self.latch {
            self.command = data;
            // On V9918, the VRAM pointer low gets written right away
            self.address = (self.address & 0xFF00) | data as u16;
            self.latch = false;
        } else {
            if data & 0x80 == 0 {
                // Set register
                let reg = data & 0x07;
                self.registers[reg as usize] = self.command;
                // On V9918, the VRAM pointer high gets also written when writing to registers
                self.address = (self.address & 0x00FF) | ((self.command as u16 & 0x03F) << 8);
            } else {
                // Set VRAM pointer
                self.address = self.address | ((data & 0x3F) as u16) | self.command as u16;
                // Pre-read VRAM if "writemode = 0"
                if (data & 0x40) == 0 {
                    self.status = self.vram[self.address as usize];
                    self.address = self.address.wrapping_add(1);
                }
            }
            self.latch = false;
        }
    }
}

impl IoDevice for TMS9918 {
    fn is_valid_port(&self, port: u8) -> bool {
        matches!(port, 0x98 | 0x99)
    }

    fn read(&mut self, port: u8) -> u8 {
        match port {
            // VRAM Read
            0x98 => self.read_vram(),
            // Register read
            0x99 => self.read_register(),
            _ => {
                error!("Invalid port: {:02X}", port);
                0xFF
            }
        }
    }

    fn write(&mut self, port: u8, data: u8) {
        // writing to data port 0x98
        match port {
            0x98 => self.write_vram(data),
            0x99 => self.write_register(data),
            _ => {
                error!("Invalid port: {:02X}", port);
            }
        }
    }
}
