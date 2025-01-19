use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
    str::FromStr,
};

use byteorder::{NativeEndian, ReadBytesExt};

#[derive(Debug)]
pub struct FileHeader {
    pub file_type: String,
    pub version: u16,
}

pub struct Map {
    pub uncompressed_map: UncompressedMap,
    //compressed_map_16bit: CompressedMap,
    //compressed_map_32bit: CompressedMap,
    //zones: Vec<Zone>,
    //objects: Vec<Object>,
    //psx_mapping_table: PsxMappingTable,
    //tile_animations: Vec<TileAnimation>,
    //lights: Vec<Light>,
    //junctions: Vec<Junction>
}

pub struct UncompressedMap(pub Vec<BlockInfo>);

#[derive(Debug)]
pub struct BlockInfo {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
    pub lid: u16,
    // TODO: use bitflags
    pub arrows: u8,
    // TODO: use bitflags
    pub slope_type: u8,
}

impl Map {
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        let file = File::open(path).unwrap();
        let mut buf_reader = BufReader::new(file);

        let _header = read_header(&mut buf_reader);

        read_chunks(&mut buf_reader)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut cursor = Cursor::new(bytes);

        let header = read_header(&mut cursor);
        dbg!(header);

        read_chunks(&mut cursor)
    }
}

impl UncompressedMap {
    pub const X: usize = 256;
    pub const Y: usize = 256;
    pub const Z: usize = 8;

    pub fn new() -> Self {
        let inner = Vec::with_capacity(Self::X * Self::Y * Self::Z);

        Self(inner)
    }
}

enum ChunkTypes {
    UncompressedMap,
    CompressedMap16Bit,
    CompressedMap32Bit,
    MapZones,
    MapObjects,
    PsxMappingTable,
    TileAnimation,
    Lights,
    JunctionList,
}

impl FromStr for ChunkTypes {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UMAP" => Ok(ChunkTypes::UncompressedMap),
            "CMAP" => Ok(ChunkTypes::CompressedMap16Bit),
            "DMAP" => Ok(ChunkTypes::CompressedMap32Bit),
            "ZONE" => Ok(ChunkTypes::MapZones),
            "MOBJ" => Ok(ChunkTypes::MapObjects),
            "PSXM" => Ok(ChunkTypes::PsxMappingTable),
            "ANIM" => Ok(ChunkTypes::TileAnimation),
            "LGHT" => Ok(ChunkTypes::Lights),
            "RGEN" => Ok(ChunkTypes::JunctionList),
            s => Err(ParseError::UnknownChunkType(s.to_string())),
        }
    }
}

enum ParseError {
    UnknownChunkType(String),
}

struct MapBuilder {
    uncompressed_map: Option<UncompressedMap>,
}

impl MapBuilder {
    pub fn new() -> MapBuilder {
        MapBuilder {
            uncompressed_map: None,
        }
    }

    pub fn load_chunk<T: Read + Seek>(
        &mut self,
        chunk_type: ChunkTypes,
        size: u32,
        buf_reader: &mut T,
    ) -> &mut MapBuilder {
        match chunk_type {
            ChunkTypes::UncompressedMap => {
                self.uncompressed_map = Some(load_uncompressed_map(size, buf_reader));
                self
            }
            _ => {
                buf_reader.seek(SeekFrom::Current(size as i64)).unwrap();
                self
            }
        }
    }

    pub fn build(self) -> Option<Map> {
        Some(Map {
            uncompressed_map: self.uncompressed_map?,
        })
    }
}

fn load_uncompressed_map<T: Read + Seek>(size: u32, buf_reader: &mut T) -> UncompressedMap {
    const BLOCK_INFO_SIZE: u32 = 12;

    let blocks_count = UncompressedMap::X * UncompressedMap::Y * UncompressedMap::Z;
    assert!(blocks_count as u32 * BLOCK_INFO_SIZE == size);

    let mut blocks = Vec::with_capacity(blocks_count);
    for _ in 0..blocks_count {
        let block = BlockInfo {
            left: buf_reader.read_u16::<NativeEndian>().unwrap(),
            right: buf_reader.read_u16::<NativeEndian>().unwrap(),
            top: buf_reader.read_u16::<NativeEndian>().unwrap(),
            bottom: buf_reader.read_u16::<NativeEndian>().unwrap(),
            lid: buf_reader.read_u16::<NativeEndian>().unwrap(),
            arrows: buf_reader.read_u8().unwrap(),
            slope_type: buf_reader.read_u8().unwrap(),
        };

        blocks.push(block);
    }

    UncompressedMap(blocks)
}

fn read_header<T: Read>(buf_reader: &mut T) -> FileHeader {
    let mut buffer = [0; 4];

    buf_reader.read_exact(&mut buffer).unwrap();
    let file_type = String::from_utf8(buffer.to_vec()).unwrap();
    let version = buf_reader.read_u16::<NativeEndian>().unwrap();

    dbg!(&file_type, &version);

    FileHeader { file_type, version }
}

fn read_chunks<T: Read + Seek>(mut buf_reader: &mut T) -> Map {
    let mut buffer = [0; 4];
    let mut map_builder = MapBuilder::new();

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

        map_builder.load_chunk(chunk_type, size, &mut buf_reader);
    }

    map_builder.build().unwrap()
}
