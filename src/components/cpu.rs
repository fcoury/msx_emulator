use std::{cell::RefCell, fmt, rc::Rc};

use tracing::{error, info, trace};

use crate::internal_state::InternalState;

use super::{memory::Memory, IoDevice};

// static constexpr byte S_FLAG = 0x80;
// static constexpr byte Z_FLAG = 0x40;
// static constexpr byte Y_FLAG = 0x20;
// static constexpr byte H_FLAG = 0x10;
// static constexpr byte X_FLAG = 0x08;
// static constexpr byte V_FLAG = 0x04;
// static constexpr byte P_FLAG = V_FLAG;
// static constexpr byte N_FLAG = 0x02;
// static constexpr byte C_FLAG = 0x01;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flag {
    S = 0x80, // Sign
    Z = 0x40, // Zero
    H = 0x10, // Half Carry
    P = 0x04, // Parity/Overflow
    N = 0x02, // Add/Subtract
    C = 0x01, // Carry
}

pub struct Z80 {
    // 8-bit registers
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Alternate registers
    pub b_alt: u8,
    pub c_alt: u8,
    pub d_alt: u8,
    pub e_alt: u8,
    pub h_alt: u8,
    pub l_alt: u8,

    // 16-bit registers
    pub sp: u16,
    pub pc: u16,
    pub ix: u16,
    pub iy: u16,

    // Interrupt flip-flops
    pub iff1: bool,
    pub iff2: bool,

    // Interrupt mode
    pub im: u8,
    interrupt_request: bool,

    // Halted?
    pub halted: bool,

    // Memory
    pub memory: Memory,

    // I/O Devices
    io_devices: Vec<Rc<RefCell<dyn IoDevice + 'static>>>,

    // Debug options
    pub max_cycles: Option<u64>,
    pub track_flags: bool,
    cycles: u64,
    last_f: u8,
}

impl fmt::Display for Z80 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = format!(
            "S: {} Z: {} H: {} P/V: {} N: {} C: {}",
            if self.f & 0b1000_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0100_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0010_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0001_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0000_1000 != 0 { "1" } else { "0" },
            if self.f & 0b0000_0100 != 0 { "1" } else { "0" },
        );
        write!(
            f,
            "#{:04X} - A: #{:02X} B: #{:02X} C: #{:02X} D: #{:02X} E: #{:02X} F: #{:02X} H: #{:02X} L: #{:02X} - {}",
            self.pc, self.a, self.b, self.c, self.d, self.e, self.f, self.h, self.l, flags
        )
    }
}

impl Z80 {
    pub fn new(memory: Memory) -> Self {
        let io_devices: Vec<Rc<RefCell<dyn IoDevice + 'static>>> = vec![];
        Z80 {
            a: 0xff,
            f: 0xff,
            b: 0xff,
            c: 0xff,
            d: 0xff,
            e: 0xff,
            h: 0xff,
            l: 0xff,
            b_alt: 0,
            c_alt: 0,
            d_alt: 0,
            e_alt: 0,
            h_alt: 0,
            l_alt: 0,
            sp: 0xFFFF,
            pc: 0,
            ix: 0,
            iy: 0,
            iff1: false,
            iff2: false,
            im: 0,
            interrupt_request: false,
            memory,
            io_devices,
            halted: false,
            max_cycles: None,
            track_flags: false,
            cycles: 0,
            last_f: 0,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0xff;
        self.f = 0xff;
        self.b = 0xff;
        self.c = 0xff;
        self.d = 0xff;
        self.e = 0xff;
        self.h = 0xff;
        self.l = 0xff;
        self.b_alt = 0;
        self.c_alt = 0;
        self.d_alt = 0;
        self.e_alt = 0;
        self.h_alt = 0;
        self.l_alt = 0;
        self.sp = 0xF000;
        self.pc = 0;
        self.ix = 0;
        self.iy = 0;
        self.iff1 = false;
        self.iff2 = false;
        self.im = 0;
    }

    pub fn get_internal_state(&self) -> InternalState {
        InternalState {
            a: self.a,
            f: self.f,
            b: self.b,
            c: self.c,
            d: self.d,
            e: self.e,
            h: self.h,
            l: self.l,
            sp: self.sp,
            pc: self.pc,
            hl: self.get_hl(),
            hl_contents: self.read_byte(self.get_hl()),
            opcode: self.read_byte(self.pc),
        }
    }

    pub fn register_device(&mut self, device: Rc<RefCell<dyn IoDevice>>) {
        self.io_devices.push(device);
    }

    #[allow(dead_code)]
    pub fn request_interrupt(&mut self) {
        self.interrupt_request = true;
    }

    pub fn execute_cycle(&mut self) {
        self.cycles += 1;
        if self.halted {
            return;
        }

        // Check if we reached max_cycles
        if let Some(max_cycles) = self.max_cycles {
            if self.cycles >= max_cycles {
                panic!("Reached {} cycles", max_cycles);
            }
        }

        if self.interrupt_request && self.iff1 {
            self.interrupt_request = false;
            self.iff1 = false;
            self.push(self.pc);
            self.pc = 0x0038; // Jump to interrupt service routine at address 0x0038
            return;
        }

        // Fetch and decode the next instruction
        let opcode = self.memory.read_byte(self.pc);
        // info!("PC: 0x{:04X} Opcode: 0x{:02X}", self.pc, opcode);
        // trace!(
        //     "A: 0x{:02X} B: 0x{:02X} C: 0x{:02X} F: 0b{:b}",
        //     self.a,
        //     self.b,
        //     self.c,
        //     self.f
        // );

        // Execute the instruction
        match opcode {
            0x00 => self.nop(),
            0xD7 => {
                // RST 0x10
                trace!("RST");
                self.rst(0x10);
            }
            0xE7 => {
                // RST 20H
                trace!("RST 20H");
                self.rst(0x20);
            }
            0xFF => {
                // RST 20H
                error!("RST 38H from PC=0x{:04X}", self.pc);
                self.rst(0x38);
            }
            0x3E => {
                // LD A, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD A, 0x{:02X}", value);
                self.a = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x06 => {
                // LD B, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD B, 0x{:02X}", value);
                self.b = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x0E => {
                // LD C, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD C, 0x{:02X}", value);
                self.c = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x16 => {
                // LD D, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD D, 0x{:02X}", value);
                self.d = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x56 => {
                // LD D, (HL)
                trace!("LD D, (HL)");
                self.d = self.read_byte(self.get_hl());
                self.pc = self.pc.wrapping_add(1);
            }
            0x66 => {
                // LD H, (HL)
                trace!("LD H, (HL)");
                self.h = self.read_byte(self.get_hl());
                self.pc = self.pc.wrapping_add(1);
            }
            0x5E => {
                // LD E, (HL)
                trace!("LD E, (HL)");
                self.e = self.read_byte(self.get_hl());
                self.pc = self.pc.wrapping_add(1);
            }
            0x1E => {
                // LD E, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD E, 0x{:02X}", value);
                self.e = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x26 => {
                // LD H, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD H, 0x{:02X}", value);
                self.h = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x2E => {
                // LD L, n
                trace!("LD L, n");
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD L, 0x{:02X}", value);
                self.l = value;
                self.pc = self.pc.wrapping_add(1);
            }
            0x78 => {
                // LD A, B
                trace!("LD A, B");
                self.pc = self.pc.wrapping_add(1);
                self.a = self.b;
            }
            0x79 => {
                // LD A, C
                trace!("LD A, C");
                self.pc = self.pc.wrapping_add(1);
                self.a = self.c;
            }
            0x7A => {
                // LD A, D
                self.pc = self.pc.wrapping_add(1);
                self.a = self.d;
            }
            0x7B => {
                // LD A, E
                self.pc = self.pc.wrapping_add(1);
                self.a = self.e;
            }
            0x7C => {
                // LD A, H
                self.pc = self.pc.wrapping_add(1);
                self.a = self.h;
            }
            0x7D => {
                // LD A, L
                self.pc = self.pc.wrapping_add(1);
                self.a = self.l;
            }
            0x47 => {
                // LD B, A
                trace!("LD B, A");
                self.pc = self.pc.wrapping_add(1);
                self.b = self.a;
            }
            0x41 => {
                // LD B, C
                self.pc = self.pc.wrapping_add(1);
                self.b = self.c;
            }
            0x42 => {
                // LD B, D
                self.pc = self.pc.wrapping_add(1);
                self.b = self.d;
            }
            0x43 => {
                // LD B, E
                self.pc = self.pc.wrapping_add(1);
                self.b = self.e;
            }
            0x44 => {
                // LD B, H
                self.pc = self.pc.wrapping_add(1);
                self.b = self.h;
            }
            0x45 => {
                // LD B, L
                self.pc = self.pc.wrapping_add(1);
                self.b = self.l;
            }
            0x4F => {
                // LD C, A
                trace!("LD C, A");
                self.pc = self.pc.wrapping_add(1);
                self.c = self.a;
            }
            0x48 => {
                // LD C, B
                self.pc = self.pc.wrapping_add(1);
                self.c = self.b;
            }
            0x4A => {
                // LD C, D
                self.pc = self.pc.wrapping_add(1);
                self.c = self.d;
            }
            0x4B => {
                // LD C, E
                self.pc = self.pc.wrapping_add(1);
                self.c = self.e;
            }
            0x4C => {
                // LD C, H
                self.pc = self.pc.wrapping_add(1);
                self.c = self.h;
            }
            0x4D => {
                // LD C, L
                self.pc = self.pc.wrapping_add(1);
                self.c = self.l;
            }
            0x57 => {
                // LD D, A
                self.pc = self.pc.wrapping_add(1);
                self.d = self.a;
            }
            0x50 => {
                // LD D, B
                self.pc = self.pc.wrapping_add(1);
                self.d = self.b;
            }
            0x51 => {
                // LD D, C
                self.pc = self.pc.wrapping_add(1);
                self.d = self.c;
            }
            0x53 => {
                // LD D, E
                self.pc = self.pc.wrapping_add(1);
                self.d = self.e;
            }
            0x54 => {
                // LD D, H
                self.pc = self.pc.wrapping_add(1);
                self.d = self.h;
            }
            0x55 => {
                // LD D, L
                self.pc = self.pc.wrapping_add(1);
                self.d = self.l;
            }
            0x5F => {
                // LD E, A
                self.pc = self.pc.wrapping_add(1);
                self.e = self.a;
            }
            0x58 => {
                // LD E, B
                self.pc = self.pc.wrapping_add(1);
                self.e = self.b;
            }
            0x59 => {
                // LD E, C
                self.pc = self.pc.wrapping_add(1);
                self.e = self.c;
            }
            0x5A => {
                // LD E, D
                self.pc = self.pc.wrapping_add(1);
                self.e = self.d;
            }
            0x5C => {
                // LD E, H
                self.pc = self.pc.wrapping_add(1);
                self.e = self.h;
            }
            0x5D => {
                // LD E, L
                self.pc = self.pc.wrapping_add(1);
                self.e = self.l;
            }
            0x67 => {
                // LD H, A
                self.pc = self.pc.wrapping_add(1);
                self.h = self.a;
            }
            0x60 => {
                // LD H, B
                self.pc = self.pc.wrapping_add(1);
                self.h = self.b;
            }
            0x61 => {
                // LD H, C
                self.pc = self.pc.wrapping_add(1);
                self.h = self.c;
            }
            0x62 => {
                // LD H, D
                self.pc = self.pc.wrapping_add(1);
                self.h = self.d;
            }
            0x63 => {
                // LD H, E
                self.pc = self.pc.wrapping_add(1);
                self.h = self.e;
            }
            0x65 => {
                // LD H, L
                self.pc = self.pc.wrapping_add(1);
                self.h = self.l;
            }
            0x6F => {
                // LD L, A
                self.pc = self.pc.wrapping_add(1);
                self.l = self.a;
            }
            0x68 => {
                // LD L, B
                self.pc = self.pc.wrapping_add(1);
                self.l = self.b;
            }
            0x69 => {
                // LD L, C
                self.pc = self.pc.wrapping_add(1);
                self.l = self.c;
            }
            0x6A => {
                // LD L, D
                self.pc = self.pc.wrapping_add(1);
                self.l = self.d;
            }
            0x6B => {
                // LD L, E
                self.pc = self.pc.wrapping_add(1);
                self.l = self.e;
            }
            0x6C => {
                // LD L, H
                self.pc = self.pc.wrapping_add(1);
                self.l = self.h;
            }
            0x77 => {
                // LD (HL), A
                self.pc = self.pc.wrapping_add(1);
                // trace!("LD (HL), A -> A before = 0x{:02X}", self.a);
                self.ld_hl_a();
                // trace!("           -> HL = 0x{:04X}", self.get_hl());
                // trace!("           -> HL = 0x{:04X}", self.get_hl());
            }
            0x70 => {
                // LD (HL), B
                self.pc = self.pc.wrapping_add(1);
                self.ld_hl_b();
            }
            0x71 => {
                // LD (HL), C
                self.pc = self.pc.wrapping_add(1);
                self.ld_hl_c();
            }
            0x72 => {
                // LD (HL), D
                self.pc = self.pc.wrapping_add(1);
                self.ld_hl_d();
            }
            0x73 => {
                // LD (HL), E
                self.pc = self.pc.wrapping_add(1);
                self.ld_hl_e();
            }
            0x74 => {
                // LD (HL), H
                self.pc = self.pc.wrapping_add(1);
                self.ld_hl_h();
            }
            0x75 => {
                // LD (HL), L
                self.pc = self.pc.wrapping_add(1);
                self.ld_hl_l();
            }
            0x36 => {
                // LD (HL), n
                let value = self.read_byte(self.pc.wrapping_add(1));
                let hl_address = self.get_hl();

                trace!("LD (HL), 0x{:02X}", value);

                self.memory.write_byte(hl_address, value);
                self.pc = self.pc.wrapping_add(2);
            }
            0x21 => {
                // LD (HL), nn
                let low_byte = self.read_byte(self.pc.wrapping_add(1));
                let high_byte = self.read_byte(self.pc.wrapping_add(2));
                let value = u16::from_le_bytes([low_byte, high_byte]);

                trace!("LD HL, 0x{:04X}", value);

                self.set_hl(value);
                self.pc = self.pc.wrapping_add(3);
            }
            0x2A => {
                // LD HL, (nn)
                let low_byte = self.memory.read_byte(self.pc.wrapping_add(1));
                let high_byte = self.memory.read_byte(self.pc.wrapping_add(2));
                let addr = u16::from_le_bytes([low_byte, high_byte]);
                let l = self.memory.read_byte(addr);
                let h = self.memory.read_byte(addr.wrapping_add(1));
                self.set_hl(u16::from_le_bytes([l, h]));
                self.pc = self.pc.wrapping_add(3);
                trace!("LD HL, (nn)");
            }
            0xF9 => {
                // LD SP, HL
                trace!("LD SP, HL");
                self.pc = self.pc.wrapping_add(1);
                self.sp = self.get_hl();
            }
            0x31 => {
                // LD SP, nn
                let low_byte = self.read_byte(self.pc.wrapping_add(1));
                let high_byte = self.read_byte(self.pc.wrapping_add(2));
                let value = u16::from_le_bytes([low_byte, high_byte]);

                trace!("LD SP, 0x{:04X}", value);

                self.sp = value;
                self.pc = self.pc.wrapping_add(3);
            }
            0x0A => {
                // LD A, (BC)
                self.pc = self.pc.wrapping_add(1);
                self.ld_a_bc();
            }
            0x1A => {
                // LD A, (DE)
                self.pc = self.pc.wrapping_add(1);
                self.ld_a_de();
            }
            0x3A => {
                // LD A, (nn)
                trace!("LD A, (nn)");
                let low_byte = self.memory.read_byte(self.pc.wrapping_add(1));
                let high_byte = self.memory.read_byte(self.pc.wrapping_add(2));
                let address = ((high_byte as u16) << 8) | (low_byte as u16);
                self.a = self.memory.read_byte(address);

                self.pc = self.pc.wrapping_add(3);
            }
            0x7E => {
                // LD A, (HL)
                self.pc = self.pc.wrapping_add(1);
                trace!(
                    "LD A, (HL) -> 0. A = 0x{:02X}, HL = 0x{:04X}, (HL) = 0x{:02X}",
                    self.a,
                    self.get_hl(),
                    self.memory.read_byte(self.get_hl())
                );
                self.ld_a_hl();
                trace!("           -> 1. A = 0x{:02X}", self.a);
            }
            0x01 => {
                // LD BC, nn
                self.pc = self.pc.wrapping_add(1);
                let nn = self.read_word(self.pc);
                trace!("LD BC, 0x{:04X}", nn);
                self.set_bc(nn);
                self.pc = self.pc.wrapping_add(2);
            }
            0x11 => {
                // LD DE, nn
                self.pc = self.pc.wrapping_add(1);
                let nn = self.read_word(self.pc);
                self.pc = self.pc.wrapping_add(1);
                trace!("LD DE, 0x{:04X}", nn);
                self.set_de(nn);
                self.pc = self.pc.wrapping_add(1);
            }
            0x12 => {
                // LD DE, A
                trace!("LD DE, A");
                self.ld_de_a();
            }
            0x32 => {
                // LD (nn), A
                let address = self.read_word(self.pc.wrapping_add(1));
                trace!("LD (0x{:04X}), A", address);
                self.memory.write_byte(address, self.a);
                self.pc = self.pc.wrapping_add(3);
            }
            0x22 => {
                // LD (nn), HL
                let address = self.read_word(self.pc.wrapping_add(1));
                trace!("LD (0x{:04X}), HL", address);
                self.memory.write_word(address, self.get_hl());
                self.pc = self.pc.wrapping_add(3);
            }
            0x10 => {
                // DJNZ n
                let displacement = self.memory.read_signed_byte(self.pc.wrapping_add(1)) + 2;
                self.b = self.b.wrapping_sub(1);

                if self.b != 0 {
                    let jump_addr = self.pc.wrapping_add(displacement as u16);
                    self.pc = jump_addr;
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            0x3C => {
                // INC A
                trace!("INC A");
                self.pc = self.pc.wrapping_add(1);
                self.a = self.a.wrapping_add(1);
                self.set_inc_flags(self.a);
            }
            0x04 => {
                // INC B
                self.pc = self.pc.wrapping_add(1);
                self.b = self.b.wrapping_add(1);
                self.set_inc_flags(self.b);
            }
            0x0C => {
                // INC C
                self.pc = self.pc.wrapping_add(1);
                self.c = self.c.wrapping_add(1);
                self.set_inc_flags(self.c);
            }
            0x14 => {
                // INC D
                self.pc = self.pc.wrapping_add(1);
                self.d = self.d.wrapping_add(1);
                self.set_inc_flags(self.d);
            }
            0x1C => {
                // INC E
                self.pc = self.pc.wrapping_add(1);
                self.e = self.e.wrapping_add(1);
                self.set_inc_flags(self.e);
            }
            0x13 => {
                // INC DE
                let de = self.get_de();
                self.set_de(de.wrapping_add(1));
                self.pc = self.pc.wrapping_add(1);
            }
            0x23 => {
                // INC HL
                let hl = self.get_hl();
                let result = hl.wrapping_add(1);

                self.set_hl(result);
                self.pc = self.pc.wrapping_add(1);
            }
            0x24 => {
                // INC H
                self.pc = self.pc.wrapping_add(1);
                self.h = self.h.wrapping_add(1);
                self.set_inc_flags(self.h);
            }
            0x2C => {
                // INC L
                self.pc = self.pc.wrapping_add(1);
                self.l = self.l.wrapping_add(1);
                self.set_inc_flags(self.l);
            }
            0x34 => {
                // INC (HL)
                self.pc = self.pc.wrapping_add(1);
                self.inc_hl();
            }
            0x3D => {
                // DEC A
                self.pc = self.pc.wrapping_add(1);
                self.a = self.a.wrapping_sub(1);
                self.set_dec_flags(self.a);
            }
            0x05 => {
                // DEC B
                self.pc = self.pc.wrapping_add(1);
                self.b = self.b.wrapping_sub(1);
                self.set_dec_flags(self.b);
            }
            0x0D => {
                // DEC C
                self.pc = self.pc.wrapping_add(1);
                self.c = self.c.wrapping_sub(1);
                self.set_dec_flags(self.c);
            }
            0x15 => {
                // DEC D
                self.pc = self.pc.wrapping_add(1);
                self.d = self.d.wrapping_sub(1);
                self.set_dec_flags(self.d);
            }
            0x1D => {
                // DEC E
                self.pc = self.pc.wrapping_add(1);
                self.e = self.e.wrapping_sub(1);
                self.set_dec_flags(self.e);
            }
            0x25 => {
                // DEC H
                trace!("DEC H");
                self.pc = self.pc.wrapping_add(1);
                self.h = self.h.wrapping_sub(1);
                self.set_dec_flags(self.h);
            }
            0x2D => {
                // DEC L
                self.pc = self.pc.wrapping_add(1);
                self.l = self.l.wrapping_sub(1);
                self.set_dec_flags(self.l);
            }
            0x2B => {
                // DEC HL
                let hl = self.get_hl();
                self.set_hl(hl.wrapping_sub(1));
                self.pc = self.pc.wrapping_add(1);
            }
            0x0B => {
                // DEC BC
                let bc = self.get_bc();
                self.set_bc(bc.wrapping_sub(1));
                self.pc = self.pc.wrapping_add(1);
            }
            0x35 => {
                // DEC (HL)
                self.pc = self.pc.wrapping_add(1);
                self.dec_hl();
            }
            0x87 => {
                // ADD A, A
                trace!("ADD A, A");
                self.add_a(self.a);
                self.pc = self.pc.wrapping_add(1);
            }
            0x80 => {
                // ADD A, B
                trace!("ADD A, B");
                self.add_a(self.b);
                self.pc = self.pc.wrapping_add(1);
            }
            0x81 => {
                // ADD A, C
                self.add_a(self.c);
                self.pc = self.pc.wrapping_add(1);
            }
            0x82 => {
                // ADD A, D
                self.add_a(self.d);
                self.pc = self.pc.wrapping_add(1);
            }
            0x83 => {
                // ADD A, E
                self.add_a(self.e);
                self.pc = self.pc.wrapping_add(1);
            }
            0x84 => {
                // ADD A, H
                self.add_a(self.h);
                self.pc = self.pc.wrapping_add(1);
            }
            0x85 => {
                // ADD A, L
                trace!("ADD A, L");
                self.add_a(self.l);
                self.pc = self.pc.wrapping_add(1);
            }
            0x86 => {
                // ADD A, (HL)
                trace!("ADD A, (HL)");
                let hl_address = self.get_hl();
                let value = self.read_byte(hl_address);
                self.add_a(value);
                self.pc = self.pc.wrapping_add(1);
            }
            0xC6 => {
                // ADD A, n
                trace!("ADD A, n");
                let immediate_value = self.read_byte(self.pc.wrapping_add(1));
                self.add_a(immediate_value);
                self.pc = self.pc.wrapping_add(2);
            }
            0x09 => {
                // ADD HL, BC
                let hl = self.get_hl();
                let bc = self.get_bc();
                let (result, carry) = hl.overflowing_add(bc);

                self.set_hl(result);
                self.set_flag(Flag::H, (hl & 0x0FFF) + (bc & 0x0FFF) > 0x0FFF);
                self.set_flag(Flag::C, carry);
                self.set_flag(Flag::N, false);
                self.pc = self.pc.wrapping_add(1);
                trace!("ADD HL, BC");
            }
            0x19 => {
                // ADD HL, DE
                let hl = self.get_hl();
                let de = self.get_de();
                let (result, carry) = hl.overflowing_add(de);

                self.set_hl(result);
                self.set_flag(Flag::H, (hl & 0x0FFF) + (de & 0x0FFF) > 0x0FFF);
                self.set_flag(Flag::C, carry);
                self.set_flag(Flag::N, false);
                self.pc = self.pc.wrapping_add(1);
                trace!("ADD HL, DE");
            }
            0x29 => {
                // ADD HL, HL
                let hl = self.get_hl();
                let (result, carry) = hl.overflowing_add(hl);

                self.set_hl(result);
                self.set_flag(Flag::H, (hl & 0x0FFF) + (hl & 0x0FFF) > 0x0FFF);
                self.set_flag(Flag::C, carry);
                self.set_flag(Flag::N, false);
                self.pc = self.pc.wrapping_add(1);
                trace!("ADD HL, HL");
            }
            0x39 => {
                // ADD HL, SP
                trace!("ADD HL, SP");
                let hl = self.get_hl();
                let result = hl.wrapping_add(self.sp);

                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, (hl & 0x0FFF) + (self.sp & 0x0FFF) > 0x0FFF);
                self.set_flag(Flag::C, result < hl);

                self.set_hl(result);
                self.pc = self.pc.wrapping_add(1);
            }
            0x8F => {
                // ADC A, A
                trace!("ADC A, A");
                self.adc_a(self.a);
                self.pc = self.pc.wrapping_add(1);
            }
            0x88 => {
                // ADC A, B
                trace!("ADC A, B");
                self.adc_a(self.a);
                self.pc = self.pc.wrapping_add(1);
            }
            0x89 => {
                // ADC A, C
                trace!("ADC A, C");
                self.adc_a(self.c);
                self.pc = self.pc.wrapping_add(1);
            }
            0x8A => {
                // ADC A, D
                trace!("ADC A, D");
                self.adc_a(self.d);
                self.pc = self.pc.wrapping_add(1);
            }
            0x8B => {
                // ADC A, E
                trace!("ADC A, E");
                self.adc_a(self.e);
                self.pc = self.pc.wrapping_add(1);
            }
            0x8C => {
                // ADC A, H
                trace!("ADC A, H");
                self.adc_a(self.h);
                self.pc = self.pc.wrapping_add(1);
            }
            0x8D => {
                // ADC A, L
                trace!("ADC A, L");
                self.adc_a(self.l);
                self.pc = self.pc.wrapping_add(1);
            }
            0x8E => {
                // ADC A, (HL)
                trace!("ADC A, (HL)");
                let hl_address = self.get_hl();
                let value = self.read_byte(hl_address);
                self.adc_a(value);
                self.pc = self.pc.wrapping_add(1);
            }
            0xCE => {
                // ADC A, n
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                let result = self.a.wrapping_add(value);
                self.a = result;
            }
            0x97 => {
                // SUB A
                trace!("SUB A");
                self.sub_a(self.a);
                self.pc = self.pc.wrapping_add(1);
            }
            0x90 => {
                // SUB B
                trace!("SUB B");
                self.sub_a(self.b);
                self.pc = self.pc.wrapping_add(1);
            }
            0x91 => {
                // SUB C
                trace!("SUB C");
                self.sub_a(self.c);
                self.pc = self.pc.wrapping_add(1);
            }
            0x92 => {
                // SUB D
                trace!("SUB D");
                self.sub_a(self.d);
                self.pc = self.pc.wrapping_add(1);
            }
            0x93 => {
                // SUB E
                trace!("SUB E");
                self.sub_a(self.e);
                self.pc = self.pc.wrapping_add(1);
            }
            0x94 => {
                // SUB H
                trace!("SUB H");
                self.sub_a(self.h);
                self.pc = self.pc.wrapping_add(1);
            }
            0x95 => {
                // SUB H
                trace!("SUB H");
                self.sub_a(self.h);
                self.pc = self.pc.wrapping_add(1);
            }
            0x96 => {
                // SUB (HL)
                trace!("SUB (HL)");
                let hl_address = self.get_hl();
                let value = self.read_byte(hl_address);
                self.sub_a(value);
                self.pc = self.pc.wrapping_add(1);
            }
            0xD6 => {
                // SUB n
                trace!("SUB n");
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                info!("SUB n with value: {:#X}", value);
                self.pc = self.pc.wrapping_add(2);
                self.sub_a(value);
            }
            0x9F => {
                // SBC A, A
                trace!("SBC A, A");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.a);
            }
            0x98 => {
                // SBC A, B
                trace!("SBC A, B");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.b);
            }
            0x99 => {
                // SBC A, C
                trace!("SBC A, C");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.c);
            }
            0x9A => {
                // SBC A, D
                trace!("SBC A, D");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.d);
            }
            0x9B => {
                // SBC A, E
                trace!("SBC A, E");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.e);
            }
            0x9C => {
                // SBC A, H
                trace!("SBC A, H");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.h);
            }
            0x9D => {
                // SBC A, L
                trace!("SBC A, L");
                self.pc = self.pc.wrapping_add(1);
                self.sbc_a(self.l);
            }
            0x9E => {
                // SBC A, (HL)
                trace!("SBC A, (HL)");
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.get_hl());
                self.sbc_a(value);
            }
            0xDE => {
                // SBC A, n
                trace!("SBC A, n");
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                self.sbc_a(value);
            }
            0xA7 => {
                // AND A
                trace!("AND A");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.a);
            }
            0xA0 => {
                // AND B
                trace!("AND B");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.b);
            }
            0xA1 => {
                // AND C
                trace!("AND C");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.c);
            }
            0xA2 => {
                // AND D
                trace!("AND D");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.d);
            }
            0xA3 => {
                // AND E
                trace!("AND E");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.e);
            }
            0xA4 => {
                // AND H
                trace!("AND H");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.h);
            }
            0xA5 => {
                // AND L
                trace!("AND L");
                self.pc = self.pc.wrapping_add(1);
                self.and_a(self.l);
            }
            0xA6 => {
                // AND (HL)
                trace!("AND (HL)");
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.get_hl());
                self.and_a(value);
            }
            0xE6 => {
                // AND n
                trace!("AND n");
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                self.and_a(value);
            }
            0xB7 => {
                // OR A
                self.pc = self.pc.wrapping_add(1);
                self.set_flag(Flag::Z, self.a == 0);
                self.set_flag(Flag::S, self.a & 0x80 != 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::P, parity(self.a));
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::C, false);
            }
            0x07 => {
                // RLCA
                trace!("RLCA");
                let msb = self.a & 0x80;
                let carry = msb != 0;

                self.a = (self.a << 1) | (msb >> 7);
                self.set_flag(Flag::C, carry);

                self.pc = self.pc.wrapping_add(1);
            }
            0xB0 => {
                // OR B
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.b;
                self.a = result;
            }
            0xB1 => {
                // OR C
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.c;
                self.a = result;
            }
            0xB2 => {
                // OR D
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.d;
                self.a = result;
            }
            0xB3 => {
                // OR E
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.e;
                self.a = result;
            }
            0xB4 => {
                // OR H
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.h;
                self.a = result;
            }
            0xB5 => {
                // OR L
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.l;
                self.a = result;
            }
            0xB6 => {
                // OR (HL)
                trace!("OR (HL)");
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.get_hl());
                self.or_a(value);
            }
            0xF6 => {
                // OR n
                trace!("OR n");
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                self.or_a(value);
            }
            0xAF => {
                // XOR A
                trace!("XOR A");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.a);
            }
            0xA8 => {
                // XOR B
                trace!("XOR B");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.b);
            }
            0xA9 => {
                // XOR C
                trace!("XOR C");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.c);
            }
            0xAA => {
                // XOR D
                trace!("XOR D");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.d);
            }
            0xAB => {
                // XOR E
                trace!("XOR E");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.e);
            }
            0xAC => {
                // XOR H
                trace!("XOR H");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.h);
            }
            0xAD => {
                // XOR L
                trace!("XOR L");
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(self.l);
            }
            0xAE => {
                // XOR (HL)
                trace!("XOR (HL)");
                let value = self.memory.read_byte(self.get_hl());
                self.pc = self.pc.wrapping_add(1);
                self.xor_a(value);
            }
            0xEE => {
                // XOR n
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                let result = self.a ^ value;
                self.a = result;
            }
            0x18 => {
                // JR e
                self.pc = self.pc.wrapping_add(1);
                let offset = self.memory.read_byte(self.pc) as i8;
                self.pc = self.pc.wrapping_add(offset as u16 + 1);
                trace!("JR 0x{:04X}", self.pc);
            }
            0x76 => {
                // HALT
                trace!("HALT");
                self.pc = self.pc.wrapping_add(1);
                self.halted = true;
            }
            0x2F => {
                // CPL
                trace!("CPL -> 0. A = 0x{:02X}", self.a);
                self.a = !self.a;
                trace!("       1. A = 0x{:02X}", self.a);
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, true);
                self.pc = self.pc.wrapping_add(1);
            }
            0xBF => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.a);
            }
            0xB8 => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.b);
            }
            0xB9 => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.c);
            }
            0xBA => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.d);
            }
            0xBB => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.e);
            }
            0xBC => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.h);
            }
            0xBD => {
                self.pc = self.pc.wrapping_add(1);
                self.cp(self.l);
            }
            0xFE => {
                // CP n
                trace!("CP n");
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                self.cp(value);
            }
            0xBE => {
                // CP (HL)
                let value = self.memory.read_byte(self.get_hl());
                trace!(
                    "CP (HL) -> A = {:02X}, HL = {:04X}, (HL) = {:02X}",
                    self.a,
                    self.get_hl(),
                    value
                );
                self.cp(value);
                self.pc = self.pc.wrapping_add(1);
            }
            0xDD => {
                trace!("CP (IX+d)");
                self.pc = self.pc.wrapping_add(1);
                let opcode = self.memory.read_byte(self.pc);
                match opcode {
                    0xBE => {
                        self.pc = self.pc.wrapping_add(1);
                        let d = self.memory.read_byte(self.pc) as i8;
                        self.pc = self.pc.wrapping_add(1);
                        let value = self.memory.read_byte(self.get_ix_d(d as u8));
                        self.cp(value);
                    }
                    _ => {
                        panic!("Unknown opcode (CP (IX+d)) 0xDD 0x{:02X}", opcode);
                    }
                }
            }
            0xFD => {
                trace!("CP (IY+d)");
                self.pc = self.pc.wrapping_add(1);
                let opcode = self.memory.read_byte(self.pc);
                match opcode {
                    0xBE => {
                        self.pc = self.pc.wrapping_add(1);
                        let d = self.memory.read_byte(self.pc) as i8;
                        self.pc = self.pc.wrapping_add(1);
                        let value = self.memory.read_byte(self.get_iy_d(d as u8));
                        self.cp(value);
                    }
                    _ => {
                        panic!("Unknown opcode (CP (IY+d)) 0xDD 0x{:02X}", opcode);
                    }
                }
            }
            0x3F => {
                // CCF
                trace!("CCF");
                let carry = self.get_flag(Flag::C);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::C, !carry);
                self.pc = self.pc.wrapping_add(1);
            }
            0x37 => {
                // SCF
                trace!("SCF");
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::C, true);
                self.pc = self.pc.wrapping_add(1);
            }
            0xEB => {
                // EX DE, HL
                let de = self.get_de();
                let hl = self.get_hl();

                self.set_de(hl);
                self.set_hl(de);

                // Increment program counter
                self.pc = self.pc.wrapping_add(1);
            }
            0xE3 => {
                // EX (SP), HL
                let hl = self.get_hl();
                let value = self.memory.read_word(self.sp);

                self.memory.write_word(self.sp, hl);
                self.set_hl(value);

                self.pc = self.pc.wrapping_add(1);
                trace!("EX (SP), HL");
            }
            0xD9 => {
                // EXX
                trace!("EXX");
                std::mem::swap(&mut self.b, &mut self.b_alt);
                std::mem::swap(&mut self.c, &mut self.c_alt);
                std::mem::swap(&mut self.d, &mut self.d_alt);
                std::mem::swap(&mut self.e, &mut self.e_alt);
                std::mem::swap(&mut self.h, &mut self.h_alt);
                std::mem::swap(&mut self.l, &mut self.l_alt);

                self.pc = self.pc.wrapping_add(1);
            }
            0xCC => {
                // CALL Z, nn
                let address = self.memory.read_word(self.pc.wrapping_add(1));
                if self.get_flag(Flag::Z) {
                    self.push(self.pc.wrapping_add(3));
                    self.pc = address;
                } else {
                    self.pc = self.pc.wrapping_add(3);
                }
            }
            0xDC => {
                // CALL C, nn
                let address = self.memory.read_word(self.pc.wrapping_add(1));
                if self.get_flag(Flag::C) {
                    self.push(self.pc.wrapping_add(3));
                    self.pc = address;
                } else {
                    self.pc = self.pc.wrapping_add(3);
                }
            }
            0xCD => {
                // CALL nn

                // The current contents of the Program Counter (PC) are pushed onto the top of the external
                // memory stack. The operands nn are then loaded to the PC to point to the address in memory at which the first op code of a subroutine is to be fetched. At the end of the subroutine,
                // a RETurn instruction can be used to return to the original program flow by popping the top
                // of the stack back to the PC. The push is accomplished by first decrementing the current
                // contents of the Stack Pointer (register pair SP), loading the high-order byte of the PC contents to the memory address now pointed to by the SP; then decrementing SP again, and
                // loading the low-order byte of the PC contents to the top of stack.
                // Because this process is a 3-byte instruction, the Program Counter was incremented by
                // three before the push is executed.

                // self.pc = self.pc.wrapping_add(1);
                // let address = self.read_word(self.pc);
                // self.call(address);
                // CALL nn
                let low_byte = self.memory.read_byte(self.pc.wrapping_add(1));
                let high_byte = self.memory.read_byte(self.pc.wrapping_add(2));
                let target_address = u16::from_le_bytes([low_byte, high_byte]);

                // Push the current PC+3 (return address) onto the stack
                let return_address = self.pc.wrapping_add(3);
                let return_address_high = ((return_address >> 8) & 0xFF) as u8;
                let return_address_low = (return_address & 0xFF) as u8;

                self.sp = self.sp.wrapping_sub(1);
                self.memory.write_byte(self.sp, return_address_high);
                self.sp = self.sp.wrapping_sub(1);
                self.memory.write_byte(self.sp, return_address_low);

                // Jump to the target subroutine address
                self.pc = target_address;
                info!("#{:04X} - CALL {:04X}", self.pc, target_address);
            }
            0xC9 => {
                // RET
                self.ret();
            }
            0xC8 => {
                // RET Z
                if self.get_flag(Flag::Z) {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xD8 => {
                // RET C
                if self.get_flag(Flag::C) {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xC0 => {
                // RET NZ
                if !self.get_flag(Flag::Z) {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xD0 => {
                // RET NC
                if !self.get_flag(Flag::C) {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xF8 => {
                // RET M
                if self.get_flag(Flag::S) {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xF0 => {
                // RET P
                if !self.get_flag(Flag::S) {
                    let low_byte = self.memory.read_byte(self.sp);
                    self.sp = self.sp.wrapping_add(1);
                    let high_byte = self.memory.read_byte(self.sp);
                    self.sp = self.sp.wrapping_add(1);
                    self.pc = u16::from_le_bytes([low_byte, high_byte]);
                    trace!("RET P");
                } else {
                    self.pc = self.pc.wrapping_add(1);
                    trace!("NOP (RET P not taken)");
                }
            }
            0xC5 => {
                // PUSH BC
                trace!("PUSH BC");
                self.pc = self.pc.wrapping_add(1);
                self.push(self.get_bc());
            }
            0xD5 => {
                // PUSH DE
                trace!("PUSH DE");
                self.pc = self.pc.wrapping_add(1);
                self.push(self.get_de());
            }
            0xE5 => {
                // PUSH HL
                trace!("PUSH HL");
                self.pc = self.pc.wrapping_add(1);
                self.push(self.get_hl());
            }
            0xF5 => {
                // PUSH AF
                trace!("PUSH AF");
                self.pc = self.pc.wrapping_add(1);
                self.push(self.get_af());
            }

            0xC1 => {
                // POP BC
                trace!("POP BC");
                self.pc = self.pc.wrapping_add(1);
                let value = self.pop();
                self.set_bc(value);
            }
            0xD1 => {
                // POP DE
                self.pc = self.pc.wrapping_add(1);
                let value = self.pop();
                self.set_de(value);
            }
            0xE1 => {
                // POP HL
                self.pc = self.pc.wrapping_add(1);
                let value = self.pop();
                self.set_hl(value);
            }
            0xF1 => {
                // POP AF
                trace!("POP AF");
                self.pc = self.pc.wrapping_add(1);
                let value = self.pop();
                self.set_af(value);
            }
            0xF2 => {
                // JP P, nn
                let addr = self.read_word(self.pc.wrapping_add(1));
                if !self.get_flag(Flag::S) {
                    self.pc = addr;
                    trace!("JP P, 0x{:04X}", addr);
                } else {
                    self.pc = self.pc.wrapping_add(3);
                    trace!("JP P, 0x{:04X} (not taken)", addr);
                }
            }

            0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA | 0xFA => {
                // JP cc, nn
                let condition = match opcode {
                    0xC2 => !self.check_flag(Flag::Z), // JP NZ, nn
                    0xCA => self.check_flag(Flag::Z),  // JP Z, nn
                    0xD2 => !self.check_flag(Flag::C), // JP NC, nn
                    0xDA => self.check_flag(Flag::C),  // JP C, nn
                    0xFA => self.check_flag(Flag::S),  // JP M, nn
                    0xC3 => true,                      // JP nn (unconditional)
                    _ => unreachable!(),
                };

                let address = self.read_word(self.pc.wrapping_add(1));
                trace!(
                    "PC = {:04X} JP cc, 0x{:04X} = {}",
                    self.pc,
                    address,
                    condition
                );

                self.pc = self.pc.wrapping_add(3);

                if condition {
                    self.pc = address;
                }
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
                trace!(
                    "Flags for JS - Z={} C={}",
                    if self.check_flag(Flag::Z) { 1 } else { 0 },
                    if self.check_flag(Flag::C) { 1 } else { 0 }
                );

                if opcode == 0x28 {
                    info!(
                        "Flags for JS - Z={} C={}",
                        if self.check_flag(Flag::Z) { 1 } else { 0 },
                        if self.check_flag(Flag::C) { 1 } else { 0 }
                    );
                }

                // JR cc, n
                let condition = match opcode {
                    0x20 => !self.check_flag(Flag::Z), // JR NZ, n
                    0x28 => self.check_flag(Flag::Z),  // JR Z, n
                    0x30 => !self.check_flag(Flag::C), // JR NC, n
                    0x38 => self.check_flag(Flag::C),  // JR C, n
                    _ => unreachable!(),
                };

                let offset = self.read_byte(self.pc.wrapping_add(1)) as i8;
                self.pc = self.pc.wrapping_add(2);

                match opcode {
                    0x20 => trace!(
                        "JR NZ, 0x{:04X} = {}",
                        self.pc.wrapping_add(offset as u16),
                        condition
                    ),
                    0x28 => info!(
                        "JR Z, 0x{:04X} = {} (offset {})",
                        self.pc.wrapping_add(offset as u16),
                        condition,
                        offset
                    ),
                    0x30 => trace!(
                        "JR NC, 0x{:04X} = {}",
                        self.pc.wrapping_add(offset as u16),
                        condition
                    ),
                    0x38 => trace!(
                        "JR C, 0x{:04X} = {}",
                        self.pc.wrapping_add(offset as u16),
                        condition
                    ),
                    _ => unreachable!(),
                };

                if condition {
                    self.pc = (self.pc as i16 + offset as i16) as u16;
                }
            }
            0x0F => {
                // RRC A
                let a = self.a;
                let carry = a & 0x01 != 0;
                let result = (a >> 1) | ((carry as u8) << 7);

                self.a = result;
                self.set_flag(Flag::S, result & 0x80 != 0);
                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::P, result.count_ones() % 2 == 0);
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::C, carry);

                self.pc = self.pc.wrapping_add(1);
                trace!("RRC A");
            }
            0xCB => {
                // Read extended opcode and execute it
                let extended_opcode = self.read_byte(self.pc.wrapping_add(1));

                match extended_opcode {
                    0x00..=0x1F => {
                        // RLC r
                        let reg_index = extended_opcode & 0x07;

                        trace!("RLC {}", reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let carry = (value & 0x80) != 0;

                        let result = (value << 1) | (carry as u8);
                        self.set_register_by_index(reg_index, result);

                        self.set_flag(Flag::S, result & 0x80 != 0);
                        self.set_flag(Flag::Z, result == 0);
                        self.set_flag(Flag::H, false);
                        self.set_flag(Flag::P, result.count_ones() % 2 == 0);
                        self.set_flag(Flag::N, false);
                        self.set_flag(Flag::C, carry);

                        self.pc = self.pc.wrapping_add(2);
                    }
                    0x28..=0x2F => {
                        // SRA r
                        let reg_index = extended_opcode & 0x07;

                        trace!("SRA {}", reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let carry = (value & 0x01) != 0;

                        let result = (value >> 1) | (value & 0x80);
                        self.set_register_by_index(reg_index, result);

                        self.set_flag(Flag::S, result & 0x80 != 0);
                        self.set_flag(Flag::Z, result == 0);
                        self.set_flag(Flag::H, false);
                        self.set_flag(Flag::P, result.count_ones() % 2 == 0);
                        self.set_flag(Flag::N, false);
                        self.set_flag(Flag::C, carry);

                        self.pc = self.pc.wrapping_add(2);
                    }
                    0x20..=0x3F => {
                        // SLA r
                        let reg_index = extended_opcode & 0x07;

                        trace!("SLA {}", reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let carry = (value & 0x80) != 0;

                        let result = value << 1;
                        self.set_register_by_index(reg_index, result);

                        self.set_flag(Flag::S, result & 0x80 != 0);
                        self.set_flag(Flag::Z, result == 0);
                        self.set_flag(Flag::H, false);
                        self.set_flag(Flag::P, result.count_ones() % 2 == 0);
                        self.set_flag(Flag::N, false);
                        self.set_flag(Flag::C, carry);

                        self.pc = self.pc.wrapping_add(2);
                    }
                    0x40..=0x7F => {
                        // BIT b, r
                        let bit = (extended_opcode >> 3) & 0x07;
                        let reg_index = extended_opcode & 0x07;

                        trace!("BIT {}, {}", bit, reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let mask = 1 << bit;
                        let bit_value = value & mask;

                        self.set_flag(Flag::S, bit_value & 0x80 != 0);
                        self.set_flag(Flag::Z, bit_value == 0);
                        self.set_flag(Flag::H, true);
                        self.set_flag(Flag::P, bit_value == 0); // P/V flag is set to the inverse of the Z flag
                        self.set_flag(Flag::N, false);

                        self.pc = self.pc.wrapping_add(2);
                    }
                    0x80..=0xBF => {
                        // RES b, r
                        let bit = (extended_opcode >> 3) & 0x07;
                        let reg_index = extended_opcode & 0x07;

                        trace!("RES {}, {}", bit, reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let mask = !(1 << bit);

                        self.set_register_by_index(reg_index, value & mask);
                        self.pc = self.pc.wrapping_add(2);
                    }
                    0xC0..=0xFF => {
                        // SET b, r
                        let bit = (extended_opcode >> 3) & 0x07;
                        let reg_index = extended_opcode & 0x07;

                        trace!("SET {}, {}", bit, reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let mask = 1 << bit;

                        self.set_register_by_index(reg_index, value | mask);
                        self.pc = self.pc.wrapping_add(2);
                    }
                }
            }

            // I/O
            0xDB => {
                // IN A, (n)
                let port = self.read_byte(self.pc.wrapping_add(1));
                trace!("IN A, (0x{:02X})", port);

                for device in &self.io_devices {
                    if device.borrow().is_valid_port(port) {
                        let mut device_ref = device.as_ref().borrow_mut();
                        self.a = device_ref.read(port);
                        break;
                    }
                }

                self.pc = self.pc.wrapping_add(2);
            }
            0xD3 => {
                // OUT (n), A
                let port = self.read_byte(self.pc.wrapping_add(1));
                let data = self.a;
                error!(
                    "PC = #{:04X} OUT (#{:02X}), A=0x{:02X}",
                    self.pc, port, data
                );

                for device in &self.io_devices {
                    if device.borrow().is_valid_port(port) {
                        let mut device_ref = device.as_ref().borrow_mut();
                        device_ref.write(port, data);
                        break;
                    }
                }

                self.pc = self.pc.wrapping_add(2);
            }

            // Extended opcodes
            0xED => {
                self.pc = self.pc.wrapping_add(1);
                let extended_opcode = self.memory.read_byte(self.pc);

                match extended_opcode {
                    0xB0 => self.ldi(),
                    0x56 => {
                        self.im = 1;
                        self.pc = self.pc.wrapping_add(1);
                    }
                    0xA3 => {
                        self.e &= !(1 << 4);
                        self.pc = self.pc.wrapping_add(1);
                    }
                    // Add extended opcodes handling here
                    // 0x4A => self.sbc_hl(RegisterPair::BC),
                    // 0x5A => self.sbc_hl(RegisterPair::DE),
                    // 0x6A => self.sbc_hl(RegisterPair::HL),
                    // 0x7A => self.sbc_hl(RegisterPair::SP),
                    // ... (other opcodes)
                    _ => {
                        self.report_unknown("Unhandled extended opcode", opcode);
                    }
                }
            }

            // Interrupts
            // EI
            0xFB => {
                trace!("EI");
                self.pc = self.pc.wrapping_add(1);
                self.iff1 = true;
            }
            // DI
            0xF3 => {
                trace!("DI");
                self.pc = self.pc.wrapping_add(1);
                self.iff1 = false;
            }

            _ => {
                self.report_unknown("Unhandled opcode", opcode);
            }
        }

        if self.track_flags && self.f != self.last_f {
            trace!(
                " *** Flags updated -> before = {:08b}, after = {:08b} Z={}\n",
                self.last_f,
                self.f,
                self.check_flag(Flag::Z)
            );
            self.last_f = self.f;
        }
    }

    fn report_unknown(&self, message: &str, opcode: u8) {
        // let prev_10_bytes = self
        //     .memory
        //     .data
        //     .iter()
        //     .rev()
        //     .skip(self.memory.data.len() - self.pc as usize)
        //     .take(10)
        //     .map(|b| format!("{:02X}", b))
        //     .collect::<Vec<String>>()
        //     .join(" ");
        let next_10_bytes = self
            .memory
            .data
            .iter()
            .skip(self.pc as usize)
            .take(10)
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ");
        panic!(
            "{} at {:04X}: {:02X} -- lookahead: {}",
            message, self.pc, opcode, next_10_bytes
        );
    }

    fn add_a(&mut self, value: u8) {
        let a = self.a;
        let result = a.wrapping_add(value);

        self.set_flag(Flag::S, result & 0x80 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::H, (a & 0x0F) + (value & 0x0F) > 0x0F);
        self.set_flag(Flag::P, ((a ^ result) & !(a ^ value)) & 0x80 != 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::C, (a as u16) + (value as u16) > 0xFF);

        self.a = result;
    }

    fn adc_a(&mut self, value: u8) {
        let a = self.a;
        let carry = if self.get_flag(Flag::C) { 1 } else { 0 };
        let result = a.wrapping_add(value).wrapping_add(carry);

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, (a & 0x0F) + (value & 0x0F) + carry > 0x0F);
        self.set_flag(Flag::C, a > 0xFF - value - carry);

        self.a = result;
    }

    fn sub_a(&mut self, value: u8) {
        let a = self.a;
        let result = a.wrapping_sub(value);

        self.set_flag(Flag::S, result & 0x80 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::H, (a & 0x0F) < (value & 0x0F));
        self.set_flag(Flag::P, ((a ^ value) & (a ^ result)) & 0x80 != 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::C, a < value);

        self.a = result;
    }

    fn sbc_a(&mut self, value: u8) {
        let carry = if self.get_flag(Flag::C) { 1 } else { 0 };
        let result = self.a.wrapping_sub(value).wrapping_sub(carry);

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::S, result & 0x80 != 0);
        self.set_flag(Flag::H, (self.a & 0x0F) < (value & 0x0F) + carry);
        self.set_flag(Flag::P, (self.a as i8) >= 0 && ((result as i8) < 0));
        self.set_flag(Flag::N, true);
        self.set_flag(
            Flag::C,
            self.a < value.checked_add(carry).unwrap_or(u8::MAX),
        );

        self.a = result;
    }

    fn and_a(&mut self, value: u8) {
        self.a &= value;

        self.set_flag(Flag::Z, self.a == 0);
        self.set_flag(Flag::S, self.a & 0x80 != 0);
        self.set_flag(Flag::H, true);
        self.set_flag(Flag::P, parity(self.a));
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::C, false);
    }

    fn or_a(&mut self, value: u8) {
        self.a |= value;

        self.set_flag(Flag::Z, self.a == 0);
        self.set_flag(Flag::S, self.a & 0x80 != 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::P, parity(self.a));
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::C, false);
    }

    fn xor_a(&mut self, value: u8) {
        info!("XOR A, {:02X}", value);
        self.a ^= value;
        info!("Z[b] = {}", self.a == 0);
        self.set_flag(Flag::Z, self.a == 0);
        info!("Z[a] = {}", self.get_flag(Flag::Z));
        self.set_flag(Flag::S, self.a & 0x80 != 0);
        self.set_flag(Flag::H, false);
        self.set_flag(Flag::P, parity(self.a));
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::C, false);
    }

    fn cp(&mut self, value: u8) {
        let result = self.a.wrapping_sub(value);

        self.set_flag(Flag::S, result & 0x80 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::H, (self.a & 0xF) < (value & 0xF));
        self.set_flag(Flag::P, overflow_sub(self.a, value, result));
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::C, self.a < value);
    }

    // Helper function to set flags for INC
    fn set_inc_flags(&mut self, value: u8) {
        self.set_flag(Flag::S, value & 0x80 != 0);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, (value & 0x0F) == 0x00);
        self.set_flag(Flag::P, value == 0x80);
        self.set_flag(Flag::N, false);
    }

    // Helper function to set flags for DEC
    fn set_dec_flags(&mut self, value: u8) {
        self.set_flag(Flag::S, value & 0x80 != 0);
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::H, (value & 0x0F) == 0x0F);
        self.set_flag(Flag::P, value == 0x80);
        self.set_flag(Flag::N, true);
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.f |= flag as u8;
        } else {
            self.f &= !(flag as u8);
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.f & (flag as u8) != 0
    }

    pub fn check_flag(&self, flag: Flag) -> bool {
        self.get_flag(flag)
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.memory.read_byte(address)
    }

    fn read_word(&self, address: u16) -> u16 {
        self.memory.read_word(address)
    }

    fn get_register_by_index(&mut self, index: u8) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.read_byte(self.get_hl()), // (HL)
            7 => self.a,
            _ => panic!("Invalid register index: {}", index),
        }
    }

    fn set_register_by_index(&mut self, index: u8, value: u8) {
        match index {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => self.memory.write_byte(self.get_hl(), value), // (HL)
            7 => self.a = value,
            _ => panic!("Invalid register index: {}", index),
        }
    }

    fn nop(&mut self) {
        // NOP does nothing, so this function is empty
    }

    fn get_af(&self) -> u16 {
        u16::from(self.a) << 8 | u16::from(self.f)
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn get_ix_d(&self, d: u8) -> u16 {
        let displacement = d as i8 as u16;
        self.ix.wrapping_add(displacement)
    }

    fn get_iy_d(&self, d: u8) -> u16 {
        let displacement = d as i8 as u16;
        self.iy.wrapping_add(displacement)
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0xFF) as u8;
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn ld_a_bc(&mut self) {
        let address = self.get_bc();
        self.a = self.memory.read_byte(address);
    }

    fn ld_a_de(&mut self) {
        let address = self.get_de();
        self.a = self.memory.read_byte(address);
    }

    fn ld_a_hl(&mut self) {
        let address = self.get_hl();
        self.a = self.memory.read_byte(address);
    }

    fn ld_hl_a(&mut self) {
        let address = self.get_hl();
        trace!("LD (HL), A: 0x{:02X} -> 0x{:04X}", self.a, address);
        self.memory.write_byte(address, self.a);
    }

    fn ld_hl_b(&mut self) {
        let address = self.get_hl();
        self.memory.write_byte(address, self.b);
    }

    fn ld_hl_c(&mut self) {
        let address = self.get_hl();
        self.memory.write_byte(address, self.c);
    }

    fn ld_hl_d(&mut self) {
        let address = self.get_hl();
        self.memory.write_byte(address, self.d);
    }

    fn ld_hl_e(&mut self) {
        let address = self.get_hl();
        self.memory.write_byte(address, self.h);
    }

    fn ld_hl_l(&mut self) {
        let address = self.get_hl();
        self.memory.write_byte(address, self.l);
    }

    fn ld_hl_h(&mut self) {
        let address = self.get_hl();
        self.memory.write_byte(address, self.h);
    }

    fn ld_de_a(&mut self) {
        let address = self.get_de();
        self.memory.write_byte(address, self.a);
    }

    fn ldi(&mut self) {
        let src = self.get_hl();
        let dest = self.get_de();
        let value = self.memory.read_byte(src);
        self.memory.write_byte(dest, value);

        self.set_hl(src.wrapping_add(1));
        self.set_de(dest.wrapping_add(1));
        self.set_bc(self.get_bc().wrapping_sub(1));

        // Set or reset the P/V flag based on the BC register value
        self.set_flag(Flag::P, self.get_bc() != 0);

        // Reset the N flag
        self.set_flag(Flag::N, false);
    }

    fn inc_hl(&mut self) {
        let hl = self.get_hl();
        let value = self.memory.read_byte(hl);
        let result = value.wrapping_add(1);

        self.set_inc_flags(result);

        self.memory.write_byte(hl, result);
    }

    fn dec_hl(&mut self) {
        let hl = self.get_hl();
        let value = self.memory.read_byte(hl);
        let result = value.wrapping_sub(1);

        self.set_dec_flags(result);

        self.memory.write_byte(hl, result);
    }

    // Stack operations
    fn push(&mut self, value: u16) {
        trace!("[->SP] 0x{:04X} into sp=0x{:04X}", value, self.sp);
        self.sp = self.sp.wrapping_sub(2);
        self.memory.write_word(self.sp, value);
    }

    fn pop(&mut self) -> u16 {
        let value = self.memory.read_word(self.sp);
        trace!("[<-SP] 0x{:04X} from sp=0x{:04X}", value, self.sp);
        self.sp = self.sp.wrapping_add(2);
        value
    }

    // CALL and RET
    fn call(&mut self, address: u16) {
        let value = self.pc.wrapping_add(2);
        trace!("CALL 0x{:04X} value=0x{:04X}", address, value);
        self.push(value);
        self.pc = address;
    }

    fn ret(&mut self) {
        trace!("RET");
        self.pc = self.pop();
    }

    fn rst(&mut self, address: u16) {
        let next_pc = self.pc.wrapping_add(1);
        self.push(next_pc);
        self.pc = address;
    }

    #[allow(unused)]
    pub fn dump(&self, dump_memory: bool) {
        println!("CPU State:");
        println!("A: {:02X} F: {:02X}", self.a, self.f);
        println!("B: {:02X} C: {:02X}", self.b, self.c);
        println!("D: {:02X} E: {:02X}", self.d, self.e);
        println!("H: {:02X} L: {:02X}", self.h, self.l);

        println!("Flags:");
        self.dump_flags();

        println!("Alternate Registers:");
        println!("B': {:02X} C': {:02X}", self.b_alt, self.c_alt);
        println!("D': {:02X} E': {:02X}", self.d_alt, self.e_alt);
        println!("H': {:02X} L': {:02X}", self.h_alt, self.l_alt);

        println!("16-bit Registers:");
        println!("SP: {:04X}", self.sp);
        println!("PC: {:04X}", self.pc);
        println!("IX: {:04X}", self.ix);
        println!("IY: {:04X}", self.iy);

        println!("Interrupts:");
        println!("IFF1: {} IFF2: {}", self.iff1, self.iff2);
        println!("IM: {}", self.im);
        println!("Interrupt Request: {}", self.interrupt_request);

        println!("Halted: {}", self.halted);

        if dump_memory {
            println!("Memory Dump:");
            for address in (0x0000..0x10000).step_by(16) {
                print!("{:04X}: ", address);
                for offset in 0..16 {
                    print!("{:02X} ", self.memory.read_byte((address + offset) as u16));
                }
                println!();
            }
        }
    }

    #[allow(unused)]
    pub fn dump_flags(&self) {
        fn debug_flag(value: bool) -> &'static str {
            if value {
                "1"
            } else {
                "0"
            }
        }

        println!("Flags:");
        println!("S (Sign):       {}", debug_flag(self.get_flag(Flag::S)));
        println!("Z (Zero):       {}", debug_flag(self.get_flag(Flag::Z)));
        println!("H (Half Carry): {}", debug_flag(self.get_flag(Flag::H)));
        println!("P (Parity):     {}", debug_flag(self.get_flag(Flag::P)));
        println!("N (Add/Sub):    {}", debug_flag(self.get_flag(Flag::N)));
        println!("C (Carry):      {}", debug_flag(self.get_flag(Flag::C)));
    }
}

fn overflow_sub(a: u8, b: u8, result: u8) -> bool {
    let signed_a = a as i8;
    let signed_b = b as i8;
    let signed_result = result as i8;
    (((signed_a as u8) ^ (signed_b as u8)) & ((signed_a as u8) ^ (signed_result as u8))) & 0x80 != 0
}

fn parity(value: u8) -> bool {
    value.count_ones() % 2 == 0
}
