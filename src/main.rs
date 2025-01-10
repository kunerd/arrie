extern crate sdl2;

extern crate gta2_viewer;

use std::fs::File;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, Palette, PixelFormatEnum};

use gta2_viewer::StyleFile;
use sdl2::surface::Surface;

const IMAGE_SIZE: usize = 64;

const DATA_PATH: &str = "data/bil.sty";

fn main() {
    let file = File::open(DATA_PATH).unwrap();
    let mut style = StyleFile::from_file(&file);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 1024, 1024)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    //canvas.set_draw_color(Color::WHITE);
    //canvas.fill_rect(None).unwrap();
    let texture_creator = canvas.texture_creator();

    let info = canvas.info();
    dbg!(info.texture_formats);

    let mut tile_iter = style.tiles.iter().enumerate();

    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => match tile_iter.next() {
                    Some((index, tile)) => {
                        let palette_index = style.palette_index.physical_index.get(index).unwrap();

                        let phys_palette =
                            style.physical_palette.get_mut(*palette_index as usize).unwrap();

                        let mut colored_image: Vec<u8> = tile
                            .0
                            .iter()
                            .map(|p| *phys_palette.colors.get(*p as usize).unwrap())
                            .flat_map(|c| c.to_ne_bytes())
                            .collect();

                        let mut surface = Surface::from_data(
                            colored_image.as_mut_slice(),
                            IMAGE_SIZE as u32,
                            IMAGE_SIZE as u32,
                            IMAGE_SIZE as u32 * 4,
                            PixelFormatEnum::BGRA8888,
                        )
                        .unwrap();
                        surface.set_color_key(true, Color::RGBA(0, 0, 0, 0)).unwrap();

                        let texture = surface.as_texture(&texture_creator).unwrap();
                        dbg!(texture.query());
                        canvas.clear();
                        canvas.copy(&texture, None, None).expect("Render failed");
                        canvas.present();
                    }
                    None => break 'mainloop,
                },
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }

        //canvas.present();
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn map_color(v: &u32) -> Color {
    let b = (v & 0xff000000) >> 24;
    let g = (v & 0x0000ff00) >> 8;
    let r = (v & 0x00ff0000) >> 16;
    let a = (v & 0x000000ff) >> 0;

    Color::RGBA(r as u8, g as u8, b as u8, a as u8)
}
