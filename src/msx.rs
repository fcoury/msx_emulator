use std::{cell::RefCell, fs::File, io::Read, os::unix::net::UnixStream, path::PathBuf, rc::Rc};

use anyhow::anyhow;
use log::info;

use crate::{
    components::{cpu::Z80, display::Display, memory::Memory, sound::AY38910, vdp::TMS9918},
    open_msx::{find_socket, Client, Response},
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
    pub track_flags: bool,
}

impl Msx {
    pub fn new() -> Self {
        let vdp = Rc::new(RefCell::new(TMS9918::new()));
        let psg = Rc::new(RefCell::new(AY38910::new()));

        let display = Display::new(256, 192);

        let mut cpu = Z80::new(Memory::new(vdp.clone(), 64 * 1024));
        cpu.register_device(vdp.clone());
        cpu.register_device(psg.clone());

        Self {
            cpu,
            vdp,
            psg,
            display,
            current_scanline: 0,
            max_cycles: None,
            breakpoints: Vec::new(),
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

        let socket = find_socket()?;
        info!("Connecting to OpenMSX on socket {:?}", socket);
        let socket = UnixStream::connect(socket)?;
        let mut client = Client::new(&socket)?;
        send_openmsx_command(&mut client, "set power off")?;
        send_openmsx_command(&mut client, "machine HOTBIT")?;
        send_openmsx_command(&mut client, "debug set_bp 0x0001")?;
        send_openmsx_command(&mut client, "set power on")?;

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

            let opcode = self.cpu.memory.read_byte(self.cpu.pc);
            self.cpu.execute_cycle();
            client.get("debug step")?;
            let emu_status = client.get_status()?;
            let our_status = self.cpu.get_internal_state();

            info!("openMSX: {}", emu_status);
            info!("    MSX: {}", our_status);
            info!("    MSX: 0x{:2X}", opcode);

            let mismatch = format!("{}", emu_status) != format!("{}", our_status);

            if mismatch {
                info!("    Status mismatch!");
            }

            // if self.breakpoints.contains(&self.cpu.pc) {
            //     println!("Breakpoint hit at {:#06X}", self.cpu.pc);
            //     self.cpu.dump(false);
            //     break;
            // }
            if mismatch {
                let readline = rl.readline(">> ");

                if let Ok(command) = readline {
                    if command == "quit" || command == "q" {
                        break;
                    }

                    if command == "reset" {
                        self.cpu.reset();
                        client.get("reset")?;
                    }
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

        send_openmsx_command(&mut client, "set power off")?;
        Ok(())
    }
}

fn send_openmsx_command(client: &mut Client, command: &str) -> anyhow::Result<String> {
    match client.request(command) {
        Ok(Response::Ok(data)) => Ok(data),
        Ok(Response::Nok(data)) => Err(anyhow!("openMSX error: {}", data)),
        Err(e) => Err(e),
    }
}
