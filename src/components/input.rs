use super::IoDevice;

pub struct Ppi {
    register_a: u8,
    register_b: u8,
    register_c: u8,
    control: u8,

    keyboard_row_selected: u8,
}

impl Ppi {
    pub fn new() -> Self {
        Ppi {
            register_a: 0,
            register_b: 0,
            register_c: 0x50, // Everything OFF. Motor and CapsLed = 1 means OFF
            control: 0,

            keyboard_row_selected: 0,
        }
    }

    pub fn reset(&mut self) {
        self.register_c = 0x50; // Everything OFF. Motor and CapsLed = 1 means OFF
        self.keyboard_row_selected = 0;
        self.update_pulse_signal();
        self.update_caps_led();
    }

    fn is_port_a_input(&self) -> bool {
        self.control & 0b1000_0000 == 0
    }

    fn is_port_b_input(&self) -> bool {
        self.control & 0b10 == 0
    }

    fn update_pulse_signal(&self) {
        // TODO: psg.set_pulse_signal((register_c & 0xa0) > 0);
    }

    fn update_caps_led(&self) {
        // TODO leds_socket.led_state_changed(0, (~registerC & 0x40) >> 6);
    }
}

impl IoDevice for Ppi {
    fn is_valid_port(&self, port: u8) -> bool {
        matches!(port, 0xA8 | 0xA9 | 0xAA | 0xAB)
    }

    fn read(&mut self, port: u8) -> u8 {
        match port {
            0xA8 => {
                // get primary slot config
                println!(
                    "  *** [PPI] Reading from PPI port {:02X} (input? {}) = {:02X}",
                    port,
                    self.is_port_a_input(),
                    self.register_a,
                );
                if self.is_port_a_input() {
                    self.register_a
                } else {
                    0xFF
                }
            }
            0xA9 => {
                // returns the keyboard port
                if self.is_port_b_input() {
                    self.register_b
                } else {
                    0xFF
                }
            }
            0xAA => {
                // returns register and flags
                // var mod = registerC ^ val;
                // if (!mod) return;
                // registerC = val;
                // if (mod & 0x0f) updateKeyboardConfig();
                // if (mod & 0xa0) updatePulseSignal();
                // if (mod & 0x40) updateCapsLed();

                self.register_c
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, port: u8, value: u8) {
        match port {
            0xA8 => {
                // set primary slot config
                println!(
                    "  *** [PPI] Writing '{:02X}' to PPI port 0xA8 (output? {})",
                    value,
                    !self.is_port_a_input()
                );
                self.register_a = value;
            }
            0xA9 => {
                println!(
                    "  *** [PPI] Writing '{:02X}' to PPI port 0xA9 (output? {})",
                    value,
                    !self.is_port_a_input()
                );
                // if !self.is_port_b_input() {
                self.register_b = value;
                // }
            }
            0xAA => {
                println!("  *** [PPI] Writing '{:02X}' to PPI port 0xAA", value);
                self.register_c = value;
                // var bit = (val & 0x0e) >>> 1;
                // if ((val & 0x01) === 0) registerC &= ~(1 << bit);
                // else registerC |= 1 << bit;

                // if (bit <= 3) updateKeyboardConfig();
                // else if (bit === 5 || bit === 7) updatePulseSignal();
                // else if (bit === 6) updateCapsLed();
            }
            0xAB => {
                println!(
                    "  *** [PPI] Writing '{:02X}' to PPI port 0xAB (control)",
                    value
                );
                self.control = value & 0x7F;
                let bit_number = (value >> 1) & 0x07;
                let bit_status = value & 0x01;
                if bit_status == 0 {
                    self.register_c &= !(1 << bit_number);
                } else {
                    self.register_c |= 1 << bit_number;
                }
            }
            _ => (),
        }
    }
}
