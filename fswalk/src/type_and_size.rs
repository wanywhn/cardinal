use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone, Copy)]
#[serde(transparent)]
/// type in the high 2 bits, size in the low 46 bits(64T-1 max)
pub struct TypeAndSize([u8; 6]);

impl TypeAndSize {
    pub fn new(r#type: NodeFileType, size: u64) -> Self {
        let bytes = (size.min((1 << 46) - 1) | ((r#type as u64) << 46)).to_le_bytes();
        let mut result = [0u8; 6];
        result.copy_from_slice(&bytes[..6]);
        Self(result)
    }

    pub fn r#type(&self) -> NodeFileType {
        NodeFileType::n(self.0[5] >> 6).unwrap()
    }

    pub fn size(&self) -> u64 {
        let value = u64::from_le_bytes([
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], 0, 0,
        ]);
        value & ((1u64 << 46) - 1)
    }
}

#[derive(
    Debug, Serialize_repr, Deserialize_repr, Encode, Decode, Clone, Copy, enumn::N, PartialEq, Eq,
)]
#[repr(u8)]
pub enum NodeFileType {
    // File occurs a lot, assign it to 0 for better compression ratio(I guess... maybe useful).
    File = 0,
    Dir = 1,
    Symlink = 2,
    Unknown = 3,
}

impl From<fs::FileType> for NodeFileType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_file() {
            NodeFileType::File
        } else if file_type.is_dir() {
            NodeFileType::Dir
        } else if file_type.is_symlink() {
            NodeFileType::Symlink
        } else {
            NodeFileType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_and_size() {
        let max_size = (1u64 << 46) - 1;
        let file_type = NodeFileType::File;
        let ts = TypeAndSize::new(file_type, max_size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), max_size);

        let file_type = NodeFileType::Dir;
        let size = 12345;
        let ts = TypeAndSize::new(file_type, size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), size);

        let file_type = NodeFileType::Symlink;
        let size = 0;
        let ts = TypeAndSize::new(file_type, size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), size);

        let file_type = NodeFileType::Unknown;
        let size = 987654321;
        let ts = TypeAndSize::new(file_type, size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), size);
    }

    #[test]
    fn test_size_overflow() {
        let too_large_size = 1u64 << 46;
        let file_type = NodeFileType::File;
        let ts = TypeAndSize::new(file_type, too_large_size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), (1 << 46) - 1); // size saturating

        let another_large_size = ((1u64 << 46) - 1) + 100;
        let ts = TypeAndSize::new(file_type, another_large_size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), (1 << 46) - 1);

        let max_size = (1u64 << 46) - 1;
        let ts = TypeAndSize::new(file_type, max_size);
        assert_eq!(ts.r#type(), file_type);
        assert_eq!(ts.size(), max_size);
    }
}
