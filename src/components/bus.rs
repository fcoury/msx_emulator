use super::IoDevice;

struct Bus {
    slot_count: u8,
    slots: Vec<Box<dyn IoDevice>>,

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

            vdp_io_clock: 0,
            primary_slot_config: 0,
            slot3_secondary_config: 0,
        }
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
