extern crate byteorder;
extern crate piston_window;
// extern crate find_folder;

use piston_window::*;
use std::path::Path;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::fmt;

use byteorder::{NativeEndian, ReadBytesExt};

use std::str::FromStr;

#[derive(Debug)]
struct StyleFile {
    header: StyleFileHeader
}

#[derive(Debug)]
struct StyleFileHeader {
    file_type: String,
    version: u16
}

impl StyleFile {
    fn from_file(file: &File) -> StyleFile {
        let mut buf_reader = BufReader::new(file);

        let header = read_header(&mut buf_reader);
        read_chunks(&mut buf_reader);

        StyleFile {
            header
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

fn read_chunks<T: Read + Seek>(buf_reader: &mut T) {
    let mut buffer = [0; 4];

    loop {
        buf_reader.read_exact(&mut buffer);

        let chunk_type = match String::from_utf8(buffer.to_vec()) {
            Ok(s) => s,
            Err(_) => break
        };

        let size = match buf_reader.read_u32::<NativeEndian>() {
            Ok(s) => s,
            Err(_) => break
        };

        println!("Chunk-type: {}\nChunk size: {}", chunk_type, size);

        match buf_reader.seek(SeekFrom::Current(size as i64)) {
            Ok(_) => (),
            Err(_) => break,
        };
    }

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

    println!("{:#?}", style);

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("piston: image", [300, 300])
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let rust_logo = Path::new("data/rust.png");
    
    let rust_logo = Texture::from_path(
            &mut window.factory,
            &rust_logo,
            Flip::None,
            &TextureSettings::new()
        ).unwrap();

    window.set_lazy(true);
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);
            image(&rust_logo, c.transform, g);
        });
    }
}

// fn show_file_type(buf: &BufReader<File>) -> BufReader<File> {
//     let mut buffer = [0; 4];
//
//     // read at most five bytes
//     let mut s = String::new();
//
//     let mut handle = buf.take(4);
//     handle.read_to_string(&mut s);
//
//     println!("File Type: {}", s);
//
//     handle.into_inner()
//     // buf.read_exact(&mut buffer);
//     // match std::str::from_utf8(&buffer) {
//     //     Ok(s) => {
//     //         println!("File Type: {}", s)
//     //
//     //     },
//     //     Err(e) => println!("{}", e)
//     // }
// }
