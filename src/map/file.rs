use std::{
    f32::consts::TAU, fs::File, io::{BufReader, Cursor, Read, Seek, SeekFrom}, path::Path, str::FromStr
};

use byteorder::{NativeEndian, ReadBytesExt};

#[derive(Debug)]
pub struct FileHeader {
    pub file_type: String,
    pub version: u16,
}

#[derive(Debug)]
pub struct Map {
    pub uncompressed_map: Option<UncompressedMap>,
    //compressed_map_16bit: CompressedMap,
    pub compressed_map_32bit: CompressedMap32,
    //zones: Vec<Zone>,
    //objects: Vec<Object>,
    //psx_mapping_table: PsxMappingTable,
    //tile_animations: Vec<TileAnimation>,
    //lights: Vec<Light>,
    //junctions: Vec<Junction>
}

#[derive(Debug)]
pub struct UncompressedMap(pub Vec<BlockInfo>);

const BASE_ARRAY_SIZE: usize = 256 * 256;

#[derive(Debug, Clone)]
pub struct CompressedMap32 {
    base: Vec<u32>,
    column_infos: Vec<u32>,
    block_infos: Vec<BlockInfo>,
}

#[derive(Debug, Clone, Copy)]
pub enum Rotate {
    Degree0,
    Degree90,
    Degree180,
    Degree270,
}

impl Rotate {
    pub fn clockwise_rad(&self) -> f32 {
        let fraction = match self {
            Rotate::Degree0 => 0.0,
            Rotate::Degree90 => 0.25,
            Rotate::Degree180 => 0.5,
            Rotate::Degree270 => 0.75,
        };

        TAU * fraction
    }
}

impl From<u8> for Rotate {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Degree0,
            1 => Self::Degree90,
            2 => Self::Degree180,
            3 => Self::Degree270,
            _ => panic!("Rotation not supported"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalFace {
    pub tile_id: usize,
    pub flat: bool,
    pub flip: bool,
    pub rotate: Rotate,
}

impl From<u16> for NormalFace {
    fn from(value: u16) -> Self {
        let tile_id = (value & 0b0000_0011_1111_1111) as usize;
        let flat = ((value >> 12) & 0x01) == 1;
        let flip = ((value >> 13) & 0x01) == 1;
        let rotate = value >> 14;
        let rotate = Rotate::from(rotate as u8);

        Self {
            tile_id,
            flat,
            flip,
            rotate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LidFace {
    pub tile_id: usize,
    pub flat: bool,
    pub flip: bool,
    pub rotate: Rotate,
}

impl From<u16> for LidFace {
    fn from(value: u16) -> Self {
        let tile_id = (value & 0b0000_0011_1111_1111) as usize;
        let flat = ((value >> 12) & 0x01) == 1;
        let flip = ((value >> 13) & 0x01) == 1;
        let rotate = value >> 14;
        let rotate = Rotate::from(rotate as u8);

        Self {
            tile_id,
            flat,
            flip,
            rotate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub left: NormalFace,
    pub right: NormalFace,
    pub top: NormalFace,
    pub bottom: NormalFace,
    pub lid: LidFace,
    // TODO: use bitflags
    pub arrows: u8,
    // TODO: use bitflags
    //pub slope_type: u8,
    pub slope_type: SlopeType,
}

#[derive(Debug, Clone)]
pub enum SlopeType {
    None,
    Degree7 {
        direction: SlopeDirection,
        level: SlopeLevel,
    },
    Degree26 {
        direction: SlopeDirection,
        level: SlopeLevel,
    },
    Degree45(SlopeDirection),
    Diagonal(DiagonalType),
    ThreeSidedDiagonal(DiagonalType),
    FourSidedDiagonal(DiagonalType),
    PartialBlock(PartialPosition),
    PartialCornerBlock,
    SlopeAbove,
    Ignore,
}

#[derive(Debug, Clone)]
pub enum SlopeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum SlopeLevel {
    Low,
    High,
}

#[derive(Debug, Clone)]
pub enum DiagonalType {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug, Clone)]
pub enum PartialPosition {
    Left,
    Right,
    Top,
    Bottom,
}

impl From<u8> for SlopeType {
    fn from(value: u8) -> Self {
        let slope_type_id = value >> 2;
        match slope_type_id {
            0 => Self::None,
            1 => Self::Degree26 {
                direction: SlopeDirection::Up,
                level: SlopeLevel::Low,
            },
            2 => Self::Degree26 {
                direction: SlopeDirection::Up,
                level: SlopeLevel::High,
            },
            3 => Self::Degree26 {
                direction: SlopeDirection::Down,
                level: SlopeLevel::Low,
            },
            4 => Self::Degree26 {
                direction: SlopeDirection::Down,
                level: SlopeLevel::High,
            },
            5 => Self::Degree26 {
                direction: SlopeDirection::Left,
                level: SlopeLevel::Low,
            },
            6 => Self::Degree26 {
                direction: SlopeDirection::Left,
                level: SlopeLevel::High,
            },
            7 => Self::Degree26 {
                direction: SlopeDirection::Right,
                level: SlopeLevel::Low,
            },
            8 => Self::Degree26 {
                direction: SlopeDirection::Right,
                level: SlopeLevel::High,
            },
            41 => Self::Degree45(SlopeDirection::Up),
            42 => Self::Degree45(SlopeDirection::Down),
            43 => Self::Degree45(SlopeDirection::Left),
            44 => Self::Degree45(SlopeDirection::Right),
            45 => Self::Diagonal(DiagonalType::UpLeft),
            46 => Self::Diagonal(DiagonalType::UpRight),
            47 => Self::Diagonal(DiagonalType::DownLeft),
            48 => Self::Diagonal(DiagonalType::DownRight),
            // FIXME: could also be 4-sided
            49 => Self::ThreeSidedDiagonal(DiagonalType::UpLeft),
            50 => Self::ThreeSidedDiagonal(DiagonalType::UpRight),
            51 => Self::ThreeSidedDiagonal(DiagonalType::DownLeft),
            52 => Self::ThreeSidedDiagonal(DiagonalType::DownRight),
            53 => Self::PartialBlock(PartialPosition::Left),
            54 => Self::PartialBlock(PartialPosition::Right),
            55 => Self::PartialBlock(PartialPosition::Top),
            56 => Self::PartialBlock(PartialPosition::Bottom),
            //57 => Self::PartialBlock(PartialPosition::TopLeft),
            //58 => Self::PartialBlock(PartialPosition::TopRight),
            //59 => Self::PartialBlock(PartialPosition::BottomRight),
            //60 => Self::PartialBlock(PartialPosition::BottomLeft),
            63 => SlopeType::SlopeAbove,
            _ => SlopeType::Ignore,
        }
    }
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
    compressed_map_32: Option<CompressedMap32>,
}

impl MapBuilder {
    pub fn new() -> MapBuilder {
        MapBuilder {
            uncompressed_map: None,
            compressed_map_32: None,
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
            ChunkTypes::CompressedMap32Bit => {
                self.compressed_map_32 = Some(load_compressed_map_32(size, buf_reader));
                self
            }
            _ => {
                buf_reader.seek(SeekFrom::Current(size as i64)).unwrap();
                self
            }
        }
    }

    pub fn build(self) -> Option<Map> {
        let uncompressed_map =
            create_uncompressed_map_from_compressed(self.compressed_map_32.clone()?);

        Some(Map {
            uncompressed_map: Some(uncompressed_map),
            compressed_map_32bit: self.compressed_map_32?,
        })
    }
}

fn create_uncompressed_map_from_compressed(compressed: CompressedMap32) -> UncompressedMap {
    let base: &[u32] = &compressed.base;
    let columns: &[u32] = &compressed.column_infos;

    let mut block_infos =
        Vec::with_capacity(UncompressedMap::X * UncompressedMap::Y * UncompressedMap::Z);

    for _ in 0..UncompressedMap::X * UncompressedMap::Y * UncompressedMap::Z {
        block_infos.push(compressed.block_infos.first().unwrap().clone());
    }

    for x in 0..256 {
        for y in 0..256 {
            let col_index = base[y * 256 + x] as usize;

            let col_info = columns[col_index];
            let height = (col_info & 0xff) as usize;
            let offset = ((col_info & 0xff00) >> 8) as usize;

            for _ in 0..offset {
                block_infos.push(compressed.block_infos.first().unwrap().clone());
            }

            for z in 0..height {
                if z >= offset {
                    let block_info = compressed
                        .block_infos
                        .get(columns[col_index + z - offset + 1] as usize)
                        .unwrap();

                    if let Some(block) = block_infos.get_mut((y * 256 + x) + z * 256 * 256) {
                        *block = block_info.clone()
                    }
                }
            }
        }
    }

    //assert_eq!(
    //    block_infos.len(),
    //    UncompressedMap::X * UncompressedMap::Y * UncompressedMap::Z
    //);

    UncompressedMap(block_infos)
}

const BLOCK_INFO_SIZE: u32 = 12;
fn load_uncompressed_map<T: Read + Seek>(size: u32, buf_reader: &mut T) -> UncompressedMap {
    let blocks_count = UncompressedMap::X * UncompressedMap::Y * UncompressedMap::Z;
    assert!(blocks_count as u32 * BLOCK_INFO_SIZE == size);

    let blocks = read_block_infos(blocks_count, buf_reader);
    UncompressedMap(blocks)
}

fn load_compressed_map_32<T: Read + Seek>(_size: u32, buf_reader: &mut T) -> CompressedMap32 {
    let mut base = Vec::with_capacity(BASE_ARRAY_SIZE);
    for _ in 0..BASE_ARRAY_SIZE {
        base.push(buf_reader.read_u32::<NativeEndian>().unwrap());
    }

    let column_info_len = buf_reader.read_u32::<NativeEndian>().unwrap();
    let mut column_infos = Vec::with_capacity(column_info_len as usize);
    for _ in 0..column_info_len {
        //let column_info = read_column_info(buf_reader);
        column_infos.push(buf_reader.read_u32::<NativeEndian>().unwrap());
    }

    let block_info_len = buf_reader.read_u32::<NativeEndian>().unwrap();
    let block_infos = read_block_infos(block_info_len as usize, buf_reader);

    CompressedMap32 {
        base,
        column_infos,
        block_infos,
    }
}

fn read_block_infos<T: Read + Seek>(len: usize, buf_reader: &mut T) -> Vec<BlockInfo> {
    let mut blocks = Vec::with_capacity(len);

    for _ in 0..len {
        let block = BlockInfo {
            left: buf_reader.read_u16::<NativeEndian>().unwrap().into(),
            right: buf_reader.read_u16::<NativeEndian>().unwrap().into(),
            top: buf_reader.read_u16::<NativeEndian>().unwrap().into(),
            bottom: buf_reader.read_u16::<NativeEndian>().unwrap().into(),
            lid: buf_reader.read_u16::<NativeEndian>().unwrap().into(),
            arrows: buf_reader.read_u8().unwrap(),
            slope_type: SlopeType::from(buf_reader.read_u8().unwrap()),
        };

        blocks.push(block);
    }

    blocks
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
