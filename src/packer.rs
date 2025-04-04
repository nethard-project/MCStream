use crate::{
    chunk::{compress_chunk, validate_local_pos, write_chunk_index},
    error::McStreamError,
    header::write_header,
    types::{Block, ChunkData, ChunkIndexEntry, ChunkPos, LocalBlockPos},
    CompressionType,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::{Seek, Write};
use std::path::Path;

/// MCS编码器，用于将建筑数据打包成MCS格式
pub struct McsEncoder {
    compression: CompressionType,
    has_signature: bool,
    chunks: HashMap<ChunkPos, ChunkData>,
    signature: Option<Vec<u8>>,
}

impl McsEncoder {
    /// 创建新的MCS编码器
    pub fn new(compression: CompressionType) -> Self {
        Self {
            compression,
            has_signature: false,
            chunks: HashMap::new(),
            signature: None,
        }
    }

    /// 设置签名数据
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.has_signature = true;
        self.signature = Some(signature);
        self
    }

    /// 添加一个方块
    pub fn add_block(
        &mut self,
        block_id: String,
        x: i32,
        y: i32,
        z: i32,
        nbt: Option<Vec<u8>>,
    ) -> Result<(), McStreamError> {
        if block_id.contains("minecraft:air") {
            return Ok(());
        }

        let chunk_pos = ChunkPos {
            x: x >> 4,
            z: z >> 4,
        };

        let local_pos = LocalBlockPos {
            x: (x & 0xF) as u8,
            y: (y + 64) as u16,
            z: (z & 0xF) as u8,
        };

        validate_local_pos(&local_pos)?;

        let chunk = self.chunks.entry(chunk_pos).or_insert_with(|| ChunkData {
            pos: chunk_pos,
            palette: Vec::new(),
            blocks: Vec::new(),
        });

        let palette_index = match chunk.palette.iter().position(|id| *id == block_id) {
            Some(index) => index as u16,
            None => {
                chunk.palette.push(block_id);
                (chunk.palette.len() - 1) as u16
            }
        };

        chunk.blocks.push(Block {
            palette_index,
            pos: local_pos,
            nbt,
        });

        Ok(())
    }

    /// 添加多个相同类型的方块
    pub fn add_blocks(
        &mut self,
        block_id: String,
        positions: &[(i32, i32, i32)],
        nbt: Option<Vec<u8>>,
    ) -> Result<(), McStreamError> {
        for &(x, y, z) in positions {
            self.add_block(block_id.clone(), x, y, z, nbt.clone())?;
        }
        Ok(())
    }

    /// 添加区块数据
    pub fn add_chunk(&mut self, chunk: ChunkData) -> Result<(), McStreamError> {
        for block in &chunk.blocks {
            validate_local_pos(&block.pos)?;
        }
        self.chunks.insert(chunk.pos, chunk);
        Ok(())
    }

    /// 将所有数据写入MCS文件
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), McStreamError> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if path.exists() {
            std::fs::remove_file(path)?;
        }

        let file = OpenOptions::new().write(true).create_new(true).open(path)?;

        let mut writer = BufWriter::new(file);
        self.write_to_writer(&mut writer)?;
        writer.flush()?;

        Ok(())
    }

    /// 将数据写入到指定的写入器
    fn write_to_writer<W: Write + Seek>(&self, writer: &mut W) -> Result<(), McStreamError> {
        // 检查是否有区块
        if self.chunks.is_empty() {
            return Err(McStreamError::ValidationError(
                "没有区块数据可写入".to_string(),
            ));
        }

        // 1. 首先写入头部（20字节）
        write_header(writer, self.compression, self.has_signature)?;

        // 2. 计算索引表位置：头部大小 = 20字节
        let index_table_offset = 20u32;

        // 修改头部中的索引表偏移值
        writer.seek(std::io::SeekFrom::Start(12))?; // 索引表偏移字段位置
        writer.write_u32::<LittleEndian>(index_table_offset)?;

        // 3. 跳到索引表位置
        writer.seek(std::io::SeekFrom::Start(index_table_offset as u64))?;

        // 4. 准备区块数据
        let mut chunk_index = Vec::new();
        let mut chunk_data = Vec::new();

        for chunk in self.chunks.values() {
            let compressed = compress_chunk(chunk, self.compression)?;
            chunk_index.push(ChunkIndexEntry {
                chunk_x: chunk.pos.x,
                chunk_z: chunk.pos.z,
                data_offset: 0, // 临时值，稍后更新
                compressed_size: compressed.len() as u32,
            });
            chunk_data.push(compressed);
        }

        // 5. 写入区块索引表（先写入长度，后面再更新偏移）
        write_chunk_index(writer, &chunk_index)?;

        // 6. 更新并写入实际的区块数据
        let mut current_offset = writer.stream_position()? as u32;

        for (i, compressed) in chunk_data.iter().enumerate() {
            // 更新区块索引的偏移
            chunk_index[i].data_offset = current_offset;

            // 写入压缩数据
            writer.write_all(compressed)?;

            // 更新下一个区块的偏移
            current_offset += compressed.len() as u32;
        }

        // 7. 回到索引表位置，用更新后的偏移值重新写入
        writer.seek(std::io::SeekFrom::Start(index_table_offset as u64))?;
        write_chunk_index(writer, &chunk_index)?;

        // 8. 跳到文件末尾
        writer.seek(std::io::SeekFrom::End(0))?;

        // 9. 写入签名数据（如果需要）
        if self.has_signature && self.signature.is_some() {
            writer.write_all(self.signature.as_ref().unwrap())?;
        }

        Ok(())
    }

    /// 获取当前存储的区块数据
    pub fn get_chunks(&self) -> &HashMap<ChunkPos, ChunkData> {
        &self.chunks
    }

    /// 清空所有区块数据
    pub fn clear(&mut self) {
        self.chunks.clear();
    }
}
