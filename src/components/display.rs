// display.rs
use sdl2::{pixels::PixelFormatEnum, render::TextureCreator, video::WindowContext};

pub struct Display {
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub texture_creator: TextureCreator<WindowContext>,
}

impl Display {
    pub fn new(width: u32, height: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("MSX Emulator", width, height)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();

        Display {
            sdl_context,
            video_subsystem,
            canvas,
            texture_creator,
        }
    }

    pub fn update_screen(&mut self, screen_buffer: &[u8]) {
        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, 256, 192)
            .unwrap();

        let palette: [u32; 16] = [
            0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xAA5500, 0xAAAAAA,
            0x555555, 0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF,
        ];

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..192 {
                    for x in 0..256 {
                        let screen_offset = (y * 256 + x) * 4;
                        let color_offset = y * 256 + x;
                        let color = screen_buffer[color_offset];
                        let color_bytes = palette[color as usize].to_le_bytes();
                        buffer[screen_offset..screen_offset + 4].copy_from_slice(&color_bytes);
                    }
                }
            })
            .unwrap();

        self.canvas.clear();
        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present();
    }
}
