use std::{cell::RefCell, fs::File, io::Read, rc::Rc};

use crate::components::{cpu::Z80, memory::Memory, sound::AY38910, vdp::TMS9918};

pub struct Msx {
    cpu: Z80,
    vdp: Rc<RefCell<TMS9918>>,
    #[allow(unused)]
    psg: Rc<RefCell<AY38910>>,

    current_scanline: u16,

    // debug options
    pub max_cycles: Option<u64>,
}

impl Msx {
    pub fn new() -> Self {
        let vdp = Rc::new(RefCell::new(TMS9918::new()));
        let psg = Rc::new(RefCell::new(AY38910::new()));
        let mut cpu = Z80::new(Memory::new(64 * 1024));

        cpu.register_device(vdp.clone());
        cpu.register_device(psg.clone());

        Self {
            cpu,
            vdp,
            psg,
            current_scanline: 0,
            max_cycles: None,
        }
    }

    #[allow(unused)]
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

    pub fn load_bios(&mut self, path: &str) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let load_address: u16 = 0x0000;
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

            let mut vdp = self.vdp.borrow_mut();
            vdp.render_scanline(self.current_scanline);

            self.current_scanline = (self.current_scanline + 1) % 262;
        }
    }
}
