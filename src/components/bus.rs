#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use tracing::error;

use super::IoDevice;

pub struct Bus {
    slot_count: u8,
    slots: Vec<Box<dyn IoDevice>>,

    // I/O Devices
    io_devices: Vec<Rc<RefCell<dyn IoDevice + 'static>>>,

    vdp_io_clock: u8,
    primary_slot_config: u8,
    slot3_secondary_config: u8,
}

impl Bus {
    pub fn new() -> Self {
        let slot_count = 4;

        // Create a Vec<Box<dyn IoDevice>> with EmptySlot instances using a loop
        let mut slots: Vec<Box<dyn IoDevice>> = Vec::with_capacity(slot_count as usize);
        for _ in 0..slot_count {
            slots.push(Box::new(EmptySlot));
        }

        Self {
            slot_count,
            slots,

            io_devices: Vec::new(),

            vdp_io_clock: 0,
            primary_slot_config: 0,
            slot3_secondary_config: 0,
        }
    }

    pub fn register_device(&mut self, device: Rc<RefCell<dyn IoDevice>>) {
        self.io_devices.push(device);
    }

    pub fn input(&mut self, port: u8) -> u8 {
        for device in &self.io_devices {
            if device.borrow().is_valid_port(port) {
                let mut device_ref = device.as_ref().borrow_mut();
                return device_ref.read(port);
            }
        }

        error!("  *** [BUS] Invalid port {:02X} read", port);
        0xff
    }

    pub fn output(&mut self, port: u8, data: u8) {
        for device in &self.io_devices {
            if device.borrow().is_valid_port(port) {
                let mut device_ref = device.as_ref().borrow_mut();
                device_ref.write(port, data);
                return;
            }
        }

        error!("  *** [BUS] Invalid port {:02X} write", port);
    }

    fn reset(&mut self) {
        self.vdp_io_clock = 0;
        self.primary_slot_config = 0;
    }
}

#[derive(Clone)]
struct EmptySlot;

impl IoDevice for EmptySlot {
    fn is_valid_port(&self, _port: u8) -> bool {
        false
    }

    fn read(&mut self, _port: u8) -> u8 {
        0xFF
    }

    fn write(&mut self, _port: u8, _data: u8) {}
}
