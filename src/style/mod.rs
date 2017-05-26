use std::fmt;
use std::fs::File;
use std::str::FromStr;
use std::io::{Read, Seek, SeekFrom, BufReader};

use byteorder::{NativeEndian, ReadBytesExt};

// FIXME make configurable or associated constants?
const PAGE_SIZE: usize = 256;
const IMAGE_SIZE: usize = 64;

#[derive(Debug)]
pub struct StyleFile {
    pub header: StyleFileHeader,
    pub tiles: Vec<Vec<u8>>,
}

impl StyleFile {
    pub fn from_file(file: &File) -> StyleFile {
        let mut buf_reader = BufReader::new(file);

        let header = read_header(&mut buf_reader);
        let chunks = match read_chunks(&mut buf_reader) {
            Some(c) => c,
            None => panic!("Error while reading chunks."),
        };

        StyleFile {
            header,
            tiles: chunks.tiles,
        }
    }
}

#[derive(Debug)]
pub struct StyleFileHeader {
    file_type: String,
    version: u16,
}

fn read_header<T: Read>(buf_reader: &mut T) -> StyleFileHeader {
    let mut buffer = [0; 4];

    buf_reader.read_exact(&mut buffer);
    let file_type = String::from_utf8(buffer.to_vec()).unwrap();

    let version = buf_reader.read_u16::<NativeEndian>().unwrap();

    StyleFileHeader { file_type, version }
}

#[derive(Debug)]
struct StyleFileChunks {
    tiles: Vec<Vec<u8>>,
}

#[derive(Debug)]
struct PaletteIndex {
    physical_palettes: Vec<u16>,
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
    CarRecyclingInfo,
}

// FIXME rename to something more expressive
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
            _ => Err(StyleChunkParseError()),
        }
    }
}

fn read_chunks<T: Read + Seek>(mut buf_reader: &mut T) -> Option<StyleFileChunks> {
    let mut buffer = [0; 4];

    loop {
        buf_reader.read_exact(&mut buffer);

        let chunk_type = match String::from_utf8(buffer.to_vec()) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let size = match buf_reader.read_u32::<NativeEndian>() {
            Ok(s) => s,
            Err(_) => return None,
        };

        match StyleFileChunkTypes::from_str(&chunk_type) {
            Ok(StyleFileChunkTypes::Tiles) => {
                let tiles = load_tiles(&mut buf_reader);
                return Some(StyleFileChunks { tiles });
            }
            Ok(_) => {}
            Err(_) => println!("Tile parse error."),
        }

        buf_reader.seek(SeekFrom::Current(size as i64)).unwrap();
    }
}

fn load_tiles<T: Read + Seek>(buf_reader: &mut T) -> Vec<Vec<u8>> {
    let mut tiles: Vec<Vec<u8>> = Vec::with_capacity(16); // one page

    // load page
    let mut page: [u8; PAGE_SIZE * PAGE_SIZE] = [0; PAGE_SIZE * PAGE_SIZE];
    for pixel in page.iter_mut() {
        *pixel = buf_reader.read_u8().unwrap();
    }
    let page = page;

    for id in 0..16 {
        let y_start = (id / 4) * IMAGE_SIZE;
        let y_end = y_start + IMAGE_SIZE;
        let mut tile = Vec::with_capacity(IMAGE_SIZE * IMAGE_SIZE);

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
