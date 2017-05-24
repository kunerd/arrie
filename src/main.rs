extern crate byteorder;
extern crate piston_window;
// extern crate find_folder;
extern crate sdl2;

use std::env;
use sdl2::image::{LoadTexture, INIT_PNG, INIT_JPG};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Texture;

use piston_window::*;
use std::path::Path;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::fmt;

use byteorder::{NativeEndian, ReadBytesExt};

use std::str::FromStr;

#[derive(Debug)]
struct StyleFile {
    header: StyleFileHeader,
    chunks: StyleFileChunks
}

#[derive(Debug)]
struct StyleFileHeader {
    file_type: String,
    version: u16
}

#[derive(Debug)]
struct StyleFileChunks {
    tiles: Vec<Vec<u8>>
}

impl StyleFile {
    fn from_file(file: &File) -> StyleFile {
        let mut buf_reader = BufReader::new(file);

        let header = read_header(&mut buf_reader);
        let chunks = match read_chunks(&mut buf_reader) {
            Some(c) => c,
            None => panic!("Error while reading chunks.")
        };

        StyleFile {
            header,
            chunks
        }
    }
}

fn read_header<T: Read>(buf_reader: &mut T) -> StyleFileHeader {
    let mut buffer = [0; 4];

    buf_reader.read_exact(&mut buffer);
    let file_type = String::from_utf8(buffer.to_vec()).unwrap();

    let version = buf_reader.read_u16::<NativeEndian>().unwrap();

    StyleFileHeader {
        file_type,
        version
    }
}

enum StyleFileChunkTypes {
    PaletteIndex,
    PhysicalPalettes,
    PaletteBase,
    Tiles,
    SpriteGraphics,
    SpriteIndex,
    SpritesBases,
    DeltaStore,
    DeltaIndex,
    FontBase,
    CarInfo,
    MapObjectInfo,
    PSXTiles,
    CarRecyclingInfo
}

struct StyleChunkParseError();

impl FromStr for StyleFileChunkTypes {
    type Err = StyleChunkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PALX" => Ok(StyleFileChunkTypes::PaletteIndex),
            "PPAL" => Ok(StyleFileChunkTypes::PhysicalPalettes),
            "PALB" => Ok(StyleFileChunkTypes::PaletteBase),
            "TILE" => Ok(StyleFileChunkTypes::Tiles),
            "SPRG" => Ok(StyleFileChunkTypes::SpriteGraphics),
            "SPRX" => Ok(StyleFileChunkTypes::SpriteIndex),
            "SPRB" => Ok(StyleFileChunkTypes::SpritesBases),
            "DELS" => Ok(StyleFileChunkTypes::DeltaStore),
            "DELX" => Ok(StyleFileChunkTypes::DeltaIndex),
            "FONB" => Ok(StyleFileChunkTypes::FontBase),
            "CARI" => Ok(StyleFileChunkTypes::CarInfo),
            "OBJI" => Ok(StyleFileChunkTypes::MapObjectInfo),
            "PSXT" => Ok(StyleFileChunkTypes::PSXTiles),
            "RECY" => Ok(StyleFileChunkTypes::CarRecyclingInfo),
            _ => Err(StyleChunkParseError())
        }
    }
}

fn read_chunks<T: Read + Seek>(mut buf_reader: &mut T) -> Option<StyleFileChunks> {
    let mut buffer = [0; 4];

    loop {
        buf_reader.read_exact(&mut buffer);

        let chunk_type = match String::from_utf8(buffer.to_vec()) {
            Ok(s) => s,
            Err(_) => return None
        };

        let size = match buf_reader.read_u32::<NativeEndian>() {
            Ok(s) => s,
            Err(_) => return None
        };

        println!("Chunk-type: {}\nChunk size: {}", chunk_type, size);
        buf_reader.seek(SeekFrom::Current(256*256*7 as i64)).unwrap();

        match StyleFileChunkTypes::from_str(&chunk_type) {
            Ok(StyleFileChunkTypes::Tiles) => {
                let tiles = load_tiles(&mut buf_reader);
                return Some(StyleFileChunks{ tiles })
            },
            Ok(_) => {},
            Err(_) => println!("Tile parse error.")
        }

        buf_reader.seek(SeekFrom::Current(size as i64)).unwrap();
    }
}

const  PAGE_SIZE: usize = 256;
const  IMAGE_SIZE: usize = 64;

fn load_tiles<T: Read + Seek>(buf_reader: &mut T) -> Vec<Vec<u8>> {
    let mut tiles: Vec<Vec<u8>> = Vec::with_capacity(16); // one page

    // load page
    let mut page: [u8; PAGE_SIZE*PAGE_SIZE] = [0; PAGE_SIZE*PAGE_SIZE];
    for pixel in page.iter_mut() {
        *pixel = buf_reader.read_u8().unwrap();
    }
    let page = page;

    for id in 0..16 {
        let y_start = (id / 4) * IMAGE_SIZE;
        let y_end = y_start + IMAGE_SIZE;
        let mut tile = Vec::with_capacity(IMAGE_SIZE*IMAGE_SIZE);

        for y in y_start..y_end {
            let x_start = (id % 4) * IMAGE_SIZE;
            let x_end = x_start + IMAGE_SIZE;

            for x in x_start..x_end {
                tile.push(page[(y * PAGE_SIZE) + x]);
            }
        }
        tiles.push(tile);
    }

    tiles
}

impl fmt::Display for StyleFileHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "File type: {}\nVersion: {}", self.file_type, self.version)
    }
}

impl fmt::Display for StyleFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Header:\n{}", self.header)
    }
}

fn main() {
    let file = File::open("data/bil.sty").unwrap();
    // let file = File::open("data/MP1-comp.gmp").unwrap();

    let style = StyleFile::from_file(&file);


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut windows = Vec::new();

    for tile in style.chunks.tiles {
        windows.push(show_tile(&video_subsystem, &tile));
    }

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                    break 'mainloop,
                _ => {}
            }
        }
    }
}

pub fn show_tile(video_subsystem: &sdl2::VideoSubsystem, tile: &[u8]) -> sdl2::video::Window {
    let window = video_subsystem.window("rust-sdl2 demo: Video", 512, 512)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let mut texture = texture_creator.create_texture_static(
        Some(PixelFormatEnum::RGB332),
        IMAGE_SIZE as u32,
        IMAGE_SIZE as u32
    ).unwrap();
    texture.update(None, tile, IMAGE_SIZE);

    canvas.copy(&texture, None, None).expect("Render failed");
    canvas.present();

    canvas.into_window()
}
