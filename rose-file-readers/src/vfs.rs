use encoding_rs::EUC_KR;
use memmap::{Mmap, MmapOptions};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::{reader::FileReader, RoseFile};

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
    root_path: PathBuf,
    base_version: u32,
    current_version: u32,
    storages: Vec<Storage>,
}

pub enum VfsFile<'a> {
    Buffer(Vec<u8>),
    View(&'a [u8]),
}

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("File not found")]
    FileNotFound,
}

impl<'a> From<&'a VfsFile<'a>> for FileReader<'a> {
    fn from(file: &'a VfsFile<'a>) -> Self {
        match file {
            VfsFile::Buffer(vec) => FileReader::from(vec),
            VfsFile::View(buf) => FileReader::from(*buf),
        }
    }
}

#[derive(Debug, Hash, Clone)]
pub struct VfsPath<'a> {
    path: Cow<'a, Path>,
}

impl<'a> VfsPath<'a> {
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn normalise_path(path: &str) -> PathBuf {
        path.replace('\\', "/").to_uppercase().into()
    }
}

impl<'a> From<&'a str> for VfsPath<'_> {
    fn from(path: &'a str) -> Self {
        VfsPath {
            path: Cow::Owned(VfsPath::normalise_path(path)),
        }
    }
}

impl<'a> From<&'a String> for VfsPath<'_> {
    fn from(path: &'a String) -> Self {
        VfsPath {
            path: Cow::Owned(VfsPath::normalise_path(path.as_str())),
        }
    }
}

impl<'a> From<&'a Path> for VfsPath<'_> {
    fn from(path: &'a Path) -> Self {
        VfsPath {
            path: Cow::Owned(VfsPath::normalise_path(path.to_string_lossy().as_ref())),
        }
    }
}

impl<'a> From<PathBuf> for VfsPath<'_> {
    fn from(path: PathBuf) -> Self {
        VfsPath {
            path: Cow::Owned(VfsPath::normalise_path(path.to_string_lossy().as_ref())),
        }
    }
}

impl<'a> From<&'a VfsPath<'a>> for VfsPath<'a> {
    fn from(path: &'a VfsPath<'a>) -> Self {
        VfsPath {
            path: Cow::Borrowed(&path.path),
        }
    }
}

impl VfsIndex {
    pub fn normalise_path(path: &str) -> PathBuf {
        path.replace('\\', "/").to_uppercase().into()
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

    pub fn open_file<'a, P: Into<VfsPath<'a>>>(&self, path: P) -> Option<VfsFile> {
        let vfs_path: VfsPath = path.into();

        for vfs in &self.storages {
            if let Some(entry) = vfs.files.get(vfs_path.path()) {
                return Some(VfsFile::View(
                    &vfs.mmap[entry.offset..entry.offset + entry.size],
                ));
            }
        }

        if let Ok(buffer) = std::fs::read(self.root_path.join(vfs_path.path())) {
            return Some(VfsFile::Buffer(buffer));
        }

        None
    }

    pub fn read_file<'a, T: RoseFile + Sized, P: Into<VfsPath<'a>>>(
        &self,
        path: P,
    ) -> Result<T, anyhow::Error> {
        if let Some(file) = self.open_file(path) {
            RoseFile::read(FileReader::from(&file), &Default::default())
        } else {
            Err(VfsError::FileNotFound.into())
        }
    }

    pub fn read_file_with<'a, T: RoseFile + Sized, P: Into<VfsPath<'a>>>(
        &self,
        path: P,
        options: &T::ReadOptions,
    ) -> Result<T, anyhow::Error> {
        if let Some(file) = self.open_file(path) {
            RoseFile::read(FileReader::from(&file), options)
        } else {
            Err(VfsError::FileNotFound.into())
        }
    }
}