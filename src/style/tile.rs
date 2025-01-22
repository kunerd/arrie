// FIXME duplicate code
const PAGE_SIZE: usize = 256;
const IMAGE_SIZE: usize = 64;

#[derive(Debug)]
pub struct Tile(pub Vec<u8>);

impl Tile {
    pub fn from_file(id: usize, page: &[u8]) -> Self {
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

        Tile(tile)
    }
}

