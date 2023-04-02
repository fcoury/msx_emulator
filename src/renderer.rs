#![allow(dead_code)]
use crate::components::vdp::TMS9918;

pub struct Renderer<'a> {
    vdp: &'a TMS9918,
    pub screen_buffer: [u8; 256 * 192],
}

impl<'a> Renderer<'a> {
    pub fn new(vdp: &'a TMS9918) -> Self {
        let screen_buffer = [0; 256 * 192];
        Self { vdp, screen_buffer }
    }

    pub fn draw(&mut self, _x0: u16, y0: u16, _x1: u16, y1: u16) {
        // TODO check for text mode
        // TODO check for scroll delta
        let fg = 15; // TODO Pixel fg = palFg[vdp.getForegroundColor()];
        let bg = 1; // TODO Pixel bg = palBg[vdp.getBackgroundColor()];

        let screen_mode = 0;
        let height = y1 - y0;

        for y in y0..height {
            // renders this raster line
            match screen_mode {
                0 => {
                    self.render_text1(y as usize, fg, bg);
                }
                _ => panic!("Unsupported screen mode: {}", screen_mode),
            }
        }
    }

    pub fn render_text1(&mut self, line: usize, fg: u8, bg: u8) {
        let pattern_area = self.vdp.pattern_table();
        let l = (line + self.vdp.get_vertical_scroll()) & 7;

        let name_start = (line / 8) * 40;
        let name_end = name_start + 40;
        let mut pixel_ptr = line * 256;
        for name in name_start..name_end {
            // FIXME why is the screen content at 0x0990 in our version?
            let screen_offset = 0x0900 + name; // Calculate the proper offset in the VRAM
            let char_code = self.vdp.vram[screen_offset]; // Get the value directly from the VRAM array
            let pattern = pattern_area[l + char_code as usize * 8];

            for i in 0..6 {
                let mask = 0x80 >> i;
                self.screen_buffer[pixel_ptr + i] = if (pattern & mask) != 0 { fg } else { bg };
            }

            pixel_ptr += 6;
        }
    }
}

//     pub fn draw(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) {
//         // TODO check for text mode
//         // TODO check for scroll delta

//         let screen_mode = 0;

//         let height = y1 - y0;

//         for y in y0..height {
//             // renders this raster line
//             match screen_mode {
//                 0 => {
//                     self.render_text_1(y);
//                 }
//                 _ => panic!("Unsupported screen mode: {}", screen_mode),
//             }
//         }
//     }

//     pub fn render_text1<Pixel: Copy>(
//         buf: &mut [Pixel; 256],
//         line: usize,
//         draw6: &dyn Fn(&mut [Pixel], Pixel, Pixel, u8),
//     ) {
//         let fg = 15; // TODO Pixel fg = palFg[vdp.getForegroundColor()];
//         let bg = 1; // TODO Pixel bg = palBg[vdp.getBackgroundColor()];

//         let pattern_area = vdp.pattern_table();
//         let l = (line + vdp.get_vertical_scroll()) & 7;

//         let name_start = (line / 8) * 40;
//         let name_end = name_start + 40;
//         let mut pixel_ptr = 0;
//         for name in name_start..name_end {
//             let char_code = vdp.vram_read_np((name + 0xC00) | (!0u32 << 12) as usize);
//             let pattern = pattern_area[l + char_code * 8];
//             draw6(&mut buf[pixel_ptr..], fg, bg, pattern);
//             pixel_ptr += 6;
//         }
//     }

//     // 40x24 Text Mode
//     // The Name Table occupies 960 bytes of VRAM from 0000H to 03BFH
//     // Pattern Table occupies 2 KB of VRAM from 0800H to 0FFFH.
//     // Each eight byte block contains the pixel pattern for a character code
//     pub fn render_text_1_mine(&mut self, y: u16) {
//         let fg = 15; // TODO Pixel fg = palFg[vdp.getForegroundColor()];
//         let bg = 1; // TODO Pixel bg = palBg[vdp.getBackgroundColor()];

//         // renders a single line of text (8 pixels high)

//         let text_line = y / 8;
//         let char_line = y % 8;

//         let vdp = self.vdp.borrow_mut();
//         let pattern_area = vdp.pattern_table();
//         let name_table_start = text_line;
//         let name_table_end = text_line + 40;

//         println!(
//             "{}: 0x{:04X} - 0x{:04X}",
//             y, name_table_start, name_table_end
//         );
//         print!("{}: ", y);
//         for nti in name_table_start..name_table_end {
//             let chr = vdp.vram[nti as usize];
//             print!("{}", chr as char);
//             let pattern = pattern_area[(chr as usize * 8)];
//             draw_char_line(&mut self.screen_buffer, nti as u8, pattern, fg, bg);
//         }
//         println!();
//     }
// }

// pub fn draw_char_line(screen_buffer: &mut [u8], n: u8, pattern: u8, fg: u8, bg: u8) {
//     let start: usize = n as usize * 6;
//     screen_buffer[start] = if pattern & 0x80 != 0 { fg } else { bg };
//     screen_buffer[start + 1] = if pattern & 0x40 != 0 { fg } else { bg };
//     screen_buffer[start + 2] = if pattern & 0x20 != 0 { fg } else { bg };
//     screen_buffer[start + 3] = if pattern & 0x10 != 0 { fg } else { bg };
//     screen_buffer[start + 4] = if pattern & 0x08 != 0 { fg } else { bg };
//     screen_buffer[start + 5] = if pattern & 0x04 != 0 { fg } else { bg };
// }
