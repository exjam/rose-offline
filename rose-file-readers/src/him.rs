use crate::{reader::RoseFileReader, RoseFile};

#[derive(Clone)]
pub struct HimFile {
    pub width: u32,
    pub height: u32,
    pub heights: Vec<f32>,
}

impl HimFile {
    pub fn get_clamped(&self, x: i32, y: i32) -> f32 {
        let x = i32::clamp(x, 0, self.width as i32 - 1) as usize;
        let y = i32::clamp(y, 0, self.height as i32 - 1) as usize;
        self.heights[y * self.width as usize + x]
    }
}

impl RoseFile for HimFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let width = reader.read_u32()?;
        let height = reader.read_u32()?;
        reader.skip(8);
        let mut heights = Vec::with_capacity((width * height) as usize);

        for _ in 0..height {
            for _ in 0..width {
                heights.push(reader.read_f32()?);
            }
        }

        Ok(Self {
            width,
            height,
            heights,
        })
    }
}
