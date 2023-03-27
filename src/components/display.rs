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

    pub fn update_screen(&mut self, screen_buffer: &[Vec<u32>]) {
        let mut texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, 320, 192)
            .unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for (y, row) in screen_buffer.iter().enumerate() {
                    for (x, &color) in row.iter().enumerate() {
                        let offset = y * pitch + x * 4;
                        let color_bytes = color.to_le_bytes();
                        buffer[offset..offset + 4].copy_from_slice(&color_bytes);
                    }
                }
            })
            .unwrap();

        self.canvas.clear();
        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present();
    }
}
