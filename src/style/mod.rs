mod tile;

<<<<<<< HEAD
use std::convert::{TryFrom, TryInto};
=======
use std::convert::TryInto;
>>>>>>> 2346cc8 (Add color lookup from palletes)
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::str::FromStr;
use std::string;

use byteorder::{NativeEndian, ReadBytesExt};

pub use self::tile::Tile;

// FIXME make configurable or associated constants?
const PAGE_SIZE: usize = 256;

#[derive(Debug)]
pub struct StyleFileHeader {
    file_type: String,
    version: u16,
}

#[derive(Debug)]
pub struct StyleFile {
    // FIXME remove pub
    pub header: StyleFileHeader,
    pub tiles: Vec<Tile>,
    pub palette_index: PaletteIndex,
    pub palette_base: PaletteBase,
    pub physical_palette: Vec<PhysicalPalette>,
    //pub palette_base: PaletteBase,
    // TODO maybe use a HashMap for palette index and physical palettes
}

impl StyleFile {
    pub fn from_file(file: &File) -> StyleFile {
        let mut buf_reader = BufReader::new(file);

        let header = read_header(&mut buf_reader).unwrap();
        let chunks = match read_chunks(&mut buf_reader) {
            Some(c) => c,
            None => panic!("Error while reading chunks."),
        };

        StyleFile {
            header,
            tiles: chunks.tiles,
            palette_index: chunks.palette_index,
            palette_base: chunks.palette_base,
            physical_palette: chunks.physical_palettes,
        }
    }
}

#[derive(Debug)]
enum ParseError {
    Io(io::Error),
    FileType(string::FromUtf8Error),
    UnknownChunkTypeError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Io(err) => err.fmt(f),
            ParseError::FileType(err) => err.fmt(f),
            ParseError::UnknownChunkTypeError(t) => {
                write!(f, "Unknown chunk type: {}", t)
            }
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            ParseError::Io(err) => err.source(),
            ParseError::FileType(err) => err.source(),
            ParseError::UnknownChunkTypeError(_) => None,
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::Io(err)
    }
}

impl From<string::FromUtf8Error> for ParseError {
    fn from(err: string::FromUtf8Error) -> Self {
        ParseError::FileType(err)
    }
}

fn read_header<T: Read>(buf_reader: &mut T) -> Result<StyleFileHeader, ParseError> {
    let mut buffer = [0; 4];

    buf_reader.read_exact(&mut buffer)?;
    let file_type = String::from_utf8(buffer.to_vec())?;
    let version = buf_reader.read_u16::<NativeEndian>()?;

    dbg!(&file_type, &version);

    Ok(StyleFileHeader { file_type, version })
}

enum ChunkBuilderError {
    MissingTilesChunkError,
    MissingPaletteIndexChunkError,
    MissingPhysicalPalettesChunk,
    MissingPaletteBase,
}

struct ChunkBuilder {
    tiles: Option<Vec<Tile>>,
    palette_index: Option<PaletteIndex>,
    palette_base: Option<PaletteBase>,
    physical_palette: Option<Vec<PhysicalPalette>>,
}

impl ChunkBuilder {
    pub fn new() -> ChunkBuilder {
        ChunkBuilder {
            tiles: None,
            palette_index: None,
            palette_base: None,
            physical_palette: None,
        }
    }

    pub fn load_chunk<T: Read + Seek>(
        &mut self,
        chunk_type: ChunkTypes,
        size: u32,
        buf_reader: &mut T,
    ) -> &mut ChunkBuilder {
        match chunk_type {
            ChunkTypes::Tiles => self.tiles(load_tiles(size, buf_reader)),
            ChunkTypes::PhysicalPalettes => {
                self.physical_palettes(load_physical_palettes(size, buf_reader))
            }
            ChunkTypes::PaletteBase => self.palette_base(load_palette_base(size, buf_reader)),
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

    pub fn physical_palettes(
        &mut self,
        physical_palettes: Vec<PhysicalPalette>,
    ) -> &mut ChunkBuilder {
        self.physical_palette = Some(physical_palettes);
        self
    }

    pub fn palette_index(&mut self, palette_index: PaletteIndex) -> &mut ChunkBuilder {
        self.palette_index = Some(palette_index);
        self
    }

    pub fn build(self) -> Result<StyleFileChunks, ChunkBuilderError> {
        let tiles = self
            .tiles
            .ok_or(ChunkBuilderError::MissingTilesChunkError)?;

        let palette_index = self
            .palette_index
            .ok_or(ChunkBuilderError::MissingPaletteIndexChunkError)?;

        let palette_base = self
            .palette_base
            .ok_or(ChunkBuilderError::MissingPaletteBase)?;

        let physical_palettes = self
            .physical_palette
            .ok_or(ChunkBuilderError::MissingPhysicalPalettesChunk)?;

        let chunks = StyleFileChunks {
            tiles,
            palette_base,
            palette_index,
            physical_palettes,
        };

        Ok(chunks)
    }

    fn palette_base(&mut self, palette_base: PaletteBase) -> &mut ChunkBuilder {
        self.palette_base = Some(palette_base);
        self
    }
}

#[derive(Debug)]
struct StyleFileChunks {
    tiles: Vec<Tile>,
    palette_index: PaletteIndex,
    palette_base: PaletteBase,
    physical_palettes: Vec<PhysicalPalette>,
}

#[derive(Debug)]
pub struct PaletteIndex {
    pub physical_index: Vec<u16>,
}

#[derive(Debug)]
pub struct PaletteBase {
    pub tile: u16,
    pub sprite: u16,
    pub car_remap: u16,
    pub ped_remap: u16,
    pub code_obj_remap: u16,
    pub map_opj_remap: u16,
    pub user_remap: u16,
    pub font_remap: u16,
}

#[derive(Debug)]
pub struct PhysicalPalette {
    pub colors: Vec<u32>,
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

impl FromStr for ChunkTypes {
    type Err = ParseError;

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
            s => Err(ParseError::UnknownChunkTypeError(s.to_string())),
        }
    }
}

fn read_chunks<T: Read + Seek>(mut buf_reader: &mut T) -> Option<StyleFileChunks> {
    let mut buffer = [0; 4];
    let mut chunk_builder = ChunkBuilder::new();

    loop {
        buf_reader.read_exact(&mut buffer).unwrap();

        let chunk_type = match String::from_utf8(buffer.to_vec()) {
            Ok(s) => s,
            Err(_) => break,
        };

        let size = match buf_reader.read_u32::<NativeEndian>() {
            Ok(s) => s,
            Err(_) => unimplemented!(),
        };

        println!("{}: {} bytes", chunk_type, size);

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

const TILES_PER_PAGE: usize = 16;

fn load_tiles<T: Read + Seek>(size: u32, buf_reader: &mut T) -> Vec<Tile> {
    let pages_count = size / (PAGE_SIZE * PAGE_SIZE) as u32;
    let mut tiles: Vec<Tile> = Vec::with_capacity(pages_count as usize * TILES_PER_PAGE);

    for _ in 0..pages_count {
        load_tiles_from_page(&mut tiles, buf_reader);
    }

    dbg!(tiles.len());
    tiles
}

fn load_tiles_from_page<T: Read + Seek>(tiles: &mut Vec<Tile>, buf_reader: &mut T) {
    let page = load_page(buf_reader);

<<<<<<< HEAD
    for row in 0..4 {
        let mut tile1 = Vec::with_capacity(64 * 64);
        let mut tile2 = Vec::with_capacity(64 * 64);
        let mut tile3 = Vec::with_capacity(64 * 64);
        let mut tile4 = Vec::with_capacity(64 * 64);

        for line in 0..64 {
            let start = row * 64 * 256 + line * 256;
            let end = start + 256;
            dbg!(end);
            let line = &page[start..end];
            tile1.extend_from_slice(&line[0..64]);
            tile2.extend_from_slice(&line[64..128]);
            tile3.extend_from_slice(&line[128..192]);
            tile4.extend_from_slice(&line[192..256]);
        }

        tiles.append(&mut vec![
            Tile(tile1),
            Tile(tile2),
            Tile(tile3),
            Tile(tile4),
        ]);
=======
    for id in 0..TILES_PER_PAGE {
        tiles.push(Tile::from_file(id, &page));
>>>>>>> 2346cc8 (Add color lookup from palletes)
    }
    //for row in 0..4 {
    //    let mut tile1 = Vec::with_capacity(64 * 64);
    //    let mut tile2 = Vec::with_capacity(64 * 64);
    //    let mut tile3 = Vec::with_capacity(64 * 64);
    //    let mut tile4 = Vec::with_capacity(64 * 64);

    //    for line in 0..64 {
    //        let start = row * 64 * 256 + line * 256;
    //        let end = start + 256;
    //        dbg!(end);
    //        let line = &page[start..end];
    //        tile1.extend_from_slice(&line[0..64]);
    //        tile2.extend_from_slice(&line[64..128]);
    //        tile3.extend_from_slice(&line[128..192]);
    //        tile4.extend_from_slice(&line[192..256]);
    //    }

    //    tiles.append(&mut vec![
    //        Tile(tile1),
    //        Tile(tile2),
    //        Tile(tile3),
    //        Tile(tile4),
    //    ]);
    //}
}

fn load_page<T: Read + Seek>(buf_reader: &mut T) -> Vec<u8> {
    let mut page = [0; PAGE_SIZE * PAGE_SIZE];

    for v in page.iter_mut() {
        *v = buf_reader.read_u8().unwrap();
    }

    page.to_vec()
}

fn load_palette_index<T: Read + Seek>(size: u32, buf_reader: &mut T) -> PaletteIndex {
    let size = (size / 2) as usize;

    let mut physical_palettes = Vec::with_capacity(size);

    for _ in 0..size {
        physical_palettes.push(buf_reader.read_u16::<NativeEndian>().unwrap());
    }

    PaletteIndex {
        physical_index: physical_palettes,
    }
}

fn load_physical_palettes<T: Read + Seek>(size: u32, buf_reader: &mut T) -> Vec<PhysicalPalette> {
    const PALETTES_PER_PAGE: usize = 64;
    let pages_count = size / (PAGE_SIZE * PAGE_SIZE) as u32;
    let mut palettes: Vec<PhysicalPalette> =
        Vec::with_capacity(pages_count as usize * PALETTES_PER_PAGE);

    for _ in 0..pages_count {
        let page = load_page(buf_reader);
        for id in 0..PALETTES_PER_PAGE {
            palettes.push(load_phys_palette_from_page(id, &page));
        }
    }

    palettes
}

fn load_phys_palette_from_page(id: usize, page: &[u8]) -> PhysicalPalette {
    let y_end = PAGE_SIZE;
    let x_start = id * 4;

    let mut colors = Vec::with_capacity(1024);
    for y in 0..y_end {
        let index = (y * PAGE_SIZE) + x_start;
        let chunk: [u8; 4] = page[index..index + 4].try_into().unwrap();
        colors.push(u32::from_ne_bytes(chunk));
    }
    PhysicalPalette { colors }
}

fn load_palette_base<T: Read + Seek>(size: u32, buf_reader: &mut T) -> PaletteBase {
    PaletteBase {
        tile: buf_reader.read_u16::<NativeEndian>().unwrap(),
        sprite: buf_reader.read_u16::<NativeEndian>().unwrap(),
        car_remap: buf_reader.read_u16::<NativeEndian>().unwrap(),
        ped_remap: buf_reader.read_u16::<NativeEndian>().unwrap(),
        code_obj_remap: buf_reader.read_u16::<NativeEndian>().unwrap(),
        map_opj_remap: buf_reader.read_u16::<NativeEndian>().unwrap(),
        user_remap: buf_reader.read_u16::<NativeEndian>().unwrap(),
        font_remap: buf_reader.read_u16::<NativeEndian>().unwrap(),
    }
}
