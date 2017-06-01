mod tile;

use std::fs::File;
use std::str::FromStr;
use std::io::{Read, Seek, SeekFrom, BufReader};

use byteorder::{NativeEndian, ReadBytesExt};

pub use self::tile::Tile;

// FIXME make configurable or associated constants?
const PAGE_SIZE: usize = 256;

#[derive(Debug)]
pub struct StyleFile {
    // FIXME remove pub
    pub header: StyleFileHeader,
    pub tiles: Vec<Tile>,
    // TODO maybe use a HashMap for palette index and physical palettes
    pub palette_index: PaletteIndex,
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
            palette_index: chunks.palette_index,
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

enum ChunkBuilderError {
    MissingTilesChunkError,
    MissingPaletteIndexChunkError,
}

struct ChunkBuilder {
    tiles: Option<Vec<Tile>>,
    palette_index: Option<PaletteIndex>,
}

impl ChunkBuilder {
    pub fn new() -> ChunkBuilder {
        ChunkBuilder {
            tiles: None,
            palette_index: None,
        }
    }

    pub fn load_chunk<T: Read + Seek>(&mut self,
                                      chunk_type: ChunkTypes,
                                      size: u32,
                                      mut buf_reader: &mut T)
                                      -> &mut ChunkBuilder {


        match chunk_type {
            ChunkTypes::Tiles => self.tiles(load_tiles(size, buf_reader)),
            ChunkTypes::PaletteIndex => self.palette_index(load_palette_index(size, buf_reader)),
            _ => {
                buf_reader.seek(SeekFrom::Current(size as i64)).unwrap();
                self
            }
        }
    }

    pub fn tiles(&mut self, tiles: Vec<Tile>) -> &mut ChunkBuilder {
        self.tiles = Some(tiles);
        self
    }

    pub fn palette_index(&mut self, palette_index: PaletteIndex) -> &mut ChunkBuilder {
        self.palette_index = Some(palette_index);
        self
    }

    pub fn build(self) -> Result<StyleFileChunks, ChunkBuilderError> {
        let tiles = try!(self.tiles.ok_or(ChunkBuilderError::MissingTilesChunkError));
        let palette_index = try!(self.palette_index
                                     .ok_or(ChunkBuilderError::MissingPaletteIndexChunkError));

        let chunks = StyleFileChunks {
            tiles,
            palette_index,
        };
        Ok(chunks)
    }
}

#[derive(Debug)]
struct StyleFileChunks {
    tiles: Vec<Tile>,
    palette_index: PaletteIndex,
}

#[derive(Debug)]
pub struct PaletteIndex {
    physical_palettes: Vec<u16>,
}

#[derive(Debug)]
pub struct PhysicalPalette {
    colors: Vec<u32>,
}

enum ChunkTypes {
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
#[derive(Debug)]
enum StyleFileParseError {
    UnknownChunkTypeError(String),
}

impl FromStr for ChunkTypes {
    type Err = StyleFileParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PALX" => Ok(ChunkTypes::PaletteIndex),
            "PPAL" => Ok(ChunkTypes::PhysicalPalettes),
            "PALB" => Ok(ChunkTypes::PaletteBase),
            "TILE" => Ok(ChunkTypes::Tiles),
            "SPRG" => Ok(ChunkTypes::SpriteGraphics),
            "SPRX" => Ok(ChunkTypes::SpriteIndex),
            "SPRB" => Ok(ChunkTypes::SpritesBases),
            "DELS" => Ok(ChunkTypes::DeltaStore),
            "DELX" => Ok(ChunkTypes::DeltaIndex),
            "FONB" => Ok(ChunkTypes::FontBase),
            "CARI" => Ok(ChunkTypes::CarInfo),
            "OBJI" => Ok(ChunkTypes::MapObjectInfo),
            "PSXT" => Ok(ChunkTypes::PSXTiles),
            "RECY" => Ok(ChunkTypes::CarRecyclingInfo),
            s => Err(StyleFileParseError::UnknownChunkTypeError(String::from(s))),
        }
    }
}

fn read_chunks<T: Read + Seek>(mut buf_reader: &mut T) -> Option<StyleFileChunks> {
    let mut chunk_builder = ChunkBuilder::new();
    let mut buffer = [0; 4];

    loop {
        buf_reader.read_exact(&mut buffer);

        let chunk_type = match String::from_utf8(buffer.to_vec()) {
            Ok(s) => s,
            Err(_) => break,
        };
        println!("{}", chunk_type);

        let size = match buf_reader.read_u32::<NativeEndian>() {
            Ok(s) => s,
            Err(_) => unimplemented!(),
        };

        // let chunk_type = ChunkTypes::from_str(&chunk_type).unwrap_or_else(|| break);
        let chunk_type = match ChunkTypes::from_str(&chunk_type) {
            Ok(c) => c,
            Err(_) => break,
        };

        chunk_builder.load_chunk(chunk_type, size, &mut buf_reader);
    }

    match chunk_builder.build() {
        Ok(chunk) => Some(chunk),
        Err(_) => None,
    }
}

fn load_tiles<T: Read + Seek>(size: u32, buf_reader: &mut T) -> Vec<Tile> {
    let pages_count = size / (PAGE_SIZE * PAGE_SIZE) as u32;
    let mut tiles: Vec<Tile> = Vec::with_capacity(pages_count as usize * 16); // one page

    for _ in 0..pages_count {
        load_tiles_from_page(&mut tiles, buf_reader);
    }

    tiles
}

fn load_tiles_from_page<T: Read + Seek>(tiles: &mut Vec<Tile>, buf_reader: &mut T) {
    let page = load_page(buf_reader);

    for id in 0..16 {
        let tile = Tile::load_from_page(id, &page);
        tiles.push(tile);
    }
}

fn load_page<T: Read + Seek>(buf_reader: &mut T) -> Vec<u8> {
    let mut page = vec![0; PAGE_SIZE * PAGE_SIZE];

    for pixel in page.iter_mut() {
        *pixel = buf_reader.read_u8().unwrap();
    }

    page
}

fn load_palette_index<T: Read + Seek>(size: u32, buf_reader: &mut T) -> PaletteIndex {
    let size = (size / 2) as usize;
    println!("{}", size);

    let mut physical_palettes = Vec::with_capacity(size);

    for _ in 0..size {
        physical_palettes.push(buf_reader.read_u16::<NativeEndian>().unwrap());
    }

    PaletteIndex { physical_palettes }
}
