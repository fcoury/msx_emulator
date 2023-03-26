use std::{fs::File, io::Read};

use crate::components::{cpu::Z80, memory::Memory, vdp::TMS9918};

pub struct Msx {
    cpu: Z80,
    vdp: TMS9918,

    // debug options
    pub max_cycles: Option<u64>,
}

impl Msx {
    pub fn new() -> Self {
        let cpu = Z80::new(Memory::new(64 * 1024));
        let vdp = TMS9918::new();

        Self {
            cpu,
            vdp,
            max_cycles: None,
        }
    }

    pub fn load_binary(&mut self, path: &str, load_address: u16) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        for (i, byte) in buffer.iter().enumerate() {
            let address = load_address.wrapping_add(i as u16);
            self.cpu.memory.write_byte(address, *byte);
        }

        Ok(())
    }

    pub fn run(&mut self) {
        self.cpu.max_cycles = self.max_cycles;

        loop {
            self.cpu.execute_cycle();
            if self.cpu.halted {
                break;
            }
            self.vdp.render_scanline();
        }
    }
}
