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

    let style = StyleFile::from_file(&file);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 512, 512)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();
    canvas.clear();

    //let mut windows = Vec::new();

    //for Tile(ref tile) in style.tiles.iter().take(16) {
    //    windows.push(show_tile(&video_subsystem, tile));
    //}

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut tile_iter = style.tiles.iter().enumerate();
    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => match tile_iter.next() {
                    Some((index, tile)) => {
                        //let _ = texture.update(None, &tile.0, IMAGE_SIZE);
                        let palette_base = style.palette_base.tile;
                        let virtual_palette_index = index + palette_base as usize;
                        let palette_index = style
                            .palette_index
                            .physical_index
                            .get(index)
                            .unwrap();
                        let phys_palette =
                            style.physical_palette.get(*palette_index as usize).unwrap();
                        let palette: Vec<_> = phys_palette.colors.iter().map(map_color).collect();
                        let palette = Palette::with_colors(&palette).unwrap();
                        //surface.set_palette(&palette).unwrap();
                        let mut surface = Surface::new(
                            IMAGE_SIZE as u32,
                            IMAGE_SIZE as u32,
                            PixelFormatEnum::RGB24,
                        )
                        .unwrap();
                        surface.set_palette(&palette).unwrap();

                        let mut texture = texture_creator
                            .create_texture_from_surface(surface)
                            .unwrap();
                        texture.update(None, &tile.0, IMAGE_SIZE).unwrap();

                        canvas.copy(&texture, None, None).expect("Render failed");
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

        canvas.present();
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn map_color(v: &u32) -> Color {
    let r = (v & 0xff000000) >> 24;
    let g = (v & 0x00ff0000) >> 16;
    let b = (v & 0x0000ff00) >> 8;
    let a = (v & 0x000000ff) >> 0;

    Color::RGBA(r as u8, g as u8, b as u8, a as u8)
}
