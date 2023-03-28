use std::{cell::RefCell, fs::File, io::Read, path::PathBuf, rc::Rc};

use log::info;

use crate::{
    components::{cpu::Z80, display::Display, memory::Memory, sound::AY38910, vdp::TMS9918},
    open_msx::Client,
    Cli,
};

pub struct Msx {
    cpu: Z80,
    vdp: Rc<RefCell<TMS9918>>,
    #[allow(unused)]
    psg: Rc<RefCell<AY38910>>,

    display: Display,

    current_scanline: u16,

    // debug options
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    open_msx: bool,
    break_on_mismatch: bool,
    pub track_flags: bool,
}

impl Msx {
    pub fn new(cli: &Cli) -> Self {
        let vdp = Rc::new(RefCell::new(TMS9918::new()));
        let psg = Rc::new(RefCell::new(AY38910::new()));

        let display = Display::new(256, 192);

        let mut cpu = Z80::new(Memory::new(vdp.clone(), 64 * 1024));
        cpu.register_device(vdp.clone());
        cpu.register_device(psg.clone());

        let mut breakpoints: Vec<u16> = Vec::new();
        for breakpoint in &cli.breakpoint {
            let breakpoint = u16::from_str_radix(&breakpoint[2..], 16).unwrap();
            breakpoints.push(breakpoint);
        }

        Self {
            cpu,
            vdp,
            psg,
            display,
            current_scanline: 0,
            max_cycles: None,
            breakpoints,
            open_msx: cli.open_msx,
            break_on_mismatch: cli.break_on_mismatch,
            track_flags: false,
        }
    }

    pub fn add_breakpoint(&mut self, address: u16) {
        self.breakpoints.push(address);
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

    pub fn load_bios(&mut self, path: PathBuf) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.cpu.memory.load_bios(&buffer)?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut event_pump = self.display.sdl_context.event_pump().unwrap();

        info!("OpenMSX: {}", self.open_msx);
        let mut client = if self.open_msx {
            let mut client = Client::new()?;
            client.init()?;
            println!("Connected to openMSX! (type 'quit' to exit)");

            Some(client)
        } else {
            None
        };

        self.cpu.max_cycles = self.max_cycles;
        self.cpu.track_flags = self.track_flags;

        let mut rl = rustyline::DefaultEditor::new()?;

        'running: loop {
            // Handle input events
            for event in event_pump.poll_iter() {
                use sdl2::event::Event;
                #[allow(clippy::single_match)]
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {}
                }
            }

            let last_opcode = self.cpu.memory.read_byte(self.cpu.pc);
            println!(
                "running pc = {:#06X} opcode = {:#04X}",
                self.cpu.pc, last_opcode
            );
            self.cpu.execute_cycle();
            println!(
                "    ran pc = {:#06X} opcode = {:#04X}",
                self.cpu.pc,
                self.cpu.memory.read_byte(self.cpu.pc)
            );

            let mut stop = false;

            if let Some(client) = &mut client {
                client.step()?;

                let emu_status = client.get_status()?;
                let our_status = self.cpu.get_internal_state();

                info!("openMSX: {}", emu_status);
                info!("    MSX: {}", our_status);

                if self.break_on_mismatch && format!("{}", emu_status) != format!("{}", our_status)
                {
                    info!("    Status mismatch!");
                    stop = true;
                }
            }

            if self.breakpoints.contains(&self.cpu.pc) {
                println!("Breakpoint hit at {:#06X}", self.cpu.pc);
                stop = true;
            }

            if stop {
                let mut quit = false;
                loop {
                    let readline = rl.readline(">> ");

                    if let Ok(command) = readline {
                        if command == "quit" || command == "q" {
                            quit = true;
                            break;
                        }

                        if command == "reset" {
                            if let Some(client) = &mut client {
                                self.cpu.reset();
                                client.send("reset")?;
                            }
                        }

                        if command.starts_with("set ") {
                            let command = command.replace("set ", "");
                            let command = command.split(' ').collect::<Vec<&str>>();
                            if command[0] == "a" {
                                let value = u8::from_str_radix(command[1], 16).unwrap();
                                self.cpu.a = value;
                                let our_status = self.cpu.get_internal_state();
                                info!("    MSX: {}", our_status);
                            }
                        }

                        if command == "d" {
                            if let Some(client) = &mut client {
                                let emu_status = client.get_status()?;
                                let our_status = self.cpu.get_internal_state();

                                println!(" opcode: {:#04X}", last_opcode);
                                println!(
                                    " opcode: {:#04X}",
                                    self.cpu.memory.read_byte(self.cpu.pc)
                                );
                                println!("openMSX: {}", emu_status);
                                println!("   ours: {}", our_status);
                            }
                        }

                        if command.starts_with("mem ") {
                            let command = command.replace("mem ", "");
                            let command = command.split(' ').collect::<Vec<&str>>();
                            if command[0].starts_with("0x") {
                                let address = u16::from_str_radix(&command[0][2..], 16).unwrap();
                                let our_status = self.cpu.memory.read_byte(address);

                                if let Some(client) = &mut client {
                                    let emu_status = client
                                        .send(&format!("debug read memory 0x{:04X}", address))?;
                                    let value = u8::from_str_radix(&emu_status, 8).unwrap();
                                    info!("openMSX: {:#04X}", value);
                                }

                                info!("   ours: {:#04X}", our_status);
                            }
                        }

                        if command.starts_with("cont") {
                            break;
                        }
                    }
                }

                if quit {
                    break;
                }
            }

            if self.cpu.halted {
                break;
            }

            let mut vdp = self.vdp.borrow_mut();
            vdp.render_scanline(self.current_scanline);

            self.current_scanline = (self.current_scanline + 1) % 262;
            self.display.update_screen(&vdp.screen_buffer);
        }

        if let Some(client) = &mut client {
            client.send("set power off")?;
        }

        Ok(())
    }
}
