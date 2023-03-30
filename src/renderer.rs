use std::{cell::RefCell, rc::Rc};

use crate::components::vdp::TMS9918;

pub struct SDLRenderer {
    vdp: Rc<RefCell<TMS9918>>,
    pub screen_buffer: [u8; 256 * 192],
}

impl SDLRenderer {
    pub fn new(vdp: Rc<RefCell<TMS9918>>) -> Self {
        let screen_buffer = [0; 256 * 192];
        Self { vdp, screen_buffer }
    }

    pub fn draw(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) {
        // TODO check for text mode
        // TODO check for scroll delta

        let screen_mode = 0;

        let height = y1 - y0;

        for y in y0..height {
            // renders this raster line
            match screen_mode {
                0 => {
                    self.render_text_1(y);
                }
                _ => panic!("Unsupported screen mode: {}", screen_mode),
            }
        }
    }

    // 40x24 Text Mode
    // The Name Table occupies 960 bytes of VRAM from 0000H to 03BFH
    // Pattern Table occupies 2 KB of VRAM from 0800H to 0FFFH.
    // Each eight byte block contains the pixel pattern for a character code
    pub fn render_text_1(&mut self, line: u16) {
        let fg = 15; // TODO Pixel fg = palFg[vdp.getForegroundColor()];
        let bg = 0; // TODO Pixel bg = palBg[vdp.getBackgroundColor()];

        let vdp = self.vdp.borrow_mut();
        let pattern_area = vdp.pattern_table();
        let name_table_start = line * 40;
        let name_table_end = name_table_start + 40;

        for nti in name_table_start..name_table_end {
            let chr = vdp.vram[nti as usize];
            let pattern = pattern_area[(chr as usize * 8) + (line % 8) as usize];
            draw_char_line(&mut self.screen_buffer, nti as u8, pattern, fg, bg);
        }
    }
}

pub fn draw_char_line(screen_buffer: &mut [u8], n: u8, pattern: u8, fg: u8, bg: u8) {
    let start: usize = n as usize * 6;
    screen_buffer[start] = if pattern & 0x80 != 0 { fg } else { bg };
    screen_buffer[start + 1] = if pattern & 0x40 != 0 { fg } else { bg };
    screen_buffer[start + 2] = if pattern & 0x20 != 0 { fg } else { bg };
    screen_buffer[start + 3] = if pattern & 0x10 != 0 { fg } else { bg };
    screen_buffer[start + 4] = if pattern & 0x08 != 0 { fg } else { bg };
    screen_buffer[start + 5] = if pattern & 0x04 != 0 { fg } else { bg };
}
