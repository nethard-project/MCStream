pub mod chunk;
pub mod compression;
pub mod error;
pub mod header;
pub mod nbt;
pub mod packer;
pub mod palette;
pub mod types;
pub mod unpacker;
pub mod utils;

pub use crate::error::McStreamError;
pub use crate::packer::McsEncoder;
pub use crate::unpacker::McsDecoder;

/// MCStream版本号常量
pub const MCS_VERSION: u16 = 0x0100; // 1.0版本

/// MCStream魔数常量
pub const MCS_MAGIC: &[u8; 8] = b"MCSTRM\0\0";

/// 压缩算法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionType {
    None = 0,
    Zstandard = 1,
    LZ4 = 2,
    Brotli = 3,
}
