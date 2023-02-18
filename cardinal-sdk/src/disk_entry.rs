use bincode::{Decode, Encode};
use std::fs;
use std::{path::PathBuf, time::SystemTime};

use crate::models::DiskEntryRaw;

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum FileType {
    Dir,
    File,
    Symlink,
    Unknown,
}

impl From<fs::FileType> for FileType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_dir() {
            FileType::Dir
        } else if file_type.is_file() {
            FileType::File
        } else if file_type.is_symlink() {
            FileType::Symlink
        } else {
            FileType::Unknown
        }
    }
}

/// Most of the useful information for a disk node.
#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Metadata {
    pub file_type: FileType,
    pub len: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub permissions_read_only: bool,
}

impl From<fs::Metadata> for Metadata {
    fn from(meta: fs::Metadata) -> Self {
        // unwrap is legal here since these things are always available on PC platforms.
        Self {
            file_type: meta.file_type().into(),
            len: meta.len(),
            created: meta.created().unwrap(),
            modified: meta.modified().unwrap(),
            accessed: meta.accessed().unwrap(),
            permissions_read_only: meta.permissions().readonly(),
        }
    }
}

pub struct DiskEntry {
    pub path: PathBuf,
    pub meta: Metadata,
}

const CONFIG: bincode::config::Configuration = bincode::config::standard();

impl TryFrom<DiskEntryRaw> for DiskEntry {
    type Error = bincode::error::DecodeError;
    fn try_from(entry: DiskEntryRaw) -> Result<Self, Self::Error> {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let (meta, _) = bincode::decode_from_slice(&entry.the_meta, CONFIG)?;
        Ok(Self {
            path: OsString::from_vec(entry.the_path).into(),
            meta,
        })
    }
}

impl TryFrom<DiskEntry> for DiskEntryRaw {
    type Error = bincode::error::EncodeError;
    fn try_from(entry: DiskEntry) -> Result<Self, Self::Error> {
        use std::os::unix::ffi::OsStringExt;
        let the_meta = bincode::encode_to_vec(&entry.meta, CONFIG)?;
        Ok(Self {
            the_path: entry.path.into_os_string().into_vec(),
            the_meta,
        })
    }
}
