use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum McStreamError {
    #[error("IO错误: {0}")]
    Io(#[from] io::Error),
    
    #[error("无效的魔数")]
    InvalidMagic,
    
    #[error("不支持的版本: {0}")]
    UnsupportedVersion(u16),
    
    #[error("不支持的压缩类型: {0}")]
    UnsupportedCompression(u8),
    
    #[error("区块索引错误")]
    ChunkIndexError,
    
    #[error("压缩错误: {0}")]
    CompressionError(String),
    
    #[error("解压错误: {0}")]
    DecompressionError(String),
    
    #[error("NBT解析错误: {0}")]
    NbtError(String),
    
    #[error("调色板错误: {0}")]
    PaletteError(String),
    
    #[error("文件太大，超过4GB限制")]
    FileTooLarge,

    #[error("坐标超出范围")]
    CoordinateOutOfRange,
    
    #[error("调色板包含空气方块")]
    AirInPalette,
    
    #[error("校验错误: {0}")]
    ValidationError(String),
} 