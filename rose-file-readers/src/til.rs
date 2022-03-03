use crate::{reader::RoseFileReader, RoseFile};

pub struct TilFile {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<u32>,
}

impl TilFile {
    pub fn get_clamped(&self, x: usize, y: usize) -> u32 {
        let x = usize::clamp(x, 0, self.width as usize - 1);
        let y = usize::clamp(y, 0, self.height as usize - 1);
        self.tiles[y * self.width as usize + x]
    }
}

impl RoseFile for TilFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for _ in 0..height {
            for _ in 0..width {
                reader.skip(3);
                tiles.push(reader.read_u32()?);
            }
        }

        Ok(Self {
            width,
            height,
            tiles,
        })
    }
}
