use std::{cell::RefCell, rc::Rc};

use log::{error, info, trace};

use super::{memory::Memory, IoDevice};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flag {
    Z = 0x80, // Zero
    S = 0x40, // Sign
    H = 0x20, // Half Carry
    P = 0x10, // Parity/Overflow
    N = 0x08, // Add/Subtract
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
    cycles: u64,
}

impl Z80 {
    pub fn new(memory: Memory) -> Self {
        let io_devices: Vec<Rc<RefCell<dyn IoDevice + 'static>>> = vec![];
        Z80 {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            b_alt: 0,
            c_alt: 0,
            d_alt: 0,
            e_alt: 0,
            h_alt: 0,
            l_alt: 0,
            sp: 0xF000,
            pc: 0,
            iff1: false,
            iff2: false,
            im: 0,
            interrupt_request: false,
            memory,
            io_devices,
            halted: false,
            max_cycles: None,
            cycles: 0,
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
        info!("PC: 0x{:04X} Opcode: 0x{:02X}", self.pc, opcode);
        // trace!(
        //     "A: 0x{:02X} B: 0x{:02X} C: 0x{:02X} F: 0b{:b}",
        //     self.a,
        //     self.b,
        //     self.c,
        //     self.f
        // );

        let mut inc_pc = false;

        // Execute the instruction
        match opcode {
            0x00 => self.nop(),
            0xD7 => {
                // RST 0x10
                trace!("RST");
                self.rst(0x10);
                inc_pc = false;
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
            }
            0x16 => {
                // LD D, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD D, 0x{:02X}", value);
                self.d = value;
            }
            0x1E => {
                // LD E, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD E, 0x{:02X}", value);
                self.e = value;
            }
            0x26 => {
                // LD H, n
                self.pc = self.pc.wrapping_add(1);
                let value = self.memory.read_byte(self.pc);
                trace!("LD H, 0x{:02X}", value);
                self.h = value;
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
                self.ld_hl_a();
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
            0xF9 => {
                // LD SP, HL
                trace!("LD SP, HL");
                self.pc = self.pc.wrapping_add(1);
                self.sp = self.get_hl();
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
                trace!("LD A, (HL)");
                self.pc = self.pc.wrapping_add(1);
                self.ld_a_hl();
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
            0x10 => {
                // DJNZ n
                self.pc = self.pc.wrapping_add(1);
                let offset = self.read_byte(self.pc) as i8;

                self.b = self.b.wrapping_sub(1);

                if self.b != 0 {
                    self.pc = self.pc.wrapping_add(1);
                    self.pc = self.pc.wrapping_add(offset as u16);
                } else {
                    self.pc = self.pc.wrapping_add(1);
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
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                let result = self.a.wrapping_sub(value);
                self.a = result;
            }
            0x9F => {
                // SBC A, A
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.a);
                self.a = result;
            }
            0x98 => {
                // SBC A, B
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.b);
                self.a = result;
            }
            0x99 => {
                // SBC A, C
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.c);
                self.a = result;
            }
            0x9A => {
                // SBC A, D
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.d);
                self.a = result;
            }
            0x9B => {
                // SBC A, E
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.e);
                self.a = result;
            }
            0x9C => {
                // SBC A, H
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.h);
                self.a = result;
            }
            0x9D => {
                // SBC A, L
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.l);
                self.a = result;
            }
            0x9E => {
                // SBC A, (HL)
                self.pc = self.pc.wrapping_add(1);
                let result = self.a.wrapping_sub(self.memory.read_byte(self.get_hl()));
                self.a = result;
            }
            0xDE => {
                // SBC A, n
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                let result = self.a.wrapping_sub(value);
                self.a = result;
            }
            0xA7 => {
                // AND A
                self.pc = self.pc.wrapping_add(1);

                self.set_flag(Flag::Z, self.a == 0);
                self.set_flag(Flag::S, self.a & 0x80 != 0);
                self.set_flag(Flag::H, true);
                self.set_flag(Flag::P, parity(self.a));
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::C, false);
            }
            0xA0 => {
                // AND B
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.b;
                self.a = result;
            }
            0xA1 => {
                // AND C
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.c;
                self.a = result;
            }
            0xA2 => {
                // AND D
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.d;
                self.a = result;
            }
            0xA3 => {
                // AND E
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.e;
                self.a = result;
            }
            0xA4 => {
                // AND H
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.h;
                self.a = result;
            }
            0xA5 => {
                // AND L
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.l;
                self.a = result;
            }
            0xA6 => {
                // AND (HL)
                self.pc = self.pc.wrapping_add(1);
                let result = self.a & self.memory.read_byte(self.get_hl());
                self.a = result;
            }
            0xE6 => {
                // AND n
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                let result = self.a & value;
                self.a = result;
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
                self.pc = self.pc.wrapping_add(1);
                let result = self.a | self.memory.read_byte(self.get_hl());
                self.a = result;
            }
            0xF6 => {
                // OR n
                trace!("OR n");
                let value = self.memory.read_byte(self.pc.wrapping_add(1));
                self.pc = self.pc.wrapping_add(2);
                let result = self.a | value;
                self.a = result;
            }
            0xAF => {
                // XOR A
                trace!("XOR A");
                self.pc = self.pc.wrapping_add(1);
                self.a = 0;

                self.set_flag(Flag::Z, true);
                self.set_flag(Flag::S, false);
                self.set_flag(Flag::H, false);
                self.set_flag(Flag::P, parity(self.a));
                self.set_flag(Flag::N, false);
                self.set_flag(Flag::C, false);
            }
            0xA8 => {
                // XOR B
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.b;
                self.a = result;
            }
            0xA9 => {
                // XOR C
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.c;
                self.a = result;
            }
            0xAA => {
                // XOR D
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.d;
                self.a = result;
            }
            0xAB => {
                // XOR E
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.e;
                self.a = result;
            }
            0xAC => {
                // XOR H
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.h;
                self.a = result;
            }
            0xAD => {
                // XOR L
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.l;
                self.a = result;
            }
            0xAE => {
                // XOR (HL)
                self.pc = self.pc.wrapping_add(1);
                let result = self.a ^ self.memory.read_byte(self.get_hl());
                self.a = result;
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
                trace!("CPL");
                self.a = !self.a;
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, true);
                self.pc = self.pc.wrapping_add(1);
            }
            0xFE => {
                // CP n
                let immediate_value = self.read_byte(self.pc.wrapping_add(1));
                let a = self.a;

                trace!("CP 0x{:02X}", immediate_value);

                let result = a.wrapping_sub(immediate_value);

                self.set_flag(Flag::Z, result == 0);
                self.set_flag(Flag::N, true);
                self.set_flag(Flag::H, (a & 0x0F) < (immediate_value & 0x0F));
                self.set_flag(Flag::C, a < immediate_value);

                self.pc = self.pc.wrapping_add(2);
            }
            0xBE => {
                // CP (HL)
                let hl_value = self.memory.read_byte(self.get_hl());
                trace!("CP (HL) [0x{:04X}]", self.get_hl());
                self.compare(self.a, hl_value);
                self.pc = self.pc.wrapping_add(1);
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
            0xCD => {
                // CALL nn
                self.pc = self.pc.wrapping_add(1);
                let address = self.read_word(self.pc);
                self.call(address);
            }
            0xC9 => {
                // RET
                self.ret();
                inc_pc = false;
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
                inc_pc = false;
            }

            0xC1 => {
                // POP BC
                trace!("POP BC");
                self.pc = self.pc.wrapping_add(1);
                let value = self.pop();
                self.set_bc(value);
                inc_pc = false;
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
                inc_pc = false;
            }
            0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA | 0xFA => {
                // JP cc, nn
                inc_pc = false;
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
                trace!("JP cc, 0x{:04X} = {}", address, condition);

                self.pc = self.pc.wrapping_add(3);

                if condition {
                    self.pc = address;
                }
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
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
                    0x28 => trace!(
                        "JR Z, 0x{:04X} = {}",
                        self.pc.wrapping_add(offset as u16),
                        condition
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
            0xCB => {
                // Read extended opcode and execute it
                let extended_opcode = self.read_byte(self.pc.wrapping_add(1));
                inc_pc = false;

                match extended_opcode {
                    0x40..=0x7F => {
                        // BIT b, r
                        let bit = (extended_opcode >> 3) & 0x07;
                        let reg_index = extended_opcode & 0x07;

                        trace!("BIT {}, {}", bit, reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let mask = 1 << bit;

                        self.set_flag(Flag::Z, value & mask == 0);
                        self.set_flag(Flag::H, true);
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
                    0x00..=0x3F => {
                        // SLA r
                        let reg_index = extended_opcode & 0x07;

                        trace!("SLA {}", reg_index);
                        let value = self.get_register_by_index(reg_index);
                        let carry = (value & 0x80) != 0;

                        let result = value << 1;
                        self.set_register_by_index(reg_index, result);

                        self.set_flag(Flag::Z, result == 0);
                        self.set_flag(Flag::N, false);
                        self.set_flag(Flag::H, false);
                        self.set_flag(Flag::C, carry);

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
                error!("OUT (#{:02X}), A=0x{:02X}", port, data);

                for device in &self.io_devices {
                    if device.borrow().is_valid_port(port) {
                        let mut device_ref = device.as_ref().borrow_mut();
                        device_ref.write(port, data);
                        break;
                    }
                }

                self.pc = self.pc.wrapping_add(2);
            }

            // Interrupts
            // EI
            0xFB => {
                self.pc = self.pc.wrapping_add(1);
                self.iff1 = true;
            }
            // DI
            0xF3 => {
                self.pc = self.pc.wrapping_add(1);
                self.iff1 = false;
            }

            _ => panic!("Unhandled opcode: {:02X}", opcode),
        }

        // Increment the program counter
        if inc_pc {
            self.pc = self.pc.wrapping_add(1);
        }
    }

    fn add_a(&mut self, value: u8) {
        let a = self.a;
        let result = a.wrapping_add(value);

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, (a & 0x0F) + (value & 0x0F) > 0x0F);
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

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, (a & 0x0F) < (value & 0x0F));
        self.set_flag(Flag::C, a < value);

        self.a = result;
    }

    // Helper function to set flags for INC
    fn set_inc_flags(&mut self, value: u8) {
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, false);
        self.set_flag(Flag::H, (value & 0x0F) == 0x00);
    }

    // Helper function to set flags for DEC
    fn set_dec_flags(&mut self, value: u8) {
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, (value & 0x0F) == 0x0F);
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

    fn compare(&mut self, a: u8, value: u8) {
        let result = a.wrapping_sub(value);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, true);
        self.set_flag(Flag::H, (a & 0x0F) < (value & 0x0F));
        self.set_flag(Flag::C, a < value);
    }
}

fn parity(value: u8) -> bool {
    let mut count = 0;
    for i in 0..8 {
        if value & (1 << i) != 0 {
            count += 1;
        }
    }
    count % 2 == 0
}
