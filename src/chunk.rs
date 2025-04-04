use crate::{
    compression::{compress_data, compression_type_from_u8, decompress_data},
    error::McStreamError,
    palette,
    types::{Block, ChunkData, ChunkIndexEntry, ChunkPos, LocalBlockPos},
    CompressionType,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};

/// 验证局部坐标是否在有效范围内
pub fn validate_local_pos(pos: &LocalBlockPos) -> Result<(), McStreamError> {
    if pos.x > 15 || pos.z > 15 || pos.y > 383 {
        return Err(McStreamError::CoordinateOutOfRange);
    }
    Ok(())
}

/// 写入区块索引表
pub fn write_chunk_index<W: Write>(
    writer: &mut W,
    entries: &[ChunkIndexEntry],
) -> Result<(), McStreamError> {
    writer.write_u32::<LittleEndian>(entries.len() as u32)?;

    for entry in entries {
        writer.write_i32::<LittleEndian>(entry.chunk_x)?;
        writer.write_i32::<LittleEndian>(entry.chunk_z)?;
        writer.write_u32::<LittleEndian>(entry.data_offset)?;
        writer.write_u32::<LittleEndian>(entry.compressed_size)?;
    }

    Ok(())
}

/// 读取区块索引表
pub fn read_chunk_index<R: Read>(reader: &mut R) -> Result<Vec<ChunkIndexEntry>, McStreamError> {
    let entry_count = reader.read_u32::<LittleEndian>()?;

    let mut entries = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        entries.push(ChunkIndexEntry {
            chunk_x: reader.read_i32::<LittleEndian>()?,
            chunk_z: reader.read_i32::<LittleEndian>()?,
            data_offset: reader.read_u32::<LittleEndian>()?,
            compressed_size: reader.read_u32::<LittleEndian>()?,
        });
    }

    Ok(entries)
}

/// 序列化单个区块为二进制数据
pub fn serialize_chunk(chunk: &ChunkData) -> Result<Vec<u8>, McStreamError> {
    let mut buffer = Vec::new();

    palette::write_palette(&mut buffer, &chunk.palette)?;
    buffer.write_u32::<LittleEndian>(chunk.blocks.len() as u32)?;

    let nbt_blocks: Vec<&Block> = chunk
        .blocks
        .iter()
        .filter(|block| block.nbt.is_some())
        .collect();

    for block in &chunk.blocks {
        buffer.write_u16::<LittleEndian>(block.palette_index)?;
        buffer.write_u8(block.pos.x)?;
        buffer.write_u16::<LittleEndian>(block.pos.y)?;
        buffer.write_u8(block.pos.z)?;
        buffer.write_u8(if block.nbt.is_some() { 0x01 } else { 0x00 })?;
    }

    buffer.write_u32::<LittleEndian>(nbt_blocks.len() as u32)?;

    for block in nbt_blocks {
        if let Some(nbt_data) = &block.nbt {
            buffer.write_u32::<LittleEndian>(nbt_data.len() as u32)?;
            buffer.write_all(nbt_data)?;
        }
    }

    Ok(buffer)
}

/// 反序列化二进制数据为区块
pub fn deserialize_chunk(data: &[u8], pos: ChunkPos) -> Result<ChunkData, McStreamError> {
    let mut cursor = Cursor::new(data);

    let palette = palette::read_palette(&mut cursor)?;
    let block_count = cursor.read_u32::<LittleEndian>()?;

    let mut blocks = Vec::with_capacity(block_count as usize);
    let mut nbt_blocks = Vec::new();

    for _ in 0..block_count {
        let palette_index = cursor.read_u16::<LittleEndian>()?;
        let x = cursor.read_u8()?;
        let y = cursor.read_u16::<LittleEndian>()?;
        let z = cursor.read_u8()?;
        let local_pos = LocalBlockPos::new(x, y, z);

        let flags = cursor.read_u8()?;
        let has_nbt = (flags & 0x01) != 0;

        blocks.push(Block {
            palette_index,
            pos: local_pos,
            nbt: if has_nbt { Some(Vec::new()) } else { None },
        });

        if has_nbt {
            nbt_blocks.push(blocks.len() - 1);
        }
    }

    let nbt_count = cursor.read_u32::<LittleEndian>()?;

    if nbt_count as usize != nbt_blocks.len() {
        return Err(McStreamError::NbtError(
            "NBT数据数量与标记不一致".to_string(),
        ));
    }

    for block_index in nbt_blocks {
        let nbt_len = cursor.read_u32::<LittleEndian>()?;
        let mut nbt_data = vec![0u8; nbt_len as usize];
        cursor.read_exact(&mut nbt_data)?;

        if let Some(block) = blocks.get_mut(block_index) {
            block.nbt = Some(nbt_data);
        }
    }

    Ok(ChunkData {
        pos,
        palette,
        blocks,
    })
}

/// 压缩区块数据
pub fn compress_chunk(
    chunk: &ChunkData,
    compression_type: CompressionType,
) -> Result<Vec<u8>, McStreamError> {
    compress_data(&serialize_chunk(chunk)?, compression_type)
}

/// 解压并反序列化区块数据
pub fn decompress_chunk(
    compressed_data: &[u8],
    compression_type: u8,
    pos: ChunkPos,
) -> Result<ChunkData, McStreamError> {
    let compression = compression_type_from_u8(compression_type)?;
    let decompressed = decompress_data(compressed_data, compression)?;
    deserialize_chunk(&decompressed, pos)
}
