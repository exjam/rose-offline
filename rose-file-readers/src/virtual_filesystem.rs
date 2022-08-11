use anyhow::Context;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{reader::RoseFileReader, RoseFile};

pub enum VfsFile<'a> {
    Buffer(Vec<u8>),
    View(&'a [u8]),
}

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("File {0} not found")]
    FileNotFound(PathBuf),
}

impl<'a> From<&'a VfsFile<'a>> for RoseFileReader<'a> {
    fn from(file: &'a VfsFile<'a>) -> Self {
        match file {
            VfsFile::Buffer(vec) => RoseFileReader::from(vec),
            VfsFile::View(buf) => RoseFileReader::from(*buf),
        }
    }
}

#[derive(Default, Debug, Hash, Clone)]
pub struct VfsPathBuf {
    path: PathBuf,
}

impl VfsPathBuf {
    pub fn new(path: &str) -> Self {
        VfsPathBuf {
            path: VfsPath::normalise_path(path),
        }
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
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

    pub fn normalise_path(path: &str) -> PathBuf {
        path.replace('\\', "/")
            .to_uppercase()
            .trim_start()
            .trim_end()
            .into()
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

impl From<PathBuf> for VfsPath<'_> {
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

impl<'a> From<&'a VfsPathBuf> for VfsPath<'a> {
    fn from(path: &'a VfsPathBuf) -> Self {
        VfsPath {
            path: Cow::Borrowed(&path.path),
        }
    }
}

pub trait VirtualFilesystemDevice {
    fn open_file<'a>(&self, path: &'a VfsPath) -> Result<VfsFile, anyhow::Error>;
    fn exists(&self, path: &VfsPath) -> bool;
}

pub struct HostFilesystemDevice {
    pub root_path: PathBuf,
}

impl HostFilesystemDevice {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }
}

impl VirtualFilesystemDevice for HostFilesystemDevice {
    fn open_file<'a>(&self, vfs_path: &'a VfsPath) -> Result<VfsFile, anyhow::Error> {
        let buffer = std::fs::read(self.root_path.join(vfs_path.path()))
            .map_err(|_| VfsError::FileNotFound(vfs_path.path().into()))?;
        Ok(VfsFile::Buffer(buffer))
    }

    fn exists(&self, vfs_path: &VfsPath) -> bool {
        self.root_path.join(vfs_path.path()).exists()
    }
}

pub struct VirtualFilesystem {
    pub devices: Vec<Box<dyn VirtualFilesystemDevice + Send + Sync>>,
}

impl VirtualFilesystem {
    pub fn new(devices: Vec<Box<dyn VirtualFilesystemDevice + Send + Sync>>) -> Self {
        Self { devices }
    }

    pub fn exists<'a, P: Into<VfsPath<'a>>>(&self, path: P) -> bool {
        let vfs_path: VfsPath = path.into();

        for device in &self.devices {
            if device.exists(&vfs_path) {
                return true;
            }
        }

        false
    }

    pub fn open_file<'a>(&self, path: impl Into<VfsPath<'a>>) -> Result<VfsFile, anyhow::Error> {
        let vfs_path: VfsPath = path.into();

        for device in &self.devices {
            match device.open_file(&vfs_path) {
                Ok(file) => return Ok(file),
                Err(error) => {
                    match error.downcast_ref::<VfsError>() {
                        Some(VfsError::FileNotFound(_)) => continue,
                        None => return Err(error),
                    };
                }
            }
        }

        Err(VfsError::FileNotFound(vfs_path.path().into()).into())
    }

    pub fn read_file<'a, T: RoseFile + Sized, P: Into<VfsPath<'a>>>(
        &self,
        path: P,
    ) -> Result<T, anyhow::Error> {
        let vfs_path: VfsPath = path.into();

        let file = self.open_file(&vfs_path)?;
        RoseFile::read(RoseFileReader::from(&file), &Default::default())
            .with_context(|| format!("Failed to read {}", vfs_path.path().to_string_lossy()))
    }

    pub fn read_file_with<'a, T: RoseFile + Sized, P: Into<VfsPath<'a>>>(
        &self,
        path: P,
        options: &T::ReadOptions,
    ) -> Result<T, anyhow::Error> {
        let vfs_path: VfsPath = path.into();

        let file = self.open_file(&vfs_path)?;
        RoseFile::read(RoseFileReader::from(&file), options)
            .with_context(|| format!("Failed to read {}", vfs_path.path().to_string_lossy()))
    }
}
