use crate::{
    CompressionType,
    error::McStreamError,
    types::{ChunkPos, McsData, McsHeader, ChunkData, ChunkIndexEntry},
    header::read_header,
    chunk::{read_chunk_index, decompress_chunk},
    utils::validate_file_size,
};
use std::io::{Read, Seek, SeekFrom};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use rayon::prelude::*;
use sha2::{Sha256, Digest};

/// MCS解码器，用于将MCS格式解包成建筑数据
pub struct McsDecoder {
    header: McsHeader,
    chunks: HashMap<ChunkPos, ChunkData>,
    data_hash: [u8; 32],
    signature: Option<Vec<u8>>,
}

impl McsDecoder {
    /// 从MCS文件读取数据
    pub fn from_file<P: AsRef<Path> + std::marker::Sync + std::marker::Copy>(path: P) -> Result<Self, McStreamError> {
        let file = File::open(&path)?;
        let file_size = file.metadata()?.len();
        
        if file_size < 20 {  // 最小文件头大小
            return Err(McStreamError::ValidationError(format!("文件过小，大小为 {} 字节", file_size)));
        }
        
        let mut reader = BufReader::new(file);
        
        // 验证文件大小
        validate_file_size(&mut reader)?;
        
        // 读取头部
        let header = read_header(&mut reader)?;
        
        // 跳转到索引表位置
        if header.index_table_offset as u64 >= file_size {
            return Err(McStreamError::ValidationError(format!(
                "索引表偏移 ({}) 超出文件大小 ({})", 
                header.index_table_offset, file_size
            )));
        }
        
        reader.seek(SeekFrom::Start(header.index_table_offset as u64))?;
        
        // 读取区块索引表
        let index_entries = read_chunk_index(&mut reader)?;
        
        // 检查是否有区块
        if index_entries.is_empty() {
            return Err(McStreamError::ChunkIndexError);
        }
        
        // 检查所有区块的偏移是否在文件范围内
        for entry in &index_entries {
            let chunk_end = (entry.data_offset + entry.compressed_size) as u64;
            if chunk_end > file_size {
                return Err(McStreamError::ValidationError(format!(
                    "区块数据超出文件范围，结束位置 {} 超出文件大小 {}", 
                    chunk_end, file_size
                )));
            }
        }
        
        // 并行读取和解压所有区块
        let compression_type = header.compression;
        let chunks: Result<HashMap<ChunkPos, ChunkData>, McStreamError> = index_entries
            .par_iter()
            .map(|entry| {
                // 跳转到区块数据位置
                let mut chunk_reader = std::fs::File::open(&path)?;
                chunk_reader.seek(SeekFrom::Start(entry.data_offset as u64))?;
                
                // 读取压缩数据
                let mut compressed_data = vec![0u8; entry.compressed_size as usize];
                chunk_reader.read_exact(&mut compressed_data)?;
                
                // 创建区块坐标
                let pos = ChunkPos::new(entry.chunk_x, entry.chunk_z);
                
                // 解压并解析区块数据
                let chunk = decompress_chunk(&compressed_data, compression_type, pos)?;
                
                Ok((pos, chunk))
            })
            .collect();
        
        // 处理区块结果
        let chunks = chunks?;
        
        // 计算最后一个区块数据的结束位置，用于读取尾部
        let last_entry = index_entries.iter().max_by_key(|e| e.data_offset + e.compressed_size)
            .ok_or(McStreamError::ChunkIndexError)?;
        let footer_offset = (last_entry.data_offset + last_entry.compressed_size) as u64;
        
        // 确保签名在文件范围内
        if footer_offset > file_size {
            return Err(McStreamError::ValidationError(format!(
                "文件格式错误：区块数据结束位置 ({}) 超出文件大小 ({})", 
                footer_offset, file_size
            )));
        }
        
        // 如果区块数据正好到文件末尾，则没有签名数据
        let signature = if footer_offset < file_size {
            // 跳转到尾部位置读取可能的签名
            reader.seek(SeekFrom::Start(footer_offset))?;
            
            // 计算文件内容的哈希
            let mut file = File::open(&path)?;
            let mut file_content = vec![0u8; footer_offset as usize];
            file.read_exact(&mut file_content)?;
            
            let mut hasher = Sha256::new();
            hasher.update(&file_content);
            
            // 读取剩余内容作为签名
            let mut signature_data = Vec::new();
            reader.read_to_end(&mut signature_data)?;
            
            if !signature_data.is_empty() && (header.flags & 0x01) != 0 {
                Some(signature_data)
            } else {
                None
            }
        } else {
            None
        };
        
        // 计算数据哈希（无论有没有签名）
        let mut file = File::open(&path)?;
        let content_size = std::cmp::min(footer_offset as usize, file_size as usize);
        let mut file_content = vec![0u8; content_size];
        file.read_exact(&mut file_content)?;
        
        let mut hasher = Sha256::new();
        hasher.update(&file_content);
        let data_hash = hasher.finalize();
        
        Ok(Self {
            header,
            chunks,
            data_hash: data_hash.into(),
            signature,
        })
    }
    
    /// 获取区块数据
    pub fn get_chunks(&self) -> &HashMap<ChunkPos, ChunkData> {
        &self.chunks
    }
    
    /// 获取指定坐标的区块
    pub fn get_chunk(&self, x: i32, z: i32) -> Option<&ChunkData> {
        self.chunks.get(&ChunkPos::new(x, z))
    }
    
    /// 转换为McsData结构
    pub fn to_mcs_data(&self) -> McsData {
        McsData {
            header: self.header.clone(),
            chunks: self.chunks.clone(),
            data_hash: self.data_hash,
            signature: self.signature.clone(),
        }
    }
    
    /// 获取文件头
    pub fn header(&self) -> &McsHeader {
        &self.header
    }
    
    /// 获取数据哈希
    pub fn data_hash(&self) -> &[u8; 32] {
        &self.data_hash
    }
    
    /// 获取签名（如果有）
    pub fn signature(&self) -> Option<&Vec<u8>> {
        self.signature.as_ref()
    }
    
    /// 获取压缩算法类型
    pub fn compression_type(&self) -> CompressionType {
        match self.header.compression {
            0 => CompressionType::None,
            1 => CompressionType::Zstandard,
            2 => CompressionType::LZ4,
            3 => CompressionType::Brotli,
            _ => CompressionType::None, // 不应该发生，因为在read_header时已验证
        }
    }
}

/// 从MCS文件读取区块索引（不加载区块数据）
pub fn read_mcs_index<P: AsRef<Path>>(path: P) -> Result<Vec<ChunkIndexEntry>, McStreamError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    
    // 读取头部
    let header = read_header(&mut reader)?;
    
    // 跳转到索引表位置
    reader.seek(SeekFrom::Start(header.index_table_offset as u64))?;
    
    // 读取区块索引表
    read_chunk_index(&mut reader)
} 