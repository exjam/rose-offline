use crate::{reader::RoseFileReader, RoseFile};

#[derive(Debug)]
pub struct TsiTexture {
    pub filename: String,
}

pub type TsiTextureId = u16;

#[derive(Debug)]
pub struct TsiSprite {
    pub texture_id: TsiTextureId,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub name: String,
}

pub struct TsiFile {
    pub textures: Vec<TsiTexture>,
    pub sprites: Vec<TsiSprite>,
}

impl RoseFile for TsiFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let num_textures = reader.read_u16()? as usize;
        let mut textures = Vec::with_capacity(num_textures);

        for _ in 0..num_textures {
            let filename = reader.read_u16_length_string()?.to_string();
            reader.skip(4);

            textures.push(TsiTexture { filename });
        }

        let total_num_sprites = reader.read_u16()? as usize;
        let mut sprites = Vec::with_capacity(total_num_sprites);

        for _ in 0..num_textures {
            let num_sprites = reader.read_u16()? as usize;

            for _ in 0..num_sprites {
                let texture_id = reader.read_u16()?;

                let left = reader.read_i32()?;
                let top = reader.read_i32()?;
                let right = reader.read_i32()?;
                let bottom = reader.read_i32()?;
                reader.skip(4);

                let name = reader.read_fixed_length_string(32)?.to_string();
                sprites.push(TsiSprite {
                    texture_id,
                    left,
                    top,
                    right,
                    bottom,
                    name,
                });
            }
        }

        Ok(Self { textures, sprites })
    }
}
