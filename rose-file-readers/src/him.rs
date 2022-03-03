use crate::{reader::FileReader, RoseFile};

pub struct HimFile {
    pub width: u32,
    pub height: u32,
    pub heights: Vec<f32>,
}

impl HimFile {
    pub fn get_clamped(&self, x: usize, y: usize) -> f32 {
        let x = usize::clamp(x, 0, self.width as usize - 1);
        let y = usize::clamp(y, 0, self.height as usize - 1);
        self.heights[y * self.width as usize + x]
    }
}

#[allow(dead_code)]
impl RoseFile for HimFile {
    type ReadOptions = ();

    fn read(mut reader: FileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
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
