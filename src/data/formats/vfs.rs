use super::reader::FileReader;
use encoding_rs::EUC_KR;
use memmap::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

struct FileEntry {
    offset: usize,
    size: usize,
}

struct Storage {
    mmap: Mmap,
    files: HashMap<String, FileEntry>,
}

#[derive(Default)]
pub struct VfsIndex {
    root_path: PathBuf,
    base_version: u32,
    current_version: u32,
    storages: Vec<Storage>,
}

pub enum VfsFile<'a> {
    Buffer(Vec<u8>),
    View(&'a [u8]),
}

impl<'a> From<&'a VfsFile<'a>> for FileReader<'a> {
    fn from(file: &'a VfsFile<'a>) -> Self {
        match file {
            VfsFile::Buffer(vec) => FileReader::from(vec),
            VfsFile::View(buf) => FileReader::from(*buf),
        }
    }
}

impl VfsIndex {
    pub fn normalise_path(path: &str) -> String {
        path.replace(r#"\"#, "/").to_uppercase()
    }

    pub fn load(path: &Path) -> Result<VfsIndex, std::io::Error> {
        let data = std::fs::read(path)?;
        let mut reader = FileReader::from(&data);

        let mut index = VfsIndex {
            root_path: {
                if let Some(path) = path.parent() {
                    path.to_owned()
                } else {
                    PathBuf::new()
                }
            },
            ..Default::default()
        };
        index.base_version = reader.read_u32()?;
        index.current_version = reader.read_u32()?;

        let num_vfs = reader.read_u32()? as usize;
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
                continue;
            }

            let file = File::open(index.root_path.join(String::from(filename)))?;
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
                    storage
                        .files
                        .insert(Self::normalise_path(&filename), FileEntry { offset, size });
                }
            }

            index.storages.push(storage);
            reader.set_position(next_vfs_position);
        }

        Ok(index)
    }

    pub fn open_file(&self, path: &str) -> Option<VfsFile> {
        if let Ok(buffer) = std::fs::read(self.root_path.join(path)) {
            return Some(VfsFile::Buffer(buffer));
        }

        let path = Self::normalise_path(path);
        for vfs in &self.storages {
            if let Some(entry) = vfs.files.get(&path) {
                return Some(VfsFile::View(
                    &vfs.mmap[entry.offset..entry.offset + entry.size],
                ));
            }
        }

        None
    }
}
