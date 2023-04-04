use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::File,
    hash::{Hash, Hasher},
    io::Read,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use rustyline::DefaultEditor;
use serde::{Deserialize, Serialize};
use tracing::{info, trace};
use twox_hash::XxHash64;

use crate::{
    components::{bus::Bus, cpu::Z80, memory::Memory, sound::AY38910, vdp::TMS9918},
    open_msx::Client,
    Cli,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Msx {
    pub cpu: Z80,
    pub vdp: TMS9918,
    pub psg: AY38910,

    // #[serde(skip)]
    // display: Option<Display>,
    current_scanline: u16,

    // debug options
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    pub open_msx: bool,
    pub break_on_mismatch: bool,
    pub track_flags: bool,
    pub previous_memory: Option<Vec<u8>>,
    pub memory_hash: u64,
}

impl Msx {
    pub fn new() -> Self {
        println!("Initializing MSX...");
        let bus = Arc::new(RwLock::new(Bus::new()));
        let memory = Memory::new(bus.clone(), 64 * 1024);
        let cpu = Z80::new(bus, memory);
        // let display = Display::new(256, 192);

        Self {
            cpu,
            // display: Some(display),
            current_scanline: 0,
            max_cycles: None,
            track_flags: false,
            vdp: TMS9918::new(),
            psg: AY38910::new(),
            open_msx: false,
            break_on_mismatch: false,
            breakpoints: Vec::new(),
            previous_memory: None,
            memory_hash: 0,
        }
    }

    pub fn from_cli(cli: &Cli) -> Self {
        let mut msx = Self::new();
        msx.open_msx = cli.open_msx;
        msx.break_on_mismatch = cli.break_on_mismatch;
        msx.max_cycles = cli.max_cycles;

        for breakpoint in &cli.breakpoint {
            let breakpoint = u16::from_str_radix(&breakpoint[2..], 16).unwrap();
            msx.add_breakpoint(breakpoint);
        }

        msx
    }

    pub fn delta_memory(&mut self, client_hash: &str) -> HashMap<u16, u8> {
        let mut delta_memory = HashMap::new();

        if let Some(previous_memory) = self.previous_memory.as_ref() {
            tracing::debug!(
                "Client hash: {} Our hash: {}",
                client_hash,
                self.memory_hash
            );
            // Compare client_hash with self.memory_hash
            if client_hash == self.memory_hash.to_string() {
                for (index, (&prev_byte, &cur_byte)) in previous_memory
                    .iter()
                    .zip(self.cpu.memory.data.iter())
                    .enumerate()
                {
                    if prev_byte != cur_byte {
                        delta_memory.insert(index as u16, cur_byte);
                    }
                }
            } else {
                trace!("Memory hash mismatch, sending full memory dump");
                for (index, &cur_byte) in self.cpu.memory.data.iter().enumerate() {
                    delta_memory.insert(index as u16, cur_byte);
                }
            }
        } else {
            trace!("No previous memory, sending full memory dump");
            for (index, &cur_byte) in self.cpu.memory.data.iter().enumerate() {
                delta_memory.insert(index as u16, cur_byte);
            }
        }

        self.memory_hash = calculate_memory_hash(&self.cpu.memory.data);
        self.previous_memory = Some(self.cpu.memory.data.clone());

        delta_memory
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

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.vdp.reset();
        self.psg.reset();
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        // let Some(display) = self.display else {
        //     bail!("Display not initialized");
        // };
        // let mut event_pump = display.sdl_context.event_pump().unwrap();

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

        #[allow(unused)]
        let mut rl = rustyline::DefaultEditor::new()?;
        let mut stop_next = false;

        // let mut renderer = Renderer::new(&self.vdp);

        loop {
            // 'running: loop {
            // Handle input events
            // for event in event_pump.poll_iter() {
            //     use sdl2::event::Event;
            //     #[allow(clippy::single_match)]
            //     match event {
            //         Event::Quit { .. } => break 'running,
            //         Event::KeyDown { keycode, .. } => match keycode {
            //             Some(Keycode::D) => {
            //                 let our_status = self.cpu.get_internal_state();

            //                 // println!(" opcode: {:#04X}", last_opcode);
            //                 // println!(" opcode: {:#04X}", self.cpu.memory.read_byte(self.cpu.pc));
            //                 println!("   ours: {}", our_status);

            //                 if let Some(client) = &mut client {
            //                     let emu_status = client.get_status()?;
            //                     println!("openMSX: {}", emu_status);
            //                 }
            //             }
            //             Some(Keycode::P) => {
            //                 let pattern_table = self.vdp.pattern_table();
            //                 println!("Pattern Table:");
            //                 for charn in 0..256 {
            //                     for charbit in charn * 8..charn * 8 + 8 {
            //                         let byte = pattern_table[charbit];
            //                         for bit in 0..8 {
            //                             let pixel = (byte >> (7 - bit)) & 1;
            //                             print!("{}", if pixel == 1 { "X" } else { " " });
            //                         }
            //                         println!();
            //                     }
            //                     println!("---");
            //                 }
            //                 println!("----------------");
            //                 let readline = rl.readline(">> ");
            //             }
            //             Some(Keycode::V) => {
            //                 info!("VDP Dump");
            //                 info!("  registers: {:#04X?}", self.vdp.registers);
            //                 info!("  status: {:#02X}", self.vdp.status);
            //                 info!("  address: {:#04X}", self.vdp.address);
            //                 if let Some(v) = self.vdp.first_write {
            //                     info!("  latched val: {:#02X}", v);
            //                 } else {
            //                     info!("  latched val: None");
            //                 }
            //             }
            //             Some(Keycode::H) => {
            //                 let s = hexdump(&self.vdp.vram);
            //                 println!("{}", s);
            //             }
            //             Some(Keycode::Q) => {
            //                 println!("Ê!");
            //                 break 'running;
            //             }
            //             _ => {}
            //         },
            //         _ => {}
            //     }
            // }

            // let last_opcode = self.cpu.memory.read_byte(self.cpu.pc);
            // debug!(
            //     "running pc = {:#06X} opcode = {:#04X}",
            //     self.cpu.pc, last_opcode
            // );
            self.cpu.execute_cycle();
            // debug!(
            //     "    ran pc = {:#06X} opcode = {:#04X}",
            //     self.cpu.pc,
            //     self.cpu.memory.read_byte(self.cpu.pc)
            // );

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

            if stop || stop_next {
                if stop_next {
                    println!("Stepped to {:#06X}", self.cpu.pc);
                }
                stop_next = false;
                let mut quit = false;

                let mut rl = DefaultEditor::new()?;
                if rl.load_history("history.txt").is_err() {
                    println!("No previous history.");
                }

                loop {
                    let readline = rl.readline(">> ");

                    if let Ok(command) = readline {
                        rl.add_history_entry(command.as_str())?;
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
                            let our_status = self.cpu.get_internal_state();

                            // println!(" opcode: {:#04X}", last_opcode);
                            // println!(" opcode: {:#04X}", self.cpu.memory.read_byte(self.cpu.pc));
                            println!("   ours: {}", our_status);

                            if let Some(client) = &mut client {
                                let emu_status = client.get_status()?;
                                println!("openMSX: {}", emu_status);
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
                                    println!("emu_status: {}", emu_status);
                                    let value = emu_status.parse::<u8>()?;
                                    info!("openMSX: {:#04X}", value);
                                }

                                info!("   ours: {:#04X}", our_status);
                            }
                        }

                        if command.starts_with("memset ") {
                            let command = command.replace("memset ", "");
                            let command = command.split(' ').collect::<Vec<&str>>();
                            if command[0].starts_with("0x") {
                                let address = u16::from_str_radix(&command[0][2..], 16).unwrap();
                                let value = u8::from_str_radix(&command[1][2..], 16).unwrap();
                                self.cpu.memory.write_byte(address, value);

                                if let Some(client) = &mut client {
                                    client.send(&format!(
                                        "debug write memory 0x{:04X} 0x{:02X}",
                                        address, value
                                    ))?;
                                }
                            }
                        }

                        if command == "step" || command == "n" {
                            stop_next = true;
                            break;
                        }

                        if command == "c" || command.starts_with("cont") {
                            break;
                        }

                        if command == "m" {
                            self.break_on_mismatch = !self.break_on_mismatch;
                            println!(
                                "Break on mismatch: {}",
                                if self.break_on_mismatch { "on" } else { "off" }
                            );
                        }

                        if command == "b" {
                            if self.breakpoints.is_empty() {
                                println!("No breakpoints set.");
                            } else {
                                println!("Breakpoints:");
                                for (i, breakpoint) in self.breakpoints.iter().enumerate() {
                                    println!("  {}. {:#06X}", i, breakpoint);
                                }
                            }
                        }

                        if command.starts_with("rb ") {
                            let command = command.replace("rb ", "");
                            let command = command.split(' ').collect::<Vec<&str>>();
                            if command[0].starts_with("0x") {
                                // TODO
                            } else {
                                let index = command[0].parse::<usize>().unwrap();
                                self.breakpoints.remove(index);
                            }
                        }
                    }
                }

                if quit {
                    rl.append_history("history.txt")?;
                    break;
                }
            }

            if self.cpu.halted {
                break;
            }

            self.current_scanline = (self.current_scanline + 1) % 192;
            if self.current_scanline == 0 {
                // renderer.draw(0, 0, 256, 192);
                // display.update_screen(&renderer.screen_buffer);
            }
        }

        if let Some(client) = &mut client {
            client.send("set power off")?;
        }

        Ok(())
    }
}

fn calculate_memory_hash(memory: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(memory);
    hasher.finish()
}
