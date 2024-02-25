use std::fs::File;
use std::num::Wrapping;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use memmap::{Mmap, MmapOptions};

use crate::{
    reader::{ReadError, RoseFileReader},
    VfsError, VfsFile, VfsPath, VirtualFilesystemDevice,
};

/// VFS format used by iRosePH and iRose Odyssey, seems to be based upon TitanVFS but
/// with changes to support multiple VFS files and uses a different xor key.
#[derive(Default)]
pub struct IrosePhVfsIndex {
    storages: Vec<Storage>,
}

struct FileEntry {
    offset: usize,
    size: usize,
}

struct Storage {
    mmap: Mmap,
    files: HashMap<u32, FileEntry>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct FileNameHash {
    hash: u32,
}

impl FileNameHash {
    fn new(hash: u32) -> Self {
        Self { hash }
    }
}

const HASH_TABLE: [u32; 256] = [
    0x697A5, 0x6045C, 0xAB4E2, 0x409E4, 0x71209, 0x32392, 0xA7292, 0xB09FC, 0x4B658, 0xAAAD5,
    0x9B9CF, 0xA326A, 0x8DD12, 0x38150, 0x8E14D, 0x2EB7F, 0xE0A56, 0x7E6FA, 0xDFC27, 0xB1301,
    0x8B4F7, 0xA7F70, 0xAA713, 0x6CC0F, 0x6FEDF, 0x2EC87, 0xC0F1C, 0x45CA4, 0x30DF8, 0x60E99,
    0xBC13E, 0x4E0B5, 0x6318B, 0x82679, 0x26EF2, 0x79C95, 0x86DDC, 0x99BC0, 0xB7167, 0x72532,
    0x68765, 0xC7446, 0xDA70D, 0x9D132, 0xE5038, 0x2F755, 0x9171F, 0xCB49E, 0x6F925, 0x601D3,
    0x5BD8A, 0x2A4F4, 0x9B022, 0x706C3, 0x28C10, 0x2B24B, 0x7CD55, 0xCA355, 0xD95F4, 0x727BC,
    0xB1138, 0x9AD21, 0xC0ACA, 0xCD928, 0x953E5, 0x97A20, 0x345F3, 0xBDC03, 0x7E157, 0x96C99,
    0x968EF, 0x92AA9, 0xC2276, 0xA695D, 0x6743B, 0x2723B, 0x58980, 0x66E08, 0x51D1B, 0xB97D2,
    0x6CAEE, 0xCC80F, 0x3BA6C, 0xB0BF5, 0x9E27B, 0xD122C, 0x48611, 0x8C326, 0xD2AF8, 0xBB3B7,
    0xDED7F, 0x4B236, 0xD298F, 0xBE912, 0xDC926, 0xC873F, 0xD0716, 0x9E1D3, 0x48D94, 0x9BD91,
    0x5825D, 0x55637, 0xB2057, 0xBCC6C, 0x460DE, 0xAE7FB, 0x81B03, 0x34D8F, 0xC0528, 0xC9B59,
    0x3D260, 0x6051D, 0x93757, 0x8027F, 0xB7C34, 0x4A14E, 0xB12B8, 0xE4945, 0x28203, 0xA1C0F,
    0xAA382, 0x46ABB, 0x330B9, 0x5A114, 0xA754B, 0xC68D0, 0x9040E, 0x6C955, 0xBB1EF, 0x51E6B,
    0x9FF21, 0x51BCA, 0x4C879, 0xDFF70, 0x5B5EE, 0x29936, 0xB9247, 0x42611, 0x2E353, 0x26F3A,
    0x683A3, 0xA1082, 0x67333, 0x74EB7, 0x754BA, 0x369D5, 0x8E0BC, 0xABAFD, 0x6630B, 0xA3A7E,
    0xCDBB1, 0x8C2DE, 0x92D32, 0x2F8ED, 0x7EC54, 0x572F5, 0x77461, 0xCB3F5, 0x82C64, 0x35FE0,
    0x9203B, 0xADA2D, 0xBAEBD, 0xCB6AF, 0xC8C9A, 0x5D897, 0xCB727, 0xA13B3, 0xB4D6D, 0xC4929,
    0xB8732, 0xCCE5A, 0xD3E69, 0xD4B60, 0x89941, 0x79D85, 0x39E0F, 0x6945B, 0xC37F8, 0x77733,
    0x45D7D, 0x25565, 0xA3A4E, 0xB9F9E, 0x316E4, 0x36734, 0x6F5C3, 0xA8BA6, 0xC0871, 0x42D05,
    0x40A74, 0x2E7ED, 0x67C1F, 0x28BE0, 0xE162B, 0xA1C0F, 0x2F7E5, 0xD505A, 0x9FCC8, 0x78381,
    0x29394, 0x53D6B, 0x7091D, 0xA2FB1, 0xBB942, 0x29906, 0xC412D, 0x3FCD5, 0x9F2EB, 0x8F0CC,
    0xE25C3, 0x7E519, 0x4E7D9, 0x5F043, 0xBBA1B, 0x6710A, 0x819FB, 0x9A223, 0x38E47, 0xE28AD,
    0xB690B, 0x42328, 0x7CF7E, 0xAE108, 0xE54BA, 0xBA5A1, 0xA09A6, 0x9CAB7, 0xDB2B3, 0xA98CC,
    0x5CEBA, 0x9245D, 0x5D083, 0x8EA21, 0xAE349, 0x54940, 0x8E557, 0x83EFD, 0xDC504, 0xA6059,
    0xB85C9, 0x9D162, 0x7AEB6, 0xBED34, 0xB4963, 0xE367B, 0x4C891, 0x9E42C, 0xD4304, 0x96EAA,
    0xD5D69, 0x866B8, 0x83508, 0x7BAEC, 0xD03FD, 0xDA122,
];
const HASH_SEED1: u32 = 0xDEADC0DEu32;
const HASH_SEED2: u32 = 0x7FED7FEDu32;

impl From<&str> for FileNameHash {
    fn from(path: &str) -> Self {
        let path = path.replace('/', "\\").replace("\\\\", "\\").to_uppercase();

        if path.is_empty() {
            Self::new(0)
        } else {
            let mut seed1 = Wrapping(HASH_SEED1);
            let mut seed2 = Wrapping(HASH_SEED2);

            for ch in path
                .chars()
                .map(|c| Wrapping(c.to_ascii_uppercase() as u32))
            {
                seed1 += seed2;
                seed2 *= Wrapping(0x21);
                seed1 ^= HASH_TABLE[ch.0 as usize];
                seed2 = seed2 + seed1 + ch + Wrapping(3);
            }

            Self::new(seed1.0)
        }
    }
}

const XOR_KEY_START: u32 = 0x4CDDCCC5;
const XOR_KEY_LEN: usize = 0xFFF;

fn xor_transform_n<const N: usize>(src: &[u8; N], xor_key_offset: usize) -> [u8; N] {
    std::array::from_fn(|index| {
        let xor_key_index = ((xor_key_offset + index) & XOR_KEY_LEN) as u32;
        let xor_key =
            ((XOR_KEY_START.wrapping_add(xor_key_index * 3)).wrapping_mul(50) & 0xFF) as u8;
        src[index] ^ xor_key
    })
}

trait IrosePhXorRead {
    fn read_u32_xor(&mut self, xor_key_offset: usize) -> Result<u32, ReadError>;
    fn read_u64_xor(&mut self, xor_key_offset: usize) -> Result<u64, ReadError>;
}

impl<'a> IrosePhXorRead for RoseFileReader<'a> {
    fn read_u32_xor(&mut self, xor_key_offset: usize) -> Result<u32, ReadError> {
        let value = self.read_u32()?;
        Ok(u32::from_le_bytes(xor_transform_n(
            &value.to_le_bytes(),
            xor_key_offset,
        )))
    }

    fn read_u64_xor(&mut self, xor_key_offset: usize) -> Result<u64, ReadError> {
        let value = self.read_u64()?;
        Ok(u64::from_le_bytes(xor_transform_n(
            &value.to_le_bytes(),
            xor_key_offset,
        )))
    }
}

impl IrosePhVfsIndex {
    pub fn load(index_path: &Path) -> Result<Self, anyhow::Error> {
        let index_root_path: PathBuf = index_path
            .parent()
            .map(|path| path.into())
            .unwrap_or_default();
        let data = std::fs::read(index_path)?;
        let mut reader = RoseFileReader::from(&data);
        let num_vfs = reader.read_u32_xor(0)? as usize;
        reader.skip(8); // unknown

        let mut storages = Vec::with_capacity(num_vfs);
        for _ in 0..num_vfs {
            let filename = reader.read_u16_length_string()?;
            let idx_offset = reader.read_u32()?;

            let restore_position = reader.position();
            reader.set_position(idx_offset as u64);

            let num_files = reader.read_u32_xor(0)? as usize;
            reader.skip(8); // unknown

            let file = File::open(index_root_path.join(String::from(filename)))?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };

            let mut storage = Storage {
                mmap,
                files: HashMap::with_capacity(num_files),
            };

            for _ in 0..num_files {
                let _unknown_0x0 = reader.read_u32_xor(0)?;
                let _unknown_0x4: u32 = reader.read_u32_xor(4)?;
                let file_offset = reader.read_u32_xor(8)? as usize;
                let _unknown_0xc = reader.read_u32_xor(12)?;
                let file_size = reader.read_u32_xor(16)? as usize;
                let _unknown_0x14 = reader.read_u32_xor(20)?;
                let file_path_hash = reader.read_u32_xor(24)?;

                storage.files.insert(
                    file_path_hash,
                    FileEntry {
                        offset: file_offset,
                        size: file_size,
                    },
                );
            }

            storages.push(storage);
            reader.set_position(restore_position);
        }

        Ok(Self { storages })
    }
}

impl VirtualFilesystemDevice for IrosePhVfsIndex {
    fn open_file(&self, vfs_path: &VfsPath) -> Result<VfsFile, anyhow::Error> {
        let path_str = vfs_path.path().to_str().unwrap();
        let path_hash = FileNameHash::from(path_str).hash;

        for storage in &self.storages {
            if let Some(entry) = storage.files.get(&path_hash) {
                return Ok(VfsFile::View(
                    &storage.mmap[entry.offset..entry.offset + entry.size],
                ));
            }
        }

        Err(VfsError::FileNotFound(vfs_path.path().into()).into())
    }

    fn exists(&self, vfs_path: &VfsPath) -> bool {
        let path_str = vfs_path.path().to_str().unwrap();
        let path_hash = FileNameHash::from(path_str).hash;

        for vfs in &self.storages {
            if vfs.files.get(&path_hash).is_some() {
                return true;
            }
        }

        false
    }
}
