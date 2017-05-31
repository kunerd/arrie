extern crate sdl2;

extern crate gta2_viewer;

use std::fs::File;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};

use gta2_viewer::StyleFile;
use gta2_viewer::Tile;

const PAGE_SIZE: usize = 256;
const IMAGE_SIZE: usize = 64;

fn main() {
    let file = File::open("data/bil.sty").unwrap();

    let style = StyleFile::from_file(&file);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut windows = Vec::new();

    for &Tile(ref tile) in style.tiles.iter().take(16) {
        windows.push(show_tile(&video_subsystem, tile));
    }

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Option::Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {}
            }
        }
    }
}

pub fn show_tile(video_subsystem: &sdl2::VideoSubsystem, tile: &[u8]) -> sdl2::video::Window {
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 512, 512)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let mut texture = texture_creator
        .create_texture_static(Some(PixelFormatEnum::RGB332),
                               IMAGE_SIZE as u32,
                               IMAGE_SIZE as u32)
        .unwrap();
    texture.update(None, tile, IMAGE_SIZE);

    canvas.copy(&texture, None, None).expect("Render failed");
    canvas.present();

    canvas.into_window()
}
