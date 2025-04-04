use crate::{CompressionType, error::McStreamError};
use std::io::{Read, Write};

const BROTLI_BUFFER_SIZE: usize = 4096;
const BROTLI_QUALITY: u32 = 4;
const BROTLI_LGWIN: u32 = 22;

/// 压缩数据
pub fn compress_data(
    data: &[u8], 
    compression_type: CompressionType
) -> Result<Vec<u8>, McStreamError> {
    match compression_type {
        CompressionType::None => Ok(data.to_vec()),
        
        CompressionType::Zstandard => {
            let mut compressed = Vec::new();
            let mut encoder = zstd::Encoder::new(&mut compressed, 3)?;
            encoder.write_all(data)?;
            encoder.finish()?;
            Ok(compressed)
        },
        
        CompressionType::LZ4 => {
            let mut compressed = Vec::new();
            lz4::EncoderBuilder::new()
                .build(&mut compressed)?
                .write_all(data)?;
            Ok(compressed)
        },
        
        CompressionType::Brotli => {
            let mut compressed = Vec::new();
            let mut encoder = brotli::CompressorWriter::new(
                &mut compressed,
                BROTLI_BUFFER_SIZE,
                BROTLI_QUALITY,
                BROTLI_LGWIN,
            );
            encoder.write_all(data)?;
            encoder.flush()?;
            drop(encoder);
            Ok(compressed)
        },
    }
}

/// 解压数据
pub fn decompress_data(
    compressed_data: &[u8], 
    compression_type: CompressionType
) -> Result<Vec<u8>, McStreamError> {
    match compression_type {
        CompressionType::None => Ok(compressed_data.to_vec()),
        
        CompressionType::Zstandard => {
            let mut decompressed = Vec::new();
            let mut decoder = zstd::Decoder::new(compressed_data)?;
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        },
        
        CompressionType::LZ4 => {
            let mut decompressed = Vec::new();
            let mut decoder = lz4::Decoder::new(compressed_data)?;
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        },
        
        CompressionType::Brotli => {
            let mut decompressed = Vec::new();
            let mut decoder = brotli::Decompressor::new(
                compressed_data,
                4096, // buffer size
            );
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        },
    }
}

/// 将压缩类型值转换为枚举
pub fn compression_type_from_u8(value: u8) -> Result<CompressionType, McStreamError> {
    match value {
        0 => Ok(CompressionType::None),
        1 => Ok(CompressionType::Zstandard),
        2 => Ok(CompressionType::LZ4),
        3 => Ok(CompressionType::Brotli),
        _ => Err(McStreamError::UnsupportedCompression(value)),
    }
} 