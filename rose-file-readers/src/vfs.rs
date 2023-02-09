use encoding_rs::EUC_KR;
use memmap::{Mmap, MmapOptions};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use crate::{reader::RoseFileReader, VfsError, VfsFile, VfsPath, VirtualFilesystemDevice};

struct FileEntry {
    offset: usize,
    size: usize,
}

struct Storage {
    mmap: Mmap,
    files: HashMap<PathBuf, FileEntry>,
}

#[derive(Default)]
pub struct VfsIndex {
    pub base_version: u32,
    pub current_version: u32,
    storages: Vec<Storage>,
}

impl VfsIndex {
    pub fn load(index_path: &Path) -> Result<VfsIndex, anyhow::Error> {
        let index_root_path = index_path
            .parent()
            .map(|path| path.into())
            .unwrap_or_else(PathBuf::new);
        let data = std::fs::read(index_path)?;
        let mut reader = RoseFileReader::from(&data);

        let base_version = reader.read_u32()?;
        let current_version = reader.read_u32()?;

        let num_vfs = reader.read_u32()? as usize;
        let mut storages = Vec::with_capacity(num_vfs);
        for _ in 0..num_vfs {
            let (filename, _, _) =
                EUC_KR.decode(reader.read_u16_length_bytes()?.split_last().unwrap().1);
            let offset = reader.read_u32()? as u64;

            let next_vfs_position = reader.position();
            reader.set_position(offset);

            let num_files = reader.read_u32()? as usize;
            let _ = reader.read_u32()?;
            let _ = reader.read_u32()?;

            if filename.to_uppercase() == "ROOT.VFS" {
                reader.set_position(next_vfs_position);
                continue;
            }

            let file = File::open(index_root_path.join(String::from(filename)))?;
            let mmap = unsafe { MmapOptions::new().map(&file)? };

            let mut storage = Storage {
                mmap,
                files: HashMap::with_capacity(num_files),
            };

            for _ in 0..num_files {
                let (filename, _, _) =
                    EUC_KR.decode(reader.read_u16_length_bytes()?.split_last().unwrap().1);
                let offset = reader.read_u32()? as usize;
                let size = reader.read_u32()? as usize;
                let _block_size = reader.read_u32()?;
                let is_deleted = reader.read_u8()?;
                let _is_compressed = reader.read_u8()?;
                let _is_encrypted = reader.read_u8()?;
                let _version = reader.read_u32()?;
                let _crc = reader.read_u32()?;

                if is_deleted == 0 {
                    storage.files.insert(
                        VfsPath::normalise_path(&filename),
                        FileEntry { offset, size },
                    );
                }
            }

            storages.push(storage);
            reader.set_position(next_vfs_position);
        }

        Ok(VfsIndex {
            base_version,
            current_version,
            storages,
        })
    }
}

impl VirtualFilesystemDevice for VfsIndex {
    fn open_file(&self, vfs_path: &VfsPath) -> Result<VfsFile, anyhow::Error> {
        for vfs in &self.storages {
            if let Some(entry) = vfs.files.get(vfs_path.path()) {
                return Ok(VfsFile::View(
                    &vfs.mmap[entry.offset..entry.offset + entry.size],
                ));
            }
        }

        Err(VfsError::FileNotFound(vfs_path.path().into()).into())
    }

    fn exists(&self, vfs_path: &VfsPath) -> bool {
        for vfs in &self.storages {
            if vfs.files.get(vfs_path.path()).is_some() {
                return true;
            }
        }

        false
    }
}
